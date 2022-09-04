
use std::fmt::Formatter;
use std::fmt::Display;
use itertools::Itertools;

use crate::reader::ConsonantDistanceSettings;
use crate::reader::VowelDistanceSettings;
use crate::translator_ru::{Vowel, Consonant, transcript, symbol_id};
use crate::reader::{GeneralSettings, MiscSettings, StressSettings, ConsonantStructureSettings, AlliterationSettings};

macro_rules! unwrap_enum {
	($v:expr, $p: pat => $r: expr) => {
		match $v{
			$p => $r,
			_ => panic!("Bad enum inner for {}: {:?}", stringify!($p), $v)
		}
	};
}

#[derive(Debug, Copy, Clone)]
pub enum Accent{
	NoAccent,
	Primary,
	Secondary,
}

#[derive(Debug, Clone)]
enum Phone{
	Vowel(Vowel),
	Consonant(Consonant),
	None
}

pub trait Phonable{
	fn contains_char(c: &char) -> bool;
}

pub trait Voweable: Phonable{
	fn distance(&self, second: &Self, sett: &VowelDistanceSettings) -> f32;
	fn accent(&self) -> Accent;
	fn accent_dist(&self, second: &Self, sett: &StressSettings, index: usize) -> f32{ // needs stress_settings -> doesn't belong to Phonable
		type A = Accent;
		let k: f32 = match (self.accent(), second.accent()) {
			(A::NoAccent, A::NoAccent) => 1.0,
		    (A::NoAccent, A::Primary)|(A::Primary, A::NoAccent) => {return sett.bad_rythm},
		    (A::NoAccent, A::Secondary)|(A::Secondary, A::NoAccent)|(A::Secondary, A::Secondary) => sett.k_not_strict_stress,
		    (A::Primary, A::Primary)|(A::Primary, A::Secondary)|(A::Secondary, A::Primary) => {
				if sett.indexation{
					return 0.0;
				}
				return sett.k_strict_stress * self.distance(second, &sett.distance)
			}
		};
		k * self.distance(second, &sett.distance) / (index as f32 + sett.shift_syll_ending).powf(sett.pow_syll_ending)
	}
}

pub trait Consonantable: Phonable{
	fn distance(&self, second: &Self, sett: &ConsonantDistanceSettings) -> f32;
}

#[derive(Debug, Clone)]
pub struct Word{
	// unlike python version, the letter order stays the same
	phones: Vec<Phone>,
	vowel_count: usize, // many counting use number of syll as param
	pub src: String,
	/// true means it has only abstract vowels, so we can skip
	/// all cons metrics when measuring distance
	pub only_stress_structure: bool
}

impl Word{
	pub fn new(w: &str, is_adj: bool) -> Self {
		let src = w.replace("'", "").replace("`", "");
		let w = transcript(w, is_adj);
		let mut phones = vec![];
		let mut current: Phone = Phone::None; // we need to initialize somehow

		for l in w.chars(){
			// stores tha type of new letter
			let new_current = {
				if Vowel::contains_char(&l){Phone::Vowel(Vowel{letter: find_u8(l, Vowel::ALL.iter()),
					 											accent: Accent::NoAccent})}
				
				else if Consonant::contains_char(&l){Phone::Consonant(Consonant{letter: find_u8(l, Consonant::ALL.iter()),
																				 voiced: false, palatalized: false})}
				else {Phone::None} // some symbol
			};

			match new_current{
				Phone::None => {
					match current{
						Phone::Vowel(ref mut v) => {
							v.accent = match l{
								'\'' => Accent::Primary,
								'`' => Accent::Secondary,
								other => unreachable!("Bad vowel identifier {}", other)
							}
						},
						Phone::Consonant(ref mut c) => {
							match l{
								'*' => {c.voiced = true},
								'^' => {c.palatalized = true},
								other => unreachable!("Bad consonant identifier {}", other)
							}
						},
						Phone::None => panic!("Modificator {} at the word start", l)
					}
				}
				Phone::Consonant(_)|Phone::Vowel(_) => {
					match current{
						Phone::None => {}
						_ => {phones.push(current)}
					}
					current = new_current;
				},
			}
		}
		phones.push(current);
		Self{phones, src, only_stress_structure: false, vowel_count: 0}.count_vowels()
	}

	fn count_vowels(mut self) -> Self{
		self.vowel_count = self.vowels().count();
		self
	}

	#[allow(dead_code)]
	pub fn get_vowel_count(&self) -> usize{
		return self.vowel_count
	}

	pub fn get_phones_count(&self) -> usize{
		return self.phones.len()
	}

