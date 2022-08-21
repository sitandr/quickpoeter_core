use std::fmt::Display;
/// # Plan of optimization
/// 0. Structurize all stressed -> speed up up to ~100 times (loosing quality)
/// 1. Structurize the endings -> speed up up to ~20 times
/// 2. Multi-threading for finding -> 2-3 times
/// 3. Directly reducing the time, e.g. replace phf hash with compile-time matches 
/// 4. Replace String to &str in words (not sure will give a speed up)
/// 5. Count meaner only one time -> ~ + 10%


use std::fmt::Formatter;
use std::fmt::Debug;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::iter::zip;
use std::slice;
use std::str;
use crate::reader::GeneralSettings;
use std::collections::HashMap;
use crate::translator_struct::Word;
use crate::reader::VECTOR_DIM;
use ordered_float::NotNan;
use crate::meaner::MeanField;
use std::collections::BinaryHeap;


#[derive(Clone)]
pub struct WordDistanceResult<'a>{
	pub dist: NotNan<f32>,
	misc: f32,
	vowel: f32,
	cons: f32,
	structure: f32,
	meaning: f32,
	pub word: &'a Word,
}

impl<'collector> WordDistanceResult<'collector>{
	/// This function doesn't count meaning (but measures everything else). Use `from forms` to measure it or add "meaning fine" manually
	pub fn new<'c, 'a>(to_find: &'a Word, measured: &'collector Word, gs: &'c GeneralSettings) -> Self{

		let (misc, vowel, cons, structure) = to_find.measure_distance(measured, gs);

		let dist = NotNan::new(misc + vowel + cons + structure).unwrap();
		WordDistanceResult{dist: dist, word: &measured, misc:misc, vowel: vowel, cons: cons, structure: structure, meaning: 0.0}
	}

	pub fn from_forms(to_find: &'_ Word, wc: &'collector WordCollector, forms: &'_ WordForms, gs: &'_ GeneralSettings, field: Option<&MeanField>) -> Self{
		let mut res = forms.range().map(|i| WordDistanceResult::new(to_find, &wc.words[i], gs)).min().unwrap();
		res.add_meaning_fine(Some(forms.meaning), field, gs);
		res
	}

	/// (adds *field distance* from meaning to self.dist, if both are not None)
	pub fn add_meaning_fine(&mut self, meaning: Option<[f32; VECTOR_DIM]>, field: Option<&MeanField>, gs: &GeneralSettings){
		if let Some(field) = field{
			if let Some(meaning) = meaning{
				self.meaning += field.dist(meaning) * gs.meaning.weight;
				self.dist += self.meaning;
			}
		}
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

fn round3(n: f32) -> f32{
	f32::round(n*1_000.0)/1_000.0
}

impl Debug for WordDistanceResult<'_>{
	fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
		write!(f, "{} — (msc:{}; v:{}, c:{}; s: {}; m: {})", 
			self.word, round3(self.misc), round3(self.vowel), round3(self.cons), round3(self.structure), round3(self.meaning))
	}
}

struct UnsafeStrSaver(*const u8, usize);

impl UnsafeStrSaver{
	fn to_str(&self) -> &str{
		unsafe{
			let slice = slice::from_raw_parts(self.0, self.1);
			str::from_utf8_unchecked(slice) // correct if constructed using new
		}
	}

	fn to_bytes(&self) -> &[u8]{
		unsafe{
			slice::from_raw_parts(self.0, self.1)
		}
	}

	fn new(s: &str) -> Self{
		UnsafeStrSaver(s.as_ptr(), s.len())
	}
}

impl PartialEq for UnsafeStrSaver {
    fn eq(&self, other: &Self) -> bool {
        self.to_bytes() == other.to_bytes()
    }
}
impl Eq for UnsafeStrSaver {}

impl Hash for UnsafeStrSaver {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.to_bytes().hash(state);
    }
}

impl Display for UnsafeStrSaver{
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.to_str())
	}
}

unsafe impl Sync for UnsafeStrSaver {}

pub struct WordForms{
	pub start_index: usize,
	pub len: usize,
	pub meaning: [f32; VECTOR_DIM],
	pub speech_part: String
}

impl WordForms{
	pub fn range(&self) -> std::ops::Range<usize>{
		self.start_index..self.start_index + self.len
	}
}

pub struct WordCollector{
	pub words: Vec<Word>,
	pub word_form_groups: Vec<WordForms>,
	pub gs: GeneralSettings,
	string2index: HashMap<UnsafeStrSaver,
							(usize, usize)> // first is index of wordformgroup, second is absolute index of word in words
}

