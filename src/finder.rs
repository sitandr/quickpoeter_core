use std::cmp::Ordering;
use crate::reader::GeneralSettings;
use std::collections::HashMap;
use crate::translator_struct::Word;
use crate::reader::{VECTOR_DIM, };
use ordered_float::OrderedFloat;


#[derive(Eq, Debug)]
pub struct WordDistanceResult<'a>{
	pub dist: OrderedFloat<f32>,
	pub word_src: &'a str,
}

impl WordDistanceResult<'_>{
	pub fn new<'c, 'a, 'b>(to_find: &'a Word, measured: &'b Word, gc: &'c GeneralSettings) -> WordDistanceResult<'b>{
		let dist = OrderedFloat(to_find.measure_distance(measured, gc));

		WordDistanceResult{dist: dist, word_src: &measured.src}
	}
}

impl Ord for WordDistanceResult<'_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.dist.cmp(&other.dist)
    }
}

impl PartialOrd for WordDistanceResult<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for WordDistanceResult<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.dist == other.dist
    }
}

pub struct WordCollector{
	pub words: Vec<Vec<Word>>,
	pub parts_of_speech: Vec<String>,
	i2w: Vec<String>,
	meanings: Vec<[f32; VECTOR_DIM]>,
	pub gs: GeneralSettings
}

impl WordCollector{
	pub fn new(i2w: Vec<String>, zaliz: HashMap<String, String>, meanings: Vec<[f32; VECTOR_DIM]>, gs: GeneralSettings) -> WordCollector{
		let mut words: Vec<Vec<Word>> = vec![];
		let mut parts_of_speech: Vec<String> = vec![];

		for ind in 0..i2w.len(){
			let name = &i2w[ind];
			
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
				declension.push(Word::new(&base, is_adj, Some(meanings[ind])));
			}
			words.push(declension);
		}

		WordCollector{i2w: i2w, words: words, parts_of_speech: parts_of_speech, meanings: meanings, gs: gs}
	}

	pub fn find_best<'a, 'b, 'c>(&'a self, to_find: &'b Word, ignore: Vec<&'c str>, top_n: u32) -> Vec<WordDistanceResult<'a>>{
		use std::collections::BinaryHeap;
		use crate::reader::read_settings;

		let mut heap = BinaryHeap::new();
		let mut c: u32 = 0;
		for w in self.into_iter(ignore){
			c += 1;
			if c%1_000 == 0{
				println!("{}", c);
			}
			let res: WordDistanceResult<'a> = WordDistanceResult::new(&to_find, w, &read_settings());
			heap.push(res);
			if heap.len() > top_n as usize{
				heap.pop(); // pops the word with the greatest distance
			}
		}

		heap.into_sorted_vec()
	}


	pub fn load_default() -> Self{
		crate::reader::load_default_word_collector()
	}

	pub fn into_iter<'a, 'b>(&'a self, ignore: Vec<&'b str>) -> WordCollectIterator<'a, 'b>{
		WordCollectIterator{wc: self, count: 0,
		 count_form: 0, ignore_parts_of_speech: ignore}
	}
}

pub struct WordCollectIterator<'a, 'b>{
	wc: &'a WordCollector,
	count: usize,
	count_form: usize,
	ignore_parts_of_speech: Vec<&'b str>
}

impl<'a, 'b> Iterator for WordCollectIterator<'a, 'b>{
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

/*
#[cfg(test)]
#[test]
fn word_collect(){
	let wc = WordCollector::load_default();
	println!("Loaded");
	println!("{:?}", wc.find_best(&Word::new("слово", false, None), vec![], 50));
}*/