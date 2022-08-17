use crate::translator_struct::*;
use crate::reader::StressSettings;
/*use lazy_static::lazy_static;
use regex::Regex;*/

const J_MARKERS: [char; 10] = ['а', 'о', 'э', 'и', 'ы', 'у', 'ь', 'ъ', '\'', '`']; // ' and ` mean there is a vowel before => marker

pub const J_VOWELS: [char; 4] = ['е', 'ё', 'ю', 'я'];

macro_rules! J_MAP {
	($x:expr) => {
		match $x {
			'ё' => 'о',
			'е' => 'э',
			'ю' => 'у',
			'я' => 'а',
			_ => unreachable!()
		}
	};
}

// const J_VOWELS: Vec<&char> = J_MAP.keys().collect();
const SOFTABLE: [char; 10] = ['с', 'х', 'ф', 'к', 'т', 'п', 'р', 'л', 'н', 'м'];
const REMOVING_VOICE: [char; 6] = ['п', 'ф', 'к', 'т', 'ш', 'с'];

//0        Т С   Ч
//1 РЛНМ П     Ш
//2        К Х  Ф
const ALLITERATION: [(f32, f32); 12] = [
	(0.0, 1.0), // р
	(0.5, 1.0), // л
	(1.0, 1.0), // н
	(1.5, 1.0), // м
	(3.0, 1.0), // п
	(4.0, 0.0), // т
	(4.0, 2.0), // к
	(5.0, 0.0), // с
	(5.0, 2.0), // х
	(6.0, 1.0), // ш
	(8.0, 0.0), // ч
	(8.0, 2.0)  // ф
];
/*
 о 
  а э 
           
у     ы
        и
*/
const ASSONANSES: [(i32, i32); 6] = [
	(5, 5), // а
	(4, 6), // о
	(6, 5), // э
	(8, 2), // и
	(7, 3), // ы
	(3, 3) // у
];

macro_rules! symbol_id {
	(!) => (6);
	(+) => (7);
	(й) => (12);
}

pub(crate) use symbol_id;

/* 
// Maybe one day I will fix it to 
macro_rules! range_match {
	($x:expr, $($letter:ident),+) => {
		{
			enum Counter{
				$($letter,)+
			}
			match $x{
				$(stringify!($letter) => (Counter::$letter as u32),)+
				_ => unreachable!()
			}
		}
	};
}
*/

#[derive(Debug)]
pub struct Vowel{
	pub letter: u8,
	pub accent: Accent, // 0 if None, 2 if secondary, 1 if primary\
}
impl Vowel{
	pub const ALL: [char; 8] = ['а', 'о', 'э', 'и', 'ы', 'у', '!', '+'];

	// needs stress_settings -> doesn't belong to Phone
	pub fn accent_distance(&self, other: &Self, sett: &StressSettings) -> f32{
		type A = Accent;
		let k: f32 = match (self.accent, other.accent) {
			(A::NoAccent, A::NoAccent) => 1.0,
		    (A::NoAccent, A::Primary)|(A::Primary, A::NoAccent) => {return sett.bad_rythm},
		    (A::NoAccent, A::Secondary)|(A::Secondary, A::NoAccent)|(A::Secondary, A::Secondary) => sett.k_not_strict_stress,
		    (A::Primary, A::Primary)|(A::Primary, A::Secondary)|(A::Secondary, A::Primary) => sett.k_strict_stress
		};
		k * self.distance(other)
	}
}

#[derive(Debug)]
pub struct Consonant{
	pub letter: u8,
	pub voiced: bool, // звонкая
	pub palatalized: bool // мягкая
}
impl Consonant{
	/// order is extremely important
	const ALL: [char; 13] = ['р', 'л', 'н', 'м', 'п', 'т', 'к', 'с', 'х', 'ш', 'ч', 'ф', 'й'];
}

fn find_u8<'a, T, I>(elem: T, mut array: I) -> u8
where I: Iterator<Item=&'a T>,
T: 'a + Eq
{
	array.position(|r| *r == elem).unwrap() as u8
}

impl Phone for Vowel{
	fn distance(&self, other: &Self) -> f32{
		if self.letter == symbol_id!(+) || self.letter == symbol_id!(!) || other.letter == symbol_id!(+) || other.letter == symbol_id!(!){
			return 1.0;
		}
		let (x1, y1) = ASSONANSES[self.letter as usize];
		let (x2, y2) = ASSONANSES[other.letter as usize];

		let res: f32 = (((x1 - x2).pow(2) + (y1 - y2).pow(2)) as f32)/26.0;
		res
	}


