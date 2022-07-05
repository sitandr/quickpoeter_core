// I shuld try using TAURI!

use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use serde::{Deserialize};
use serde_pickle::de::DeOptions;
use half::f16;
use std::time::{Instant};
use std::hash::Hash;

pub const VECTOR_DIM: usize = 150;

/* General settings */

#[derive(Deserialize, Debug)]
pub struct MiscSettings{
    pub same_cons_end: f32,
    pub length_diff_fine: f32
}

#[derive(Deserialize, Debug)]
pub struct StressSettings{
    pub k_not_strict_stress: f32,
    pub k_strict_stress: f32,
    pub bad_rythm: f32,
    pub asympt: f32,
    pub weight: f32
}

#[derive(Deserialize, Debug)]
pub struct ConsonantStructureSettings{
    pub pow: f32,
    pub weight: f32,
    pub asympt: f32,
}

#[derive(Deserialize, Debug)]
pub struct AlliterationSettings{
    pub shift_coord: f32,
    pub shift_syll_ending: f32,
    pub pow_coord_delta: f32,
    pub pow_syll_ending: f32,
    pub weight: f32,
    pub asympt: f32,
}

#[derive(Deserialize, Debug)]
pub struct MeaningSettings{
    pub weight: f32,
}

#[derive(Deserialize, Debug)]
pub struct GeneralSettings{
    pub misc: MiscSettings,
    pub stresses: StressSettings, 
    pub consonant_structure: ConsonantStructureSettings,
    pub alliteration: AlliterationSettings,
    pub meaning: MeaningSettings,
}

#[derive(Debug)]
pub struct RawData{
    pub index2word: Vec<String>,
    pub word2index: HashMap<String, u32>,
    pub min_zaliz: HashMap<String, String>, // based on http://jurta.org/ru/nlp/rus/zaliz
    pub vectors: Vec<[f32;VECTOR_DIM]>,
    pub special_info: HashMap<String, u32>
}

impl RawData{
    pub fn load_default() -> Self{
        let i2w: Vec<String> = pickle_read("res/r_index2word.pkl");
        let w2i = cloning_hash_from_list(i2w.clone());

        let mz = pickle_read("res/r_min_zaliz.pkl");
        let si = pickle_read("res/r_special_info.pkl");

        let vects = bin_read16("res/r_vectors_16.bc");

        Self {index2word: i2w, word2index: w2i,
              min_zaliz: mz, special_info: si, vectors: vects}

    }
}

use std::convert::TryInto;
use std::fmt::Debug;

fn vec2arr<T: Debug, const N: usize>(arr: Vec<Vec<T>>) -> Vec<[T; N]> {
    let mut new_arr = vec![];
    for elem in arr {
        new_arr.push(elem.try_into().expect("Wrong dim"));
    }
    new_arr
}

// this can read standart f32 data
#[allow(dead_code)]
fn bin_read(path: &str) -> Vec<[f32;VECTOR_DIM]>{
    let f = BufReader::new(File::open(path).unwrap());
    let data: Vec<Vec<f32>> = bincode::deserialize_from(f).unwrap();
    vec2arr(data)
}

fn vec16_to_vec32(v: Vec<f16>) -> Vec<f32>{
    v.into_iter().map(f16::to_f32).collect()
}

fn bin_read16(path: &str) -> Vec<[f32;VECTOR_DIM]>{
    let f = BufReader::new(File::open(path).unwrap());
    let data: Vec<Vec<f16>> = bincode::deserialize_from(f).unwrap();
    let data: Vec<Vec<f32>> = data.into_iter().map(vec16_to_vec32).collect();
    vec2arr(data)
}

pub fn pickle_read<'a, T>(path: &str) -> T
    where T: Deserialize<'a>
    {
    let file = File::open(path).expect(&("Error opening ".to_owned() + path));
    let reader = BufReader::new(file);
    let data: T = serde_pickle::from_reader(reader,
        DeOptions::new()).expect(&("Error reading: ".to_owned() + path));
    data
}

pub fn read_settings() -> GeneralSettings{
    let file = File::open("config/coefficients.yaml").expect("Error opening coeff file");
    let reader = BufReader::new(file);
    let gensettings: GeneralSettings = serde_yaml::from_reader(reader).expect("Error reading coeff file");
    gensettings
}

fn cloning_hash_from_list<T: Eq + Hash>(list: Vec<T>) -> HashMap<T, u32> {
    let mut hash = HashMap::new();
    for (ind, value) in list.into_iter().enumerate(){
        hash.insert(value, ind as u32);
    }
    hash
} 

#[ignore]
#[cfg(test)]
#[test]
fn test_loading(){
    println!("Loading data, this will take a while…");

    let current = Instant::now(); 
    let _i2w: Vec<String> = pickle_read("res/r_index2word.pkl");
    println!("Loaded words in {:#?} seconds", current.elapsed());

    let current = Instant::now(); 
    let _w2i_g: HashMap<String, u32> = cloning_hash_from_list(_i2w.clone());
    println!("Created word2index in {:#?} seconds", current.elapsed());

    // Generating value-ind costs 9 ms (6 ms without copying vec),

    let current = Instant::now(); 
    let _mz: HashMap<String, String> = pickle_read("res/r_min_zaliz.pkl");
    let _si: HashMap<String, u32> = pickle_read("res/r_special_info.pkl");
    println!("Loaded dict in {:#?} seconds", current.elapsed());

    let _current = Instant::now();
    let _vects: Vec<[f32;VECTOR_DIM]> = bin_read16("res/r_vectors_16.bc");
    // for some reason, in the test it displays two times much time than in main code
    println!("Loaded meaning in {:#?} seconds", current.elapsed());
}

use crate::translator_struct::Word;

#[cfg(test)]
#[test]
fn test_try_settings(){
    let gs = read_settings();
    let w1 = Word::new("сло'во", false);
    let w2 = Word::new("сла'ва", false);
    println!("слово-слава {}", w1.measure_distance(&w2, &gs));
    let w1 = Word::new("преда'тельство", false);
    let w2 = Word::new("рыда'тьустал", false);
    println!("преда'тельство-рыдатьустал {}", w1.measure_distance(&w2, &gs));
}

use crate::finder::WordCollector;
use ordered_float::OrderedFloat;

#[cfg(test)]
#[test]
fn test_try_loading(){
    let current = Instant::now(); 

    let rd = RawData::load_default();
    let gs = read_settings();
    println!("Loaded raw in {:#?} seconds", current.elapsed());

    let current = Instant::now(); 
    let wc = WordCollector::new(&rd.index2word, rd.min_zaliz);
    println!("Created words in {:#?} seconds", current.elapsed());
    // println!("0: {:#?} ps {}, 10: {:#?} {}", wc.words[0], wc.parts_of_speech[0], wc.words[10], wc.parts_of_speech[10]);

    let current = Instant::now();
    let w0 = Word::new("глазу'нья", false);
    let mut iter = wc.into_iter(vec![]);
    println!("{}, {}", wc.words[0][0], wc.words[1][0]);
    println!("{} {} {}", iter.next().unwrap(), iter.next().unwrap(), iter.next().unwrap());
    let w2 = iter.min_by_key(|w| OrderedFloat(w0.measure_distance(&w, &gs))).unwrap();
    println!("Min: {} — {}", w2, w0.measure_distance(&w2, &gs));
    println!("Глянула: {}", w0.measure_distance(&Word::new("гля'нула", false), &gs));
    println!("Found words in {:#?} seconds", current.elapsed());
}