	/// constructs new only_stress_structure word
	pub fn new_abstract(w: &str) -> Self{
		let phones = w.chars().map(|l| match l{
			'+' => Vowel{letter: symbol_id!(+), accent: Accent::NoAccent},
			'!' => Vowel{letter: symbol_id!(!), accent: Accent::Primary},
			_ => unreachable!("Bad identifier, {}", l)
		}).map(|stress| Phone::Vowel(stress)).collect();
		Self{phones, src: w.to_string(), only_stress_structure: true, vowel_count: 0}.count_vowels()
	}

	fn has_cons_end(&self) -> bool{
		matches!(self.phones.last().unwrap(), Phone::Consonant(_))
	}

	/// use .rev() to get back order
	fn vowels(&self) -> impl DoubleEndedIterator<Item = &Vowel>
	{
		self.phones.iter().filter_map(|p| match p{
			Phone::Vowel(v) => Some(v),
			_ => None
		})
	}

	/// returns iterator over (start of syll, length of syll (may be 0))
	/// starting from the end
	/// VCCVC => (4, 1), (1, 2), (0, 0)
	/// CCVV => (4, 0), (3, 0), (0, 2)
	/// IMPORTANT: indexes might be out of bounds if block len == 0
	/// Not DoubleIterator — it will break the FnMut for filter_map
	fn splitted_consonants_rev(& self) -> impl Iterator<Item = (usize, usize)> + '_
	{
		let mut len = 0;
		let closure = move |(i, p)| -> Option<(usize, usize)> {
			let i: usize = i;
			match p{
				&Phone::Vowel(_) => {
					let r = Some((i.wrapping_add(1), len));
					len = 0;
					r},
				&Phone::Consonant(_) =>  {len += 1; None},
				_ => unreachable!()
			}
		};
		let res = self.phones.iter().enumerate().rev().chain(std::iter::once((usize::MAX, &Phone::Vowel(Vowel{letter: 0, accent: Accent::NoAccent})))).filter_map(closure);
		res
	}