	fn from_vec(v: &Vec<char>) -> Self{
		assert!(v.len() <= 2);// !!! IMPORTANT: FIXING DICT
		let accent = {
			if v.len() > 1{
				match v[1]{
				'\'' => Accent::Primary,
				'`' => Accent::Secondary,
				other => unreachable!("Bad identifier {}", other)
				}
			}
			else{
				Accent::NoAccent
			}
		};
		let mut letter = v[0];

		if letter == 'о' && v.len() == 1{ // безударная "о" становится "а"
			letter = 'а';
		}

		Self{letter: find_u8(letter, Self::ALL.iter()), accent: accent}
	}
	fn contains_char(c: &char) -> bool{
		Self::ALL.contains(c)
	}
}

impl Phone for Consonant{
	fn distance(&self, other: &Self) -> f32{
		if self.letter == other.letter{
			return 0.0;
		}
		if self.letter != symbol_id!(й) && other.letter != symbol_id!(й){
			let (x1, y1) = ALLITERATION[self.letter as usize];
			let (x2, y2) = ALLITERATION[other.letter as usize];
			let mut d: f32 = 0.0;
			if self.voiced == other.voiced {d += 0.5}
			if self.palatalized == other.palatalized {d += 0.5};

			((x1 - x2).powf(2.0) + (y1 - y2).powf(2.0) + d)/66.0
		}
		else{
			// й + …? — already checked they are not equal
			1.0 
		}
	}

	fn from_vec(v: &Vec<char>) -> Self{
		assert!(v.len() <= 3);
		let mut voiced = false;
		let mut palatalized = false;

		for i in 1..v.len(){
			match v[i]{
				'*' => {voiced = true},
				'^' => {palatalized = true},
				other => unreachable!("Bad identifier {}", other)
			}
		}
		let v: Self = Self{letter: find_u8(v[0], Self::ALL.iter()), voiced: voiced, palatalized: palatalized};
		v
	}
	fn contains_char(c: &char) -> bool{
		Self::ALL.contains(c)
	}
}


pub fn transcript(w: &str, is_adj: bool) -> String{
	// returns a postfix transcript like "к*ара'ш"
	let mut w: Vec<char> = w.to_lowercase().chars().collect();
	if is_adj{
    	replace_g_in_adj(&mut w);
    }
	j_replace(&mut w);
	i_soften(&mut w);
    letter_replace(&mut w);
    remove_voice(&mut w);

    w.into_iter().collect()
}

fn j_replace(w: &mut Vec<char>){
	if J_VOWELS.contains(&w[0]){ // енот — "й" в начале слова
		if w[0] == 'ё'{
			w.insert(1, '\'');
		}
		w[0] = J_MAP!(&w[0]);
		w.insert(0, 'й')
	}

	let mut offset:usize = 0;
	for i in 1..w.len(){ // starting with 1 because already checked the start
		let ind = i + offset;
		let val = w[ind];

		if J_VOWELS.contains(&val){
			w[ind] = J_MAP!(&val); // е —> э
			if val == 'ё'{
				w.insert(ind + 1, '\'');
				offset += 1;
			}
			
			if J_MARKERS.contains(&w[ind - 1]){
				w.insert(ind, 'й')
			}
			else{
				w.insert(ind, '^')
			}
			offset += 1; // shift indexes
		}
		else if val == 'о' && w[i - 1] == 'ь'{ // бульон
			w.insert(ind, 'й');
			offset += 1;
		}
	}
}

fn i_soften(w: &mut Vec<char>){
	// "и" is the only non-j vowel that softens those before 
	let mut offset:usize = 0;
	for i in 1..w.len(){ // starting with 1 because can't soften the first letter
		let ind = i + offset;
		let val = &w[ind];

		if *val == 'и' && SOFTABLE.contains(&w[i - 1]){
			w.insert(i, '^');
			offset += 1; // shift indexes
		}
	}
}

fn letter_replace(w: &mut Vec<char>){

	let mut offset:usize = 0;
	for i in 0..w.len(){ 
		let ind = i.wrapping_add(offset);
		let val = &w[ind];

		let replacement = match val {
			'б' => Some(vec!['п', '*']),
			'в' => Some(vec!['ф', '*']),
			'г' => Some(vec!['к', '*']),
			'д' => Some(vec!['т', '*']),
			'ж' => Some(vec!['ш', '*']),
			'з' => Some(vec!['с', '*']),
			'ц' => Some(vec!['т', 'с']),
			'щ' => Some(vec!['ш', '^']),
			'ь' => Some(vec!['^']),
			'ъ' => Some(vec![]),
			_ => None
		};

		if let Some(val) = replacement{
			offset = offset.wrapping_add(val.len().wrapping_sub(1));
			w.splice(ind..ind+1, val.into_iter());
		}
	}
}

