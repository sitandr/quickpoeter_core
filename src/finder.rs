use std::collections::HashMap;
use crate::translator_struct::Word;


#[derive(Debug)]
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
			let mut all_data = data.split('+');
			let part_of_speech: &str = all_data.next().unwrap();
			let mut bases:Vec<&str> = all_data.collect();
			let endings:Vec<&str> = bases.pop().unwrap().split(';').collect();


			
			parts_of_speech.push(part_of_speech.to_string());
			let is_adj: bool = match part_of_speech{
				"п"|"мс"|"мс-п"|"г"|"числ-п" => true,
				_ => false
			};

			// println!("{}, {:?}", name, bases);
			/*if bases[0].len() > 0{
				declension.push(Word::new(bases[0], is_adj));
			}*/
			

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
	pub fn into_iter<'a>(&'a self, ignore: Vec<&'a str>) -> WordCollectIterator{
		WordCollectIterator{wc: self, count: 0,
		 count_form: 0, ignore_parts_of_speech: ignore}
	}
}

pub struct WordCollectIterator<'a>{
	wc: &'a WordCollector,
	count: usize,
	count_form: usize,
	ignore_parts_of_speech: Vec<&'a str>
}

impl<'a> Iterator for WordCollectIterator<'a>{
	type Item = &'a Word;
	fn next(&mut self)-> Option<Self::Item> {
		
		let w_forms = &self.wc.words[self.count];
		if self.count_form >= w_forms.len(){
			self.count_form = 0;
			self.count += 1;
			if self.count >= self.wc.words.len(){
				return None;
			}
			else{
				return self.next();
			}
		}
		else if self.ignore_parts_of_speech.contains(&&&*self.wc.parts_of_speech[self.count]){
			self.count_form += 1;
			self.next()
		}
		else{
			self.count_form += 1;
			Some(&w_forms[self.count_form - 1])
		}
	}
}
