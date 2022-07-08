/// # Plan of optimization
/// 0. Structurize all stressed -> speed up up to ~100 times (loosing quality)
/// 1. Structurize the endings -> speed up up to ~20 times
/// 2. Multi-threading for finding -> 2-3 times
/// 3. Directly reducing the time, e.g. replace phf hash with compile-time matches 
/// 4. Replace String to &str in words (not sure will give a speed up)
/// 5. Count meaner only one time -> ~ + 10%


use std::fmt::Formatter;
use std::fmt::Debug;
use std::time::Instant;
use std::cmp::Ordering;
use crate::reader::GeneralSettings;
use std::collections::HashMap;
use crate::translator_struct::Word;
use crate::reader::{VECTOR_DIM, };
use ordered_float::NotNan;
use crate::meaner::MeanField;
use std::collections::BinaryHeap;


#[derive(Clone)]
pub struct WordDistanceResult<'a>{
	pub dist: NotNan<f32>,
	pub word: &'a Word,
}

impl WordDistanceResult<'_>{
	pub fn new<'c, 'a, 'b>(to_find: &'a Word, measured: &'b Word, gs: &'c GeneralSettings) -> WordDistanceResult<'b>{
		let dist = NotNan::new(to_find.measure_distance(measured, gs)).unwrap();
		WordDistanceResult{dist: dist, word: &measured}
	}
	pub fn from_forms<'c, 'a, 'b>(to_find: &'a Word, measured: &'b Vec<Word>, gs: &'c GeneralSettings, field: Option<&MeanField>) -> WordDistanceResult<'b>{
		let mut res = measured.iter().map(|w| WordDistanceResult::new(to_find, w, gs)).min().unwrap();

		if let Some(field) = field{
			if let Some(meaning) = res.word.meaning{
				res.dist += field.dist(meaning) * gs.meaning.weight;
			}
		}
		res
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

impl Eq for WordDistanceResult<'_>{}

impl Debug for WordDistanceResult<'_>{
	fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
		write!(f, "{}: {}", self.word.src, self.dist)
	}
}

pub struct WordCollector{
	pub words: Vec<Vec<Word>>,
	pub parts_of_speech: Vec<String>,
	pub meanings: Vec<[f32; VECTOR_DIM]>,
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

		WordCollector{words: words, parts_of_speech: parts_of_speech, meanings: meanings, gs: gs}
	}



	pub fn find_best<'a, 'b, 'c>(&'a self, to_find: &'b Word, ignore: Vec<&'c str>, top_n: u32, field: Option<&MeanField>) -> Vec<WordDistanceResult<'a>>{

		let mut heap = BinaryHeap::new();
		let mut max_d: NotNan<f32> = NotNan::new(f32::MAX).unwrap();
		let mut c: u32 = 0;

		let mut collected = false;

		for w_forms in self.into_iter(ignore){
			
			let res = WordDistanceResult::from_forms(to_find, w_forms, &self.gs, field);
			if !collected{
				c += 1;
				heap.push(res);
				if c >= top_n{
					collected = true;
					max_d = heap.peek().unwrap().dist;
				}
			}
			else{
				if max_d > res.dist{
					heap.pop(); // pops the word with the greatest distance!
					heap.push(res);
					max_d = heap.peek().unwrap().dist;
				}
				
			}
		}

		heap.into_sorted_vec()
	}


	pub fn load_default() -> Self{
		crate::reader::load_default_word_collector()
	}

	pub fn into_iter<'a, 'b>(&'a self, ignore: Vec<&'b str>) -> WordCollectIterator<'a, 'b>{
		WordCollectIterator{wc: self, count: 0,
		 ignore_parts_of_speech: ignore}
	}
}

pub struct WordCollectIterator<'a, 'b>{
	wc: &'a WordCollector,
	count: usize,
	ignore_parts_of_speech: Vec<&'b str>
}

impl<'a, 'b> Iterator for WordCollectIterator<'a, 'b>{
	type Item = &'a Vec<Word>;
	fn next(&mut self)-> Option<Self::Item> {
		if self.count >= self.wc.words.len(){
			return None;
		}

		let w_forms = &self.wc.words[self.count];
		if self.ignore_parts_of_speech.contains(&&&*self.wc.parts_of_speech[self.count]){
			self.count += 1;
			self.next()
		}
		else{
			self.count += 1;
			Some(w_forms)
		}
	}
}

#[cfg(test)]
#[test]
fn word_collect(){
	let current = Instant::now();
	let wc = WordCollector::load_default();
	println!("Loaded words in {:#?}", current.elapsed());
	let current = Instant::now();

	println!("{:?}", wc.find_best(&Word::new("глазу'нья", false, None), vec![], 50, Some(&MeanField::new(vec![wc.meanings[1525]]))));
	println!("Found words in {:#?} seconds", current.elapsed());
	/*println!("{:?}", wc.find_best(&Word::new("глазу'нья", false, Some(wc.meanings[1525])), vec![], 50, None));
	println!("{:?}", wc.find_best(&Word::new("глазу'нья", false, None), vec![], 50, None));*/

}