fn check_previous_voice(w: &mut Vec<char>, pos: usize) -> Option<usize>{
	if pos <= 0{
		return None;
	}
	if w[pos - 1] == '*'{
		return Some(pos - 1)
	}
	else if pos >= 2 && w[pos - 1] == '^' && w[pos - 2] == '*'{
		return Some(pos - 2)
	}
	None
}

fn is_voiced(w: &mut Vec<char>, pos: usize) -> bool{
	if pos >= w.len() - 2{
		return false;
	}
	if w[pos + 1] == '*'{
		return true;
	}
	else if pos <= w.len() - 3 && w[pos + 1] == '^' && w[pos + 2] == '*'{
		return true;
	}
	false
}

fn remove_voice(w: &mut Vec<char>){
	if let Some(pos) = check_previous_voice(w, w.len()){
		w.remove(pos);
	}

	let mut offset = 0;
	for i in 1..w.len(){ // can't remove voice from nothing
		let ind:usize = i.wrapping_add(offset);
		let val = &w[ind];

		if REMOVING_VOICE.contains(val) && (!is_voiced(w, ind)){
			if let Some(pos) = check_previous_voice(w, ind){
				w.remove(pos);
				offset = offset.wrapping_sub(1);
			}
		}
	}
}

fn replace_g_in_adj(w: &mut Vec<char>){
	if w.len() < 3{
		return;
	}
	if w[w.len()-3..w.len()] == ['е', 'г', 'о']{
		w.splice(w.len()-3..w.len(), ['е', 'в', 'о'].into_iter());
	}
	else if w[w.len()-3..w.len()] == ['о', 'г', 'о']{
		w.splice(w.len()-3..w.len(), ['о', 'в', 'о'].into_iter());
	}
}


#[cfg(test)]
#[test]
fn j_replace_check(){
	assert_eq!(transcript("а'", false), "а'");
	assert_eq!(transcript("Я", false), "йа");
	assert_eq!(transcript("Митя Ляпин", false), "м^ит^а л^апин");
	assert_eq!(transcript("Митя Льяпин", false), "м^ит^а л^йапин");
	assert_eq!(transcript("Енёня`яя", false), "йэн^о'н^а`йайа");
	assert_eq!(transcript("миньо'н", false), "м^ин^йо'н");
	assert_eq!(transcript("бабузжка", false), "п*ап*ус*шка");
	assert_eq!(transcript("гроб", false), "к*роп");
	assert_eq!(transcript("дождь", false), "т*ошт^");
	assert_eq!(transcript("его", true), "йэф*о");
	assert_eq!(transcript("кроманьонец", false), "кроман^йон^этс");
	assert_eq!(transcript("Ёжик", false), "йо'ш*ик");
}


#[cfg(test)]
#[test]
fn testing(){
	use std::time::{Instant};

	let current = Instant::now(); 
	let _ = J_MARKERS.contains(&'а');
	let _ = J_MARKERS.contains(&'г');
    println!("Checked by vec in {:#?} seconds", current.elapsed());

    let current = Instant::now(); 
	transcript("кроманьонец", false);
	transcript("Енёня`яя", false);
    println!("Transcripted	 in {:#?} seconds", current.elapsed());

	use std::mem;
	dbg!(mem::size_of::<Vowel>());
	dbg!(mem::size_of::<Consonant>());
}

#[cfg(test)]
#[test]
fn map_or_match_speed(){
	use std::time::{Instant};
	let current = Instant::now(); 
	let r = 'ш';
	let r2 = match r{
		'р' => 0,
		'л' => 1,
		'н' => 2,
		'м' => 3,
		'п' => 4,
		'т' => 5,
		'к' => 6,
		'с' => 7,
		'х' => 8,
		'ш' => 9,
		'ч' => 10,
		'ф' => 11,
		_ => 100
	};
	println!("Matched in {:#?} seconds", current.elapsed());
	println!("{:?}", r2);

	/*
	let current = Instant::now();
	let r2 = range_match!(&*r.to_string(), а, б, в, ш, г, е);
	println!("Macro in {:#?} seconds", current.elapsed());
	*/

	let current = Instant::now();
	let r2 = find_u8(r, Consonant::ALL.iter());
	println!("Found in {:#?} seconds", current.elapsed());



	println!("{:?}", r2);
}