	/// return (min, max) by len
	pub fn get_sorted_by_sylls<'a>(one: &'a Self, other: &'a Self) -> (&'a Self, &'a Self){
		if one.vowel_count > other.vowel_count{
			(other, one)
		}
		else{
			(one, other)
		}
	}

	pub fn measure_vowel_dist(&self, other: &Self, sett: &StressSettings) -> f32{
		let mut dist = 0.0;
		let first_iter = self.vowels().rev();
		let second_iterator = other.vowels().rev();
		
		for (i, (v1, v2)) in first_iter.zip(second_iterator).enumerate(){ // self is smaller
			// dbg!(v1, v2, i, v1.accent_dist(v2, sett) / (i as f32 + sett.shift_syll_ending).powf(sett.pow_syll_ending));
			dist += v1.accent_dist(v2, sett, i);
		}
		dist/(self.vowel_count as f32 + sett.asympt_shift).powf(sett.asympt)*sett.weight
	}

	pub fn measure_cons_dist(&self, other: &Self, sett: &AlliterationSettings) -> f32{
		let mut dist = 0.0;

		// syll_index is index from the word end
		for (syll_ind_1, (s_ind_1, len_1)) in other.splitted_consonants_rev().enumerate(){ // iterating over first sylls
			for cons_ind_1 in s_ind_1..s_ind_1 + len_1{ // over first letters
				let c1 = unwrap_enum!(&other.phones[cons_ind_1], Phone::Consonant(ref c) => c);

				let mut dist_min = f32::MAX; // let's consider only cases with any consonants in second word

				for (syll_ind_2, (s_ind_2, len_2)) in self.splitted_consonants_rev().enumerate(){ // second sylls
					let sum_syl_len = (len_1 + len_2) as f32;
					for cons_ind_2 in s_ind_2..s_ind_2 + len_2{ // second letter

						let d1 = (syll_ind_1) as f32 + (cons_ind_1 - s_ind_1) as f32 /sum_syl_len;
						let d2 = (syll_ind_2) as f32 + (cons_ind_2 - s_ind_2) as f32 /sum_syl_len;

						let mut k  = ((d1 - d2).abs() +  sett.shift_coord).powf(sett.pow_coord_delta);
						k /= (d1 + d2 + sett.shift_syll_ending).powf(sett.pow_syll_ending);
						
						let c2 = unwrap_enum!(&self.phones[cons_ind_2], Phone::Consonant(ref c) => c);
						
						let d = c1.distance(&c2, &sett.distance);
						dist_min = f32::min(d, dist_min);
						dist += d/k;
					}
				}

				dist += dist_min * sett.permutations;
			}
		}

		for (s_ind_1, len_1) in self.splitted_consonants_rev(){ // iterating over first sylls
			for cons_ind_1 in s_ind_1..s_ind_1 + len_1{ // over first letters
				let c1 = unwrap_enum!(&self.phones[cons_ind_1], Phone::Consonant(ref c) => c);

				let mut dist_min = f32::MAX; // let's consider only cases with any consonants in second word

				for (s_ind_2, len_2) in other.splitted_consonants_rev(){ // second sylls
					for cons_ind_2 in s_ind_2..s_ind_2 + len_2{ // second letter
						let c2 = unwrap_enum!(&other.phones[cons_ind_2], Phone::Consonant(ref c) => c);
						let d = c1.distance(&c2, &sett.distance);
						dist_min = f32::min(d, dist_min);
					}
				}
				dist += dist_min * sett.permutations;
			}
		}

		dist/(self.vowel_count as f32 + sett.asympt_shift).powf(sett.asympt)*sett.weight 
	}

	// TODO: ending is more important!
	// (as with vowels)
	pub fn measure_struct_dist(&self, other: &Self, sett: &ConsonantStructureSettings) -> f32{
		let mut dist = 0.0;
		
		for (i, ((_, l1), (_, l2))) in self.splitted_consonants_rev().zip(other.splitted_consonants_rev()).enumerate(){ // self is smaller
			dist += ((l1 as f32 - l2 as f32)).abs().powf(sett.pow) / (i as f32 + sett.shift_syll_ending).powf(sett.pow_syll_ending);
		}
		dist/(self.vowel_count as f32 + sett.asympt_shift).powf(sett.asympt)*sett.weight
	}

	pub fn measure_misc(&self, other: &Self, sett: &MiscSettings) -> f32{
		let mut dist = 0.0;
		if self.has_cons_end() != other.has_cons_end(){
			dist += sett.same_cons_end;
		}

		let length_diff: f32 = ((other.vowel_count - self.vowel_count) as f32).abs();
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

			if second.vowel_count != first.vowel_count{
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

	/// Returns position of primary stress and vec of positions of secondary
	/// **IMPORTANT!** Returns the number of *vowel* in letter notation (starting from 0).
	/// It can't return absolute char just because it knows only about the sounds
	/// (works correctly only with words with single primary and possible secondary stresses)
	#[allow(dead_code)] 
	pub fn get_stresses(&self) -> (usize, Option<usize>){
		let mut primary = usize::MAX;
		let mut secondary = None;
		for (ind, vowel) in self.vowels().enumerate(){
			match vowel.accent{
				Accent::Primary => primary = ind,
				Accent::Secondary => secondary = Some(ind),
				Accent::NoAccent => {}
			}
		}
		assert_ne!(primary, usize::MAX);
		(primary, secondary)
	}

	/// returns letter and position from the END of PRIMARY stress
	pub fn get_primary_stress(&self) -> (u8, usize){
		for (ind, vowel) in self.vowels().rev().enumerate(){
			match vowel.accent{
				Accent::Primary => return (vowel.letter, ind),
				_ => {}
			}
		}
		unreachable!()
	}

	/// returns all stresses in word (useful if many possible, e.g. user input)
	pub fn get_all_stresses(&self) -> Vec<(u8, usize)>{
		self.vowels().rev().enumerate().filter_map(|(ind, vowel)|
			match vowel.accent{
				Accent::Primary|Accent::Secondary => Some((vowel.letter, ind)),
				Accent::NoAccent => None
			}
		).collect_vec()
	}
}

pub fn find_u8<'a, T, I>(elem: T, mut array: I) -> u8
where I: Iterator<Item=&'a T>,
T: 'a + Eq
{
	array.position(|r| *r == elem).unwrap() as u8
}

impl Display for Word{

	fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
		write!(f, "{}", self.src)
	}
}

#[cfg(test)]
#[test]
fn measure_struct(){
	let gs = GeneralSettings::load_default();
	let w = Word::new("приве'т", false);
	let w1 = Word::new("уже'", false);
	let w2 = Word::new("мише'нь", false);
	dbg!(w.measure_struct_dist(&w1, &gs.consonant_structure));
	dbg!(w.measure_struct_dist(&w2, &gs.consonant_structure));
}

#[cfg(test)]
#[test]
fn check_stress(){
	assert_eq!(Word::new("ещё", false).get_stresses().0, 1);
	assert_eq!(Word::new("лома'ть", false).get_stresses().0, 1);
	assert_eq!(Word::new("ско'лько", false).get_stresses().0, 0);
}

#[cfg(test)]
#[test]
fn check_consonant_iterator(){
	let w = Word::new("ныро'д", false);
	let mut iter = w.splitted_consonants_rev();
	assert_eq!(iter.next(), Some((4, 1)));
	assert_eq!(iter.next(), Some((2, 1)));
	assert_eq!(iter.next(), Some((0, 1)));
	assert_eq!(iter.next(), None);
	let w = Word::new("узлы", false);
	let mut iter = w.splitted_consonants_rev();
	assert_eq!(iter.next(), Some((4, 0)));
	assert_eq!(iter.next(), Some((1, 2)));
	assert_eq!(iter.next(), Some((0, 0)));
	assert_eq!(iter.next(), None);
}