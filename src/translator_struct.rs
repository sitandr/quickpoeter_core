
use std::fmt::Formatter;
use std::fmt::Display;
use crate::translator_ru::{Vowel, Consonant, transcript, symbol_id};
use crate::reader::{GeneralSettings, MiscSettings, StressSettings, ConsonantStructureSettings, AlliterationSettings};
use crate::reader::VECTOR_DIM;

#[derive(Debug, Copy, Clone)]
pub enum Accent{
	NoAccent,
	Primary,
	Secondary,
}

pub trait Phone{
	fn distance(&self, second: &Self) -> f32;
	fn from_vec(v: &Vec<char>) -> Self;
	fn contains_char(c: &char) -> bool;
}

#[derive(Debug)]
pub struct Syll{
	// to get simplier logic, sylls are defined as starting from vowel
	// -_a_nd-
	leading_vowel: Option<Vowel>, // None at first if starting from cons
	trailing_consonants: Vec<Consonant>
}

#[derive(Debug)]
pub struct Word{
	// unlike python version, the letter order stays the same
	pub sylls: Vec<Syll>, // syllables
	pub src: String,
	pub meaning: Option<[f32; VECTOR_DIM]>,
	/// true means it has only abstract vowels, so we can skip
	/// all cons metrics when measuring distance
	pub only_stress_structure: bool
}

impl Word{
	pub fn new(w: &str, is_adj: bool, meaning: Option<[f32; VECTOR_DIM]>) -> Self {
		let unproc = word_to_unprocessed_vecs(w, is_adj);
		let mut sylls = vec![];

		let mut l_vowel  = None;
		let mut t_cons = vec![];

		for mut phone_vec in unproc{
			let type_ = phone_vec.pop().unwrap();
			if type_ == 'v'{
				if l_vowel.is_some() || t_cons.len() > 0{ // this is not start of the word
					sylls.push(Syll{leading_vowel: l_vowel, trailing_consonants: t_cons});
				}
				l_vowel = Some(Vowel::from_vec(&phone_vec));
				t_cons = vec![];
			}
			else{
				t_cons.push(Consonant::from_vec(&phone_vec));
			}

		}
		sylls.push(Syll{leading_vowel: l_vowel, trailing_consonants: t_cons});

		Self{sylls, src: w.to_string(),
			 meaning, only_stress_structure: false}
	}

	/// constructs new only_stress_structure word
	pub fn new_abstract(w: &str) -> Self{
		let sylls = w.chars().map(|l| match l{
			'+' => Vowel{letter: symbol_id!(+), accent: Accent::NoAccent},
			'!' => Vowel{letter: symbol_id!(!), accent: Accent::Primary},
			_ => unreachable!("Bad identifier, {}", l)
		}).map(|stress| Syll{leading_vowel: Some(stress), trailing_consonants: vec![]}).collect();
		Self{sylls: sylls, src: w.to_string(), meaning: None, only_stress_structure: true}
	}

	fn has_cons_end(&self) -> bool{
		self.sylls.last().unwrap().trailing_consonants.len() > 0
	}

