use crate::translator_ru::{Vowel, Consonant, transcript};



pub trait Phone{
	fn similarity(&self, second: &Self) -> f32;
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
	pub syllables: Vec<Syll>
}

impl Word{
	fn new(w: &str, is_adj: bool) -> Self {
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

		Self{syllables: sylls}
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

#[cfg(test)]
#[test]
fn create_word(){
	let res = Word::new("дряньюня", false);
	println!("{:#?}", res);
}
