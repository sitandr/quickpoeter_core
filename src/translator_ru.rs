use crate::translator_struct::*;
use phf::phf_map;

static KEYWORDS: phf::Map<&'static str, Phone> = phf_map! {
    "а" => Phone::Vowel(Vowel{letter: 'а', accent: 0}),
    
};

impl Word{
	fn new(from_s: &str){
		for c in from_s.chars(){

		}
	}
}
