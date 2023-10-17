/*
Rust implementation of advanced ryhmes finder
Copyright (C) 2022  Andrej Sitnikov (sitandr, andr-sitnikov@mail.ru)

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.


Module that imports dictionary and config files
*/

use crate::finder::WordCollector;
use half::f16;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_pickle::de::DeOptions;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

pub const VECTOR_DIM: usize = 150;

/* General settings */

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MiscSettings {
    pub same_cons_end: f32,
    pub length_diff_fine: f32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ConsonantDistanceSettings {
    pub map: [(f32, f32); 12],
    pub pow: f32,
    pub denominator: f32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct VowelDistanceSettings {
    pub map: [(f32, f32); 6],
    pub pow: f32,
    pub denominator: f32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct StressSettings {
    pub k_not_strict_stress: f32,
    pub k_strict_stress: f32,
    pub bad_rythm: f32,
    pub asympt: f32,
    pub weight: f32,
    pub shift_syll_ending: f32,
    pub pow_syll_ending: f32,
    pub asympt_shift: f32,
    pub indexation: bool,
    pub distance: VowelDistanceSettings,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AlliterationSettings {
    pub shift_coord: f32,
    pub shift_syll_ending: f32,
    pub pow_coord_delta: f32,
    pub pow_syll_ending: f32,
    pub weight: f32,
    pub asympt: f32,
    pub asympt_shift: f32,
    pub permutations: f32,
    pub distance: ConsonantDistanceSettings,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ConsonantStructureSettings {
    pub pow: f32,
    pub weight: f32,
    pub asympt: f32,
    pub shift_syll_ending: f32,
    pub pow_syll_ending: f32,
    pub asympt_shift: f32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MeaningSettings {
    pub pow: f32,
    pub single_pow: f32,
    pub single_weight: f32,
    pub weight: f32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PopularitySettings {
    pub weight: f32,
    pub pow: f32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct UnsymmetricalSettings {
    pub optimal_length: f32,
    pub less_w: f32,
    pub less_pow: f32,
    pub more_w: f32,
    pub more_pow: f32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SamePartSpeechSettings {
    pub verb: f32,
    pub noun: f32,
    pub adj: f32,
    pub adv: f32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GeneralSettings {
    pub misc: MiscSettings,
    pub stresses: StressSettings,
    pub consonant_structure: ConsonantStructureSettings,
    pub alliteration: AlliterationSettings,
    pub meaning: MeaningSettings,
    pub popularity: PopularitySettings,
    pub unsymmetrical: UnsymmetricalSettings,
    pub same_speech_part: SamePartSpeechSettings,
}

macro_rules! construct_path {
    ($base: expr, $($dir: expr)*, $file: expr) => {
        &{
            let mut b = $base.clone();
            $(b.push($dir);)*
            b.push($file);
            b
        }
    };
}

impl GeneralSettings {
    pub fn load_default(dir: &PathBuf) -> GeneralSettings {
        yaml_read(construct_path!(dir, "config", "coefficients.yaml"))
            .expect("Error reading default settings")
    }
}

#[derive(Deserialize)]
pub struct MeanStrThemes {
    pub str_themes: HashMap<String, Vec<String>>,
}

impl MeanStrThemes {
    pub fn load_default(dir: &PathBuf) -> MeanStrThemes {
        MeanStrThemes {
            str_themes: yaml_read(construct_path!(dir, "config", "themes.yaml"))
                .expect("Error reading themes"),
        }
    }
}

pub fn load_default_word_collector(dir: &PathBuf) -> WordCollector {
    let i2w: Vec<String> = pickle_read(construct_path!(dir, "res", "r_index2word.pkl"));

    let mz: HashMap<String, String> = pickle_read(construct_path!(dir, "res", "r_min_zaliz.pkl"));

    let vects = bin_read16(construct_path!(dir, "res", "r_vectors_16.bc"));

    WordCollector::new(i2w, mz, vects)
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
fn bin_read(path: &PathBuf) -> Vec<[f32; VECTOR_DIM]> {
    let f = BufReader::new(File::open(path).unwrap());
    let data: Vec<Vec<f32>> = bincode::deserialize_from(f).unwrap();
    vec2arr(data)
}

fn vec16_to_vec32(v: Vec<f16>) -> Vec<f32> {
    v.into_iter().map(f16::to_f32).collect()
}

fn bin_read16(path: &PathBuf) -> Vec<[f32; VECTOR_DIM]> {
    let f = BufReader::new(File::open(path).unwrap());
    let data: Vec<Vec<f16>> = bincode::deserialize_from(f).unwrap();
    let data: Vec<Vec<f32>> = data.into_iter().map(vec16_to_vec32).collect();
    vec2arr(data)
}

pub fn pickle_read<'a, T>(path: &PathBuf) -> T
where
    T: Deserialize<'a>,
{
    let file = File::open(path).expect(&*format!("Error opening {:?}", path));
    let reader = BufReader::new(file);
    let data: T = serde_pickle::from_reader(reader, DeOptions::new())
        .expect(&*format!("Error reading {:?}", path));
    data
}

pub fn yaml_read<T>(path: &PathBuf) -> Result<T, String>
where
    T: DeserializeOwned,
{
    let file = File::open(path).map_err(|err| err.to_string())?;
    let reader = BufReader::new(file);
    Ok(serde_yaml::from_reader(reader).map_err(|err| err.to_string())?)
}

#[ignore]
#[cfg(test)]
#[test]
fn test_loading() {
    use std::time::Instant;
    println!("Loading data, this will take a while…");

    let current = Instant::now();
    let _i2w: Vec<String> = pickle_read(&PathBuf::from("res/r_index2word.pkl"));
    println!("Loaded words in {:#?} seconds", current.elapsed());

    /*let current = Instant::now();
    let _w2i_g: HashMap<String, u32> = cloning_hash_from_list(_i2w.clone());
    println!("Created word2index in {:#?} seconds", current.elapsed());*/

    // Generating value-ind costs 9 ms (6 ms without copying vec),

    let current = Instant::now();
    let _mz: HashMap<String, String> = pickle_read(&PathBuf::from("res/r_min_zaliz.pkl"));
    // let _si: HashMap<String, u32> = pickle_read("res/r_special_info.pkl");
    println!("Loaded dict in {:#?} seconds", current.elapsed());

    let _current = Instant::now();
    let _vects: Vec<[f32; VECTOR_DIM]> = bin_read16(&PathBuf::from("res/r_vectors_16.bc"));
    // for some reason, in the test it displays two times much time than in main code
    println!("Loaded meaning in {:#?} seconds", current.elapsed());
}

#[cfg(test)]
#[test]
fn test_try_settings() {
    use crate::translator_struct::Word;
    println!(
        "{:?}",
        MeanStrThemes::load_default(&PathBuf::from(".")).str_themes["Art"]
    );
    let gs = GeneralSettings::load_default(&PathBuf::from("."));
    let w1 = Word::new("сло'во", false);
    let w2 = Word::new("сла'ва", false);
    println!("слово-слава {:?}", w1.measure_distance(&w2, &gs));
    let w1 = Word::new("преда'тельство", false);
    let w2 = Word::new("рыда'тьустал", false);
    println!(
        "преда'тельство-рыдатьустал {:?}",
        w1.measure_distance(&w2, &gs)
    );
}
