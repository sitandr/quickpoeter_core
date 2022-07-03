use crate::translator_ru::{Vowel, Consonant};



pub trait Phone{
	fn similarity(&self, second: &Self) -> f32;
}

#[derive(Debug)]
pub struct Syll{
	// to get simplier logic, sylls are defined as starting from vowel
	// -_a_nd-
	leading_vowel: Vowel,
	trailing_consonants: Consonant
}

pub struct Word{
	// unlike python version, the letter order stays the same
	pub syllables: Vec<Syll>
}


