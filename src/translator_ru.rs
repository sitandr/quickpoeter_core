use crate::translator_struct::*;
use phf::phf_map;
use crate::reader::StressSettings;
/*use lazy_static::lazy_static;
use regex::Regex;*/


const J_MARKERS: [char; 10] = ['а', 'о', 'э', 'и', 'ы', 'у', 'ь', 'ъ', '\'', '`']; // ' and ` mean there is a vowel before => marker

const J_VOWELS: [char; 4] = ['е', 'ё', 'ю', 'я'];
const J_MAP: phf::Map<char, char> = phf_map! {
	'ё' => 'о',
	'е' => 'э',
	'ю' => 'у',
	'я' => 'а'
};
// const J_VOWELS: Vec<&char> = J_MAP.keys().collect();
const SOFTABLE: [char; 10] = ['с', 'х', 'ф', 'к', 'т', 'п', 'р', 'л', 'н', 'м'];
const REMOVING_VOICE: [char; 7] = ['п', 'ф', 'к', 'т', 'ш', 'с', 'ш'];

//0        Т С   Ч
//1 РЛНМ П     Ш
//2        К Х  Ф
const ALLITERATION: phf::Map<char, (f32, f32)> = phf_map! {
	'р' => (0.0, 1.0),
	'л' => (0.5, 1.0),
	'н' => (1.0, 1.0),
	'м' => (1.5, 1.0),
	'п' => (3.0, 1.0),
	'т' => (4.0, 0.0),
	'к' => (4.0, 2.0),
	'с' => (5.0, 0.0),
	'х' => (5.0, 2.0),
	'ш' => (6.0, 1.0),
	'ч' => (8.0, 0.0),
	'ф' => (8.0, 2.0)
};
/*
 о 
  а э 
           
у     ы
        и
*/
const ASSONANSES: phf::Map<char, (i8, i8)> = phf_map! {
	'а' => (5, 5),
	'о' => (4, 6),
	'э' => (6, 5),
	'и' => (8, 2),
	'ы' => (7, 3),
	'у' => (3, 3)
};

#[derive(Debug, Copy, Clone)]
pub enum Accent{
	NoAccent,
	Primary,
	Secondary,
}

#[derive(Debug)]
pub struct Vowel{
	pub letter: char,
	pub accent: Accent, // 0 if None, 2 if secondary, 1 if primary\
}
impl Vowel{
	const ALL: [char; 6] = ['а', 'о', 'э', 'и', 'ы', 'у'];

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
	pub letter: char,
	pub voiced: bool, // звонкая
	pub palatalized: bool // мягкая
}
impl Consonant{
	const ALL: [char; 13] = ['р', 'л', 'н', 'м', 'п', 'т', 'к', 'с', 'х', 'ш', 'ч', 'ф', 'й'];
}


impl Phone for Vowel{
	fn distance(&self, other: &Self) -> f32{
		let (x1, y1) = ASSONANSES[&self.letter];
		let (x2, y2) = ASSONANSES[&other.letter];

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
		let v: Self = Self{letter: v[0], accent: accent};
		v
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
		if self.letter != 'й' && other.letter != 'й'{
			let (x1, y1) = ALLITERATION[&self.letter];
			let (x2, y2) = ALLITERATION[&other.letter];
			let mut k: f32 = if self.voiced == other.voiced {1.5} else {1.0};
			if self.palatalized == other.palatalized {k *= 1.5};

			((x1 - x2).powf(2.0) + (y1 - y2).powf(2.0))/65.0 * k
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
		let v: Self = Self{letter: v[0], voiced: voiced, palatalized: palatalized};
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
		w[0] = J_MAP[&w[0]];
		w.insert(0, 'й')
	}

	let mut offset:usize = 0;
	for i in 1..w.len(){ // starting with 1 because already checked the start
		let ind = i + offset;
		let val = w[ind];

		if J_VOWELS.contains(&val){
			w[ind] = J_MAP[&val]; // е —> э
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
}

use std::time::{Instant};

#[cfg(test)]
#[test]
fn testing(){

	let current = Instant::now(); 
	let _ = J_MARKERS.contains(&'а');
	let _ = J_MARKERS.contains(&'г');
    println!("Checked by vec in {:#?} seconds", current.elapsed());

    let current = Instant::now(); 
	transcript("кроманьонец", false);
	transcript("Енёня`яя", false);
    println!("Transcripted	 in {:#?} seconds", current.elapsed());
}