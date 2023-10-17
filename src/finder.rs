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


Module that is responsible for keeping and finding the best words
*/

use std::collections::HashSet;
use std::fmt::Display;
use std::path::PathBuf;

use crate::meaner::MeanTheme;
use crate::reader::GeneralSettings;
use crate::reader::MeaningSettings;
use crate::reader::PopularitySettings;
use crate::reader::SamePartSpeechSettings;
use crate::reader::UnsymmetricalSettings;
use crate::reader::VECTOR_DIM;
use crate::translator_ru::symbol_id;
use crate::translator_struct::Word;
use ordered_float::NotNan;
use serde::ser::SerializeStruct;
use serde::Serialize;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::hash::{Hash, Hasher};
use std::iter::zip;
use std::slice;
use std::str;

/// general info like what and how to find
pub struct FindingInfo<'collector, 'finding> {
    pub wc: &'collector WordCollector,
    pub to_find: &'finding Word,
    pub part_of_speech: Option<&'finding str>,
    pub gs: &'finding GeneralSettings,
    pub theme: Option<&'finding MeanTheme>,
    // all these refs need to live only through finding time
    // except collector; collector should live through all the time distance result exists
}

impl<'collector: 'finding, 'finding> FindingInfo<'collector, 'finding> {
    pub fn new(
        wc: &'collector WordCollector,
        to_find: &'finding Word,
        gs: &'finding GeneralSettings,
        theme: Option<&'finding MeanTheme>,
    ) -> Self {
        FindingInfo {
            wc,
            to_find,
            part_of_speech: wc.get_speech_part(&to_find.src),
            gs,
            theme,
        }
    }
}

#[derive(Clone)]
pub struct WordDistanceResult<'a> {
    pub dist: NotNan<f32>,
    misc: f32,
    vowel: f32,
    cons: f32,
    structure: f32,
    meaning: f32,
    popularity: f32,
    unsymmetrical: f32,
    same_part: f32,
    pub word: &'a Word,
}

impl<'collector> WordDistanceResult<'collector> {
    /// This function doesn't count meaning and other dictionary-dependent things
    /// (but measures everything "clean"). Use `from forms` to measure it or add "meaning fine" manually
    /// P.S. `unsymmetrical` is also not measured there as it is not "clean" thing, it doesn't affect real dist
    pub fn new<'c, 'a>(
        to_find: &'a Word,
        measured: &'collector Word,
        gs: &'c GeneralSettings,
    ) -> Self {
        let (misc, vowel, cons, structure) = to_find.measure_distance(measured, gs);

        let dist = NotNan::new(misc + vowel + cons + structure).unwrap();
        WordDistanceResult {
            dist,
            word: &measured,
            misc,
            vowel,
            cons,
            structure,
            meaning: 0.0,
            popularity: 0.0,
            unsymmetrical: 0.0,
            same_part: 0.0,
        }
    }

    pub fn from_forms(forms_index: usize, info: &FindingInfo<'collector, '_>) -> Self {
        let forms = &info.wc.word_form_groups[forms_index];
        let mut res = forms
            .range()
            .map(|i| WordDistanceResult::new(info.to_find, &info.wc.words[i], info.gs))
            .min()
            .unwrap();

        res.add_form_dists(info, forms_index);
        res
    }

    /// creates distance using only words that are presented in allowed_indexes
    /// returns None if no words left
    pub fn from_froms_with_filter(
        forms_index: usize,
        info: &FindingInfo<'collector, '_>,
        allowed_word_indexes: &HashSet<usize>,
    ) -> Option<Self> {
        let forms = &info.wc.word_form_groups[forms_index];

        let mut res = forms
            .range()
            .filter_map(|i| {
                if allowed_word_indexes.contains(&i) {
                    Some(WordDistanceResult::new(
                        info.to_find,
                        &info.wc.words[i],
                        info.gs,
                    ))
                } else {
                    None
                }
            })
            .min()?;

        res.add_form_dists(info, forms_index);
        Some(res)
    }

    /// adding distances that need word forms object to be known
    pub fn add_form_dists(&mut self, info: &FindingInfo, forms_index: usize) {
        let forms = &info.wc.word_form_groups[forms_index];
        self.add_meaning_dist(Some(forms.meaning), info.theme, &info.gs.meaning);
        self.add_popularity_dist(forms_index, &info.gs.popularity);
        self.add_unsymmetrical_dist(&info.gs.unsymmetrical);
        self.add_speech_part_dist(
            info.part_of_speech,
            &*forms.speech_part,
            &info.gs.same_speech_part,
        );
    }