	/// return (min, max) by len
	pub fn get_sorted_by_sylls<'a>(one: &'a Self, other: &'a Self) -> (&'a Self, &'a Self){
		if one.sylls.len() > other.sylls.len(){
			(other, one)
		}
		else{
			(one, other)
		}
	}

	pub fn measure_vowel_dist(&self, other: &Self, sett: &StressSettings) -> f32{
		let mut dist = 0.0;
		
		for i1 in 0..self.sylls.len(){ // self is smaller
			let i2 = other.sylls.len() - self.sylls.len() + i1;
			let s1 = &self.sylls[i1];
			let s2 = &other.sylls[i2];
			if let Some(v1) = &s1.leading_vowel{
				if let Some(v2) = &s2.leading_vowel{
					dist += v1.accent_distance(v2, sett);
				}
			}
		}
		dist/(self.sylls.len() as f32).powf(sett.asympt)*sett.weight
	}

	pub fn measure_cons_dist(&self, other: &Self, sett: &AlliterationSettings) -> f32{
		let mut dist = 0.0;

		for (is, il, c1) in self.into_iter(){
			for (is2, il2, c2) in other.into_iter(){
				let slength = self.sylls[is].trailing_consonants.len();
				let slength2 = other.sylls[is2].trailing_consonants.len();
				let sum_syl_len = (slength + slength2) as f32;

				let d1 = (self.sylls.len() - is) as f32 + (slength - il) as f32 /sum_syl_len;
				let d2 = (other.sylls.len() - is2) as f32 + (slength2 - il2) as f32 /sum_syl_len;

                let mut k  = ((d1 - d2).abs() +  sett.shift_coord).powf(sett.pow_coord_delta);
                k *= (d1 + d2 + sett.shift_syll_ending).powf(sett.pow_syll_ending);     
                dist += c1.distance(c2)/k;
			}
		}
		dist/(self.sylls.len() as f32).powf(sett.asympt)*sett.weight
	}

	pub fn measure_struct_dist(&self, other: &Self, sett: &ConsonantStructureSettings) -> f32{
		let mut dist = 0.0;
		
		for i1 in 0..self.sylls.len(){ // self is smaller
			let i2 = other.sylls.len() - self.sylls.len() + i1;
			let s1 = &self.sylls[i1];
			let s2 = &other.sylls[i2];
			dist += ((s1.trailing_consonants.len() as f32 - s2.trailing_consonants.len() as f32)).abs().powf(sett.pow);
		}
		dist/(self.sylls.len() as f32).powf(sett.asympt)*sett.weight
	}

	pub fn measure_misc(&self, other: &Self, sett: &MiscSettings) -> f32{
		let mut dist = 0.0;
		if self.has_cons_end() != other.has_cons_end(){
			dist += sett.same_cons_end;
		}

		let length_diff: f32 = ((other.sylls.len() - self.sylls.len()) as f32).abs();
		dist += sett.length_diff_fine * length_diff;
		dist
	}

	pub fn measure_distance(&self, other: &Self, gs: &GeneralSettings) -> (f32, f32, f32, f32){
		let (first, second) = Self::get_sorted_by_sylls(self, other);
		let misc;
		let vowel;
		let cons;
		let structure;

		if first.only_stress_structure || second.only_stress_structure{
			vowel = first.measure_vowel_dist(second, &gs.stresses);
			let mut diff = second.sylls.len() - first.sylls.len();
			if second.sylls.first().unwrap().leading_vowel.is_none(){
				diff = diff.wrapping_sub(1);
			}
			
			if diff != 0{
				misc = 100_000.0;
			}
			else{
				misc = 0.0;
			}
			cons = 0.0;
			structure = 0.0;
		}
		else{

			misc = first.measure_misc(second, &gs.misc);
			vowel = first.measure_vowel_dist(second, &gs.stresses);
			cons = first.measure_cons_dist(second, &gs.alliteration);
			structure = first.measure_struct_dist(second, &gs.consonant_structure);
		}
		(misc, vowel, cons, structure)
	}
	fn into_iter(&self) -> WordConsIterator{
		WordConsIterator::new(self)
	}

	/// Returns position of primary stress and vec of positions of secondary
	/// **IMPORTANT!** Returns the number of *vowel* in letter notation (starting from 0).
	#[allow(dead_code)] 
	pub fn get_stresses(&self) -> (usize, Vec<usize>){
		let mut primary = usize::MAX;
		let mut secondary = vec![];
		let mut offset = 0;
		for (ind, syll) in self.sylls.iter().enumerate(){
			if let Some(vowel) = &syll.leading_vowel{
				match vowel.accent{
					Accent::Primary => primary = ind - offset,
					Accent::Secondary => secondary.push(ind - offset),
					Accent::NoAccent => {}
				}
			}
			else{
				offset += 1;
			}
		}
		assert_ne!(primary, usize::MAX);
		(primary, secondary)
	}
}

/// returns vec of vecs, where each of them is like [letter, …signs… (many possible), v/c (vowel or consonant marker)]
fn word_to_unprocessed_vecs(w: &str, is_adj: bool) -> Vec<Vec<char>>{
	let w = transcript(w, is_adj);
	let mut res = vec![];
	let mut this_vec = vec![];
	let mut current_type: char = '?'; // unknown type

	for l in w.chars(){
		// stores tha type of new letter
		let new_current = {
			if Vowel::contains_char(&l){Some('v')}
			else if Consonant::contains_char(&l){Some('c')}
			else {None} // some symbol
		};

		if let Some(new_current) = new_current{
			if current_type != '?'{
				this_vec.push(current_type);
				res.push(this_vec);
				this_vec = vec![];
			}
			current_type = new_current;
		}
		this_vec.push(l);
	}
	this_vec.push(current_type);
	res.push(this_vec);
	res
}

impl Display for Word{

	fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
		write!(f, "{}", self.src)
	}
}


struct WordConsIterator<'a>{
	word: &'a Word,
	count_syll: usize,
	count_lyll: usize,
}

impl<'b> Iterator for WordConsIterator<'b>{
	type Item = (usize, usize, &'b Consonant);

	fn next(&mut self) -> Option<Self::Item>{
		let syll = &self.word.sylls[self.count_syll];
		if self.count_lyll >= syll.trailing_consonants.len(){
			self.count_lyll = 0;
			self.count_syll += 1;
			if self.count_syll >= self.word.sylls.len(){
				None
			}
			else{
				self.next()
			}
		}
		else{
			let cl = self.count_lyll;
			self.count_lyll += 1;
			Some((self.count_syll, cl, &syll.trailing_consonants[cl]))
		}

	}
}

impl<'b> WordConsIterator<'b>{
	fn new(w: &'b Word) -> Self{
		WordConsIterator{word: w, count_syll:0, count_lyll:0}
	}
}

#[test]
fn check_stress(){
	assert_eq!(Word::new("ещё", false, None).get_stresses().0, 1);
	assert_eq!(Word::new("лома'ть", false, None).get_stresses().0, 1);
	assert_eq!(Word::new("ско'лько", false, None).get_stresses().0, 0);
}