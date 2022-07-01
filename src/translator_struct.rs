

#[derive(Debug)]
pub struct Vowel{
	pub letter: char,
	pub accent: u8, // 0 if None, 2 if secondary, 1 if primary
}

#[derive(Debug)]
pub struct Consonant{
	pub letter: char,
	pub voiced: bool, // звонкая
	pub palatalized: bool // мягкая
}

pub enum Phone{
	Vowel(Vowel),
	Consonant(Consonant)
}

#[derive(Debug)]
struct Syll{
	// to get simplier logic, sylls are defined as starting from vowel
	// -_a_nd-
	leading_vowel: Vowel,
	trailing_consonants: Consonant
}

pub struct Word{
	// unlike python version, the letter order stays the same
	syllables: Vec<Syll>
}