    /// (adds *theme distance* from meaning to self.dist, if both are not None)
    /// is incorrect if casted twice
    pub fn add_meaning_dist(
        &mut self,
        meaning: Option<[f32; VECTOR_DIM]>,
        theme: Option<&MeanTheme>,
        sett: &MeaningSettings,
    ) {
        if let Some(theme) = theme {
            if let Some(meaning) = meaning {
                self.meaning = theme.dist(meaning, sett) * sett.weight;
                self.dist += self.meaning;
            }
        }
    }

    /// the same
    pub fn add_popularity_dist(&mut self, index: usize, sett: &PopularitySettings) {
        self.popularity = sett.weight * (index as f32).powf(sett.pow);
        self.dist += self.popularity;
    }

    pub fn add_unsymmetrical_dist(&mut self, sett: &UnsymmetricalSettings) {
        let delta = self.word.get_phones_count() as f32 - sett.optimal_length;
        if delta.is_sign_positive() {
            self.unsymmetrical = sett.more_w * delta.powf(sett.more_pow);
        } else {
            self.unsymmetrical = sett.less_w * (-delta).powf(sett.less_pow)
        }
        self.dist += self.unsymmetrical;
    }

    pub fn add_speech_part_dist(
        &mut self,
        to_find_sp: Option<&str>,
        my_sp: &str,
        sett: &SamePartSpeechSettings,
    ) {
        match to_find_sp {
            None => {}
            Some(sp) => {
                self.same_part = match (sp, my_sp) {
                    ("г", "г") => sett.verb,
                    ("с", "с") => sett.noun,
                    ("п", "п") => sett.adj,
                    ("н", "н") => sett.adv,
                    _ => 0.0,
                };
                self.dist += self.same_part;
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

impl Eq for WordDistanceResult<'_> {}

fn round3(n: f32) -> String {
    format!("{:<6}", f32::round(n * 1_000.0) / 1_000.0)
}

impl Debug for WordDistanceResult<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "\n{:<13} — {} (msc:{}; vwl:{}, cns:{}; str: {}; mng: {}; pop: {}; uns: {}; sSP: {})",
            self.word.src,
            round3(self.dist.into_inner()),
            round3(self.misc),
            round3(self.vowel),
            round3(self.cons),
            round3(self.structure),
            round3(self.meaning),
            round3(self.popularity),
            round3(self.unsymmetrical),
            round3(self.same_part)
        )
    }
}

impl Serialize for WordDistanceResult<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("WordDistanceResult", 8)?;
        s.serialize_field("dist", &self.dist.into_inner())?;
        s.serialize_field("misc", &self.misc)?;
        s.serialize_field("vowel", &self.vowel)?;
        s.serialize_field("cons", &self.cons)?;
        s.serialize_field("struct", &self.structure)?;
        s.serialize_field("meaning", &self.meaning)?;
        s.serialize_field("popular", &self.popularity)?;
        s.serialize_field("popular", &self.popularity)?;
        s.serialize_field("unsymm", &self.unsymmetrical)?;
        s.serialize_field("sameSP", &self.same_part)?;
        s.serialize_field("word", &self.word.src)?;
        s.end()
        /*pub dist: NotNan<f32>,
        misc: f32,
        vowel: f32,
        cons: f32,
        structure: f32,
        meaning: f32,
        popularity: f32,
        unsymmetrical: f32,*/
    }
}

/// saves &str in "raw" form;
/// **CAUTION**: will be **undefined behaviour** if the string it refs to will be deleted
/// I use it here to bypass a borrow checker — using it there *is safe* because I use it
/// to reference Strings *in the same object* the Strings are stored
/// All the fields are private, so it can't be deconstructed outside to cause UB.
struct UnsafeStrSaver(*const u8, usize);

impl UnsafeStrSaver {
    fn to_str(&self) -> &str {
        unsafe {
            let slice = slice::from_raw_parts(self.0, self.1);
            str::from_utf8_unchecked(slice) // correct if constructed using new
        }
    }

