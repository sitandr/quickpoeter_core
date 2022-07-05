use std::collections::HashMap;
// use crate::reader::{RawData, VECTOR_DIM};
use crate::translator_struct::Word;


pub struct WordCollector{
	pub words: Vec<Vec<Word>>,
	pub parts_of_speech: Vec<String>
//	meanings: Vec<[f32; VECTOR_DIM]>
}

impl WordCollector{
	pub fn new(i2w: &Vec<String>, zaliz: HashMap<String, String>) -> WordCollector{
		let mut words: Vec<Vec<Word>> = vec![];
		let mut parts_of_speech: Vec<String> = vec![];

		for name in i2w{
			
			let mut declension: Vec<Word> = vec![];
			let data = &zaliz[name];
			let mut endings = data.split(';');

			let header: &str = endings.next().unwrap();


			let mut bases = header.split('+');
			let part_of_speech = bases.next().unwrap();
			parts_of_speech.push(part_of_speech.to_string());
			let is_adj: bool = match part_of_speech{
				"п"|"мс"|"мс-п"|"г"|"числ-п" => true,
				_ => false
			};

			let bases: Vec<&str> = bases.collect();
			// println!("{}, {:?}", name, bases);
			if bases[0].len() > 0{
				declension.push(Word::new(bases[0], is_adj));
			}
			

			for mut e in endings{
				let mut e2 = e.to_string();
				// println!("{}, {}", name, e);
				let mut base = match e.chars().next(){
					Some(c) if c.is_digit(10) => {
						e2.remove(0);
						e = &e2;
						bases[c.to_digit(10).unwrap() as usize]
					},
					_ => {bases[0]}
				}.to_string();

				base.push_str(e);
				// println!("{}", base);
				declension.push(Word::new(&base, is_adj));
			}
			words.push(declension);
		}

		WordCollector{words: words, parts_of_speech: parts_of_speech}
	}
}