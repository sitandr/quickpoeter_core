
use std::fmt::Formatter;
use std::fmt::Display;
use crate::translator_ru::{Vowel, Consonant, transcript};
use crate::reader::{GeneralSettings, MiscSettings, StressSettings, ConsonantStructureSettings, AlliterationSettings, MeaningSettings};


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
	src: String
}

impl Word{
	pub fn new(w: &str, is_adj: bool) -> Self {
		let unproc = word_to_unprocessed_vecs(w, is_adj);
		let mut sylls = vec![];

		let mut l_vowel  = None;
		let mut t_cons = vec![];

		for mut phone_vec in unproc{
			let type_ = phone_vec.pop().unwrap();
			if type_ == 'v'{
				sylls.push(Syll{leading_vowel: l_vowel, trailing_consonants: t_cons});
				l_vowel = Some(Vowel::from_vec(&phone_vec));
				t_cons = vec![];
			}
			else{
				t_cons.push(Consonant::from_vec(&phone_vec));
			}

		}
		sylls.push(Syll{leading_vowel: l_vowel, trailing_consonants: t_cons});

		Self{sylls: sylls, src: w.to_string()}
	}

	fn has_cons_end(&self) -> bool{
		self.sylls.last().unwrap().trailing_consonants.len() > 0
	}

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
				let slength2 = other.sylls[is].trailing_consonants.len();
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
		if self.has_cons_end() == other.has_cons_end(){
			dist += sett.same_cons_end;
		}

		let length_diff: f32 = ((other.sylls.len() - self.sylls.len()) as f32).abs();
		dist += sett.length_diff_fine * length_diff;
		dist
	}

	pub fn measure_distance(&self, other: &Self, gs: &GeneralSettings) -> f32{
		let mut dist = 0.0;
		let (first, second) = Self::get_sorted_by_sylls(self, other);

		dist += first.measure_misc(second, &gs.misc);
		//println!("Other: {}", dist);
		dist += first.measure_vowel_dist(second, &gs.stresses);
		//println!("vowel: {}", first.measure_vowel_dist(second, &gs.stresses));
		dist += first.measure_cons_dist(second, &gs.alliteration);
		//println!("cons: {}", first.measure_cons_dist(second, &gs.alliteration));
		dist += first.measure_struct_dist(second, &gs.consonant_structure);
		//println!("struct: {}", first.measure_struct_dist(second, &gs.consonant_structure));

		dist
	}
	fn into_iter(&self) -> WordConsIterator{
		WordConsIterator::new(self)
	}
}

fn word_to_unprocessed_vecs(w: &str, is_adj: bool) -> Vec<Vec<char>>{
	// returns vec of vecs, where each of them is like [letter, …signs… (many possible), v/c (vowel or consonant marker)]
	let w = transcript(w, is_adj);
	let mut res = vec![];
	let mut this_vec = vec![];
	let mut current_type: char = '?'; // unknown type

	for l in w.chars(){
		let new_current = {
			if Vowel::contains_char(&l){Some('v')}
			else if Consonant::contains_char(&l){Some('c')}
			else {None}
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

#[cfg(test)]
#[test]
fn create_word(){
	let res = Word::new("дряньяня", false);
	let res2 = Word::new("драчунья", false);
	let mut gs = GeneralSettings{
	 	misc: MiscSettings{same_cons_end: 0.0, length_diff_fine: 0.0},
		stresses: StressSettings{asympt: 0.0, bad_rythm: 0.0, k_not_strict_stress: 0.0, k_strict_stress: 0.0, weight: 0.0}, 
		consonant_structure: ConsonantStructureSettings{asympt: 0.0, pow: 0.0, weight: 0.0},
		alliteration: AlliterationSettings{asympt: 0.0, pow_coord_delta: 0.0, pow_syll_ending: 0.0, shift_coord: 0.0, shift_syll_ending: 0.0, weight: 0.0},
		meaning: MeaningSettings{weight: 0.0}};

	assert_eq!(res.measure_distance(&res2, &gs), 0.0);
	/*gs.stresses.asympt = 1.0;
	gs.stresses.bad_rythm = -10.0;
	gs.stresses.k_strict_stress = 5.0;
	gs.stresses.k_not_strict_stress = 2.0;
	gs.stresses.weight = 1.0;
	//assert_eq!(res.measure_distance(&res2, &gs), 0.125);
	//assert_eq!(res.measure_distance(&Word::new("драчу'нья", false), &gs), -1.25);*/
	let res = Word::new("мно'ю", false);
}