    fn to_bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.0, self.1) }
    }

    fn new(s: &str) -> Self {
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

impl Display for UnsafeStrSaver {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

unsafe impl Sync for UnsafeStrSaver {}

pub struct WordForms {
    pub start_index: usize,
    pub len: usize,
    pub meaning: [f32; VECTOR_DIM],
    pub speech_part: String,
}

impl WordForms {
    pub fn range(&self) -> std::ops::Range<usize> {
        self.start_index..self.start_index + self.len
    }
}

pub struct WordCollector {
    words: Vec<Word>,
    word_form_groups: Vec<WordForms>,
    string2index: HashMap<UnsafeStrSaver, usize>, // word string -> word index
    index2group_index: HashMap<usize, usize>,     // index of word -> index of wordgroup
    stress_indexing: HashMap<(u8, usize), HashSet<usize>>, // (letter, letter_index) -> [matching word index]
}

impl WordCollector {
    pub fn new(
        i2w: Vec<String>,
        mut zaliz: HashMap<String, String>,
        meanings: Vec<[f32; VECTOR_DIM]>,
    ) -> WordCollector {
        let mut words: Vec<Word> = vec![];
        let mut word_form_groups: Vec<WordForms> = vec![];
        let mut string2index = HashMap::new();
        let mut index2group_index = HashMap::new();
        let mut stress_indexing = HashMap::new();

        for (group_index, (name, meaning)) in zip(i2w, meanings).enumerate() {
            let data = zaliz.remove(&name).unwrap();
            let mut all_data = data.split('+');
            let speech_part: &str = all_data.next().unwrap();
            let mut bases: Vec<&str> = all_data.collect();
            let endings: Vec<&str> = bases.pop().unwrap().split(';').collect();

            let is_adj: bool = match speech_part {
                "п" | "мс" | "мс-п" | "г" | "числ-п" => true,
                _ => false,
            };

            let speech_part = speech_part.to_string();

            let w_form_group = WordForms {
                start_index: words.len(),
                len: endings.len(),
                meaning,
                speech_part,
            };
            word_form_groups.push(w_form_group);
            for mut e in endings.into_iter() {
                let mut e2 = e.to_string();
                let mut base = match e.chars().next() {
                    Some(c) if c.is_digit(10) => {
                        e2.remove(0);
                        e = &e2;
                        bases[c.to_digit(10).unwrap() as usize]
                    }
                    _ => bases[0],
                }
                .to_string();

                base.push_str(e);
                let w = Word::new(&base, is_adj);

                index2group_index.insert(words.len(), group_index);

                let stress_info = w.get_primary_stress();
                stress_indexing
                    .entry(stress_info)
                    .or_insert(HashSet::new())
                    .insert(words.len());

                words.push(w);
            }
        }
        let mut wc = WordCollector {
            words,
            word_form_groups,
            string2index: HashMap::new(),
            index2group_index,
            stress_indexing,
        };
        for wgroup in wc.word_form_groups.iter() {
            for word_index in wgroup.range() {
                let word_form = &wc.words[word_index];
                string2index.insert(UnsafeStrSaver::new(&*word_form.src), word_index);
            }
        }
        wc.string2index = string2index;
        wc
    }

    /// the actual finding work: filters words (bad stresses if indexing is enabled), then creates WordDistance objects for all filtered,
    /// pushes them into heap, and returns best *n* results
    /// *ignore* — will skip listed parts of speech  
    pub fn find_best<'c>(
        &'c self,
        info: &FindingInfo<'c, '_>,
        ignore: Vec<&str>,
        top_n: u32,
    ) -> Result<Vec<WordDistanceResult>, String> {
        let mut heap = TopNHeap::new(top_n as usize);
        let allowed = self.words_with_same_stresses(info.to_find);

        let regexp = info.to_find.get_regexp()?;
        let mut regexping = false;
        let allowed = match regexp {
            Some(reg) => {
                regexping = true;
                allowed
                    .filter(|i| reg.is_match(&self.words[*i].src))
                    .collect::<HashSet<usize>>()
            }
            None => allowed.collect::<HashSet<usize>>(),
        };

        for (wform_index, wform) in self.word_form_groups.iter().enumerate() {
            if ignore.contains(&&*wform.speech_part) {
                continue;
            }
            if info.gs.stresses.indexation || regexping {
                let res = WordDistanceResult::from_froms_with_filter(wform_index, &info, &allowed);

                if let Some(res) = res {
                    heap.push(res);
                }
            } else {
                let res = WordDistanceResult::from_forms(wform_index, &info);
                heap.push(res);
            }
        }

        Ok(heap.heap.into_sorted_vec())
    }

    /// returns iterator of corresponding word indexes
    pub fn words_with_same_stresses(&self, word: &Word) -> impl Iterator<Item = usize> + '_ {
        let stresses = word.get_all_stresses();

        stresses
            .into_iter()
            .map(|stress_info: (u8, usize)| {
                if stress_info.0 == symbol_id!(!) {
                    // "universal" letter
                    self.stress_indexing
                        .iter()
                        .filter(|(key, _)| key.1 == stress_info.1)
                        .map(|(_, v)| v)
                        .flatten()
                        .map(|x| *x)
                        .collect::<HashSet<usize>>()
                } else {
                    self.stress_indexing[&stress_info].clone()
                }
            })
            .flatten()
    }

    pub fn load_default(dir: &PathBuf) -> Self {
        crate::reader::load_default_word_collector(dir)
    }

    pub fn get_index(&self, not_stressed: &str) -> Option<&usize> {
        self.string2index.get(&UnsafeStrSaver::new(not_stressed))
    }

    /// returns matching group from index of word inside
    pub fn get_forms_by_word_index(&self, index: &usize) -> Option<&usize> {
        self.index2group_index.get(index)
    }

    pub fn get_word(&self, not_stressed: &str) -> Option<&Word> {
        self.get_index(not_stressed).map(|&ind| &self.words[ind])
    }

    pub fn get_forms(&self, not_stressed: &str) -> Option<&usize> {
        self.get_index(not_stressed)
            .and_then(|ind| self.get_forms_by_word_index(ind))
    }

    pub fn get_meaning(&self, not_stressed: &str) -> Option<[f32; VECTOR_DIM]> {
        self.get_forms(not_stressed)
            .map(|&i| self.word_form_groups[i].meaning)
    }

    pub fn get_speech_part(&self, not_stressed: &str) -> Option<&str> {
        self.get_forms(not_stressed)
            .map(|&i| &*self.word_form_groups[i].speech_part)
    }
}