impl WordCollector{
	pub fn new(i2w: Vec<String>, mut zaliz: HashMap<String, String>, meanings: Vec<[f32; VECTOR_DIM]>, gs: GeneralSettings) -> WordCollector{
		let mut words: Vec<Word> = vec![];
		let mut word_form_groups: Vec<WordForms> = vec![];
		let mut string2index = HashMap::new();

		for (name, meaning) in zip(i2w, meanings){
			
			let data = zaliz.remove(&name).unwrap();
			let mut all_data = data.split('+');
			let speech_part: &str = all_data.next().unwrap();
			let mut bases:Vec<&str> = all_data.collect();
			let endings:Vec<&str> = bases.pop().unwrap().split(';').collect();

			let is_adj: bool = match speech_part{
				"п"|"мс"|"мс-п"|"г"|"числ-п" => true,
				_ => false
			};
			
			let speech_part = speech_part.to_string();

			let w_form_group = WordForms{start_index: words.len(), len: endings.len(), meaning, speech_part};
			word_form_groups.push(w_form_group);
			for mut e in endings.into_iter(){
				let mut e2 = e.to_string();
				let mut base = match e.chars().next(){
					Some(c) if c.is_digit(10) => {
						e2.remove(0);
						e = &e2;
						bases[c.to_digit(10).unwrap() as usize]
					},
					_ => {bases[0]}
				}.to_string();

				base.push_str(e);
				let w = Word::new(&base, is_adj);
				// w.get_stresses(); // checks if all have actual stress
				words.push(w);
			}
		}
		let mut wc = WordCollector{words, word_form_groups, gs, string2index: HashMap::new()};
		for (ind, wgroup) in wc.word_form_groups.iter().enumerate(){
			for word_index in wgroup.range(){
				let word_form = &wc.words[word_index];
				string2index.insert(UnsafeStrSaver::new(&*word_form.src), (ind, word_index));
			}
		}
		wc.string2index = string2index;
		wc
	}



	pub fn find_best<'a, 'b, 'c>(&'a self, to_find: &'b Word, ignore: Vec<&'c str>, top_n: u32, field: Option<&MeanField>) -> Vec<WordDistanceResult<'a>>{

		let mut heap = BinaryHeap::new();
		let mut max_d: NotNan<f32> = NotNan::new(f32::MAX).unwrap();
		let mut c: u32 = 0;

		let mut collected = false;

		for wform_group in self.word_form_groups.iter(){
			if ignore.contains(&&*wform_group.speech_part){
				continue;
			}
			let res = WordDistanceResult::from_forms(to_find, &self, wform_group, &self.gs, field);
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

	pub fn get_meaning(&self, not_stressed: &str) -> Option<[f32;VECTOR_DIM]>{
		self.string2index.get(&UnsafeStrSaver::new(not_stressed)).map(|(ind, _)| self.word_form_groups[*ind].meaning)
	}
	pub fn get_word(&self, not_stressed: &str) -> Option<&Word>{
		self.string2index.get(&UnsafeStrSaver::new(not_stressed)).map(|(_, w_ind)| &self.words[*w_ind])
	}
}

#[cfg(test)]
#[test]
fn memory_measure(){
	use std::mem;
	dbg!(mem::size_of::<UnsafeStrSaver>());
	dbg!(mem::size_of::<(usize,usize)>());
}

#[cfg(test)]
#[test]
fn word_collect(){
	use crate::reader::MeanStrFields;
	use std::time::Instant;
	let current = Instant::now();
	let wc = WordCollector::load_default();
	let mf = MeanStrFields::load_default();
	println!("Loaded words in {:#?}", current.elapsed());

	let current = Instant::now();
	

	let field = MeanField::from_str(&wc, &mf.str_fields["Love"]).expect("Can't find words");//&vec!["гиппопотам", "минотавр"]).unwrap();


	println!("{:?}", wc.find_best(&Word::new("глазу'нья", false), vec![], 50, Some(&field)));
	println!("Found words in {:#?} seconds", current.elapsed());
	println!("{:?}", wc.find_best(&Word::new("глазу'нья", false), vec![], 50, None));
	println!("{:?}", wc.find_best(&Word::new("глазу'нья", false), vec![], 50, None));

	let current = Instant::now();
	println!("{:?}", wc.get_word("ударение").unwrap().get_stresses());
	println!("Found stress in {:#?}", current.elapsed());

	println!("{:?}", wc.find_best(&Word::new("глазу'нья", false), vec![], 50, None));
	println!("Found words in {:#?} seconds", current.elapsed());

	//use std::{thread, time::Duration};
	//let mut wc = wc;

	/*
	println!("Sleeping (basic)");
	thread::sleep(Duration::from_millis(10_000));
	*/
}

#[ignore]
#[cfg(test)]
#[test]
fn profile_load(){
	use std::{thread, time::Duration};
	let mut wc = WordCollector::load_default();

	
	println!("Sleeping (basic)");
	thread::sleep(Duration::from_millis(10_000));
	
	wc.words = vec![];
	println!("Removed words");
	thread::sleep(Duration::from_millis(10_000));

	wc.string2index = HashMap::new();
	println!("Removed stringify");
	thread::sleep(Duration::from_millis(10_000));
}