struct TopNHeap<'collector> {
    max_dist: NotNan<f32>,
    collected: bool,
    top_n: usize,
    heap: BinaryHeap<WordDistanceResult<'collector>>,
}

impl<'collector> TopNHeap<'collector> {
    fn push(&mut self, res: WordDistanceResult<'collector>) {
        if !self.collected {
            self.heap.push(res);
            if self.heap.len() >= self.top_n {
                self.collected = true;
                self.max_dist = self.heap.peek().unwrap().dist;
            }
        } else {
            if self.max_dist > res.dist {
                self.heap.pop(); // pops the word with the greatest distance!
                self.heap.push(res);
                self.max_dist = self.heap.peek().unwrap().dist;
            }
        }
    }

    fn new(top_n: usize) -> Self {
        TopNHeap {
            max_dist: NotNan::new(f32::MAX).unwrap(),
            top_n,
            heap: BinaryHeap::new(),
            collected: false,
        }
    }
}

/// to stay at stable I use tests as benchmarks. Use them with `cargo test word_collect --release -- --nocapture`
#[cfg(test)]
#[test]
fn word_collect() {
    use crate::reader::MeanStrThemes;
    use std::time::Instant;
    let current = Instant::now();
    let wc = WordCollector::load_default(&PathBuf::from("."));
    let mf = MeanStrThemes::load_default(&PathBuf::from("."));
    let gs = GeneralSettings::load_default(&PathBuf::from("."));
    println!("Loaded words in {:#?}", current.elapsed());

    let current = Instant::now();

    let theme = MeanTheme::from_str(&wc, &mf.str_themes["Love"]).expect("Can't find words"); //&vec!["гиппопотам", "минотавр"]).unwrap();

    println!(
        "{:?}",
        wc.find_best(
            &FindingInfo::new(&wc, &Word::new("глазу'нья", false), &gs, Some(&theme)),
            vec![],
            50
        )
    );
    println!("Found words in {:#?} seconds", current.elapsed());

    let current = Instant::now();
    println!("{:?}", wc.get_word("ударение").unwrap().get_stresses());
    println!("Found stress in {:#?}", current.elapsed());

    let current = Instant::now();
    println!(
        "{:?}",
        wc.find_best(
            &FindingInfo::new(&wc, &Word::new("пра'вда", false), &gs, Some(&theme)),
            vec![],
            50
        )
    );
    println!("Found word in {:#?}", current.elapsed());

    let current = Instant::now();
    println!(
        "{:?}",
        wc.find_best(
            &FindingInfo::new(&wc, &Word::new("лома'ть", false), &gs, Some(&theme)),
            vec![],
            50
        )
    );
    println!("Found word in {:#?}", current.elapsed());

    let current = Instant::now();
    println!(
        "{:?}",
        wc.find_best(
            &FindingInfo::new(&wc, &Word::new("!кий", false), &gs, Some(&theme)),
            vec![],
            50
        )
    );
    println!("Found word «!кий» in {:#?}", current.elapsed());

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
fn profile_load() {
    use std::{thread, time::Duration};
    let mut wc = WordCollector::load_default(&PathBuf::from("."));

    println!("Sleeping (basic)");
    thread::sleep(Duration::from_millis(10_000));

    wc.words = vec![];
    println!("Removed words");
    thread::sleep(Duration::from_millis(10_000));

    wc.string2index = HashMap::new();
    println!("Removed stringify");
    thread::sleep(Duration::from_millis(10_000));
}
