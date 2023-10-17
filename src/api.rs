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


This module provides commands for using tool from extern sources (or console)
*/

use crate::finder::{FindingInfo, WordCollector, WordDistanceResult};
use crate::meaner::MeanTheme;
use crate::reader::{GeneralSettings, MeanStrThemes};
use crate::translator_ru::ALL_VOWELS;
use crate::translator_struct::Word;
use clap::Parser;

/// Compex tool for finding ryphms;
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// What to find (use ' to mind the stress)
    #[clap(value_parser)]
    pub to_find: String,

    /// Mean theme name (one from config/themes.yaml)
    #[clap(short, long, value_parser)]
    pub theme: Option<String>,

    /// Remove some parts of speech separated with "+"
    /// List of available parts of speech:
    /// с      существительное
    /// п      прилагательное  
    /// мс     местоимение-существительное  
    /// мс-п   местоименное-прилагательное  
    /// г      глагол  
    /// н      наречие  
    /// числ   числительное  
    /// числ-п счётное прилагательное  
    /// вводн  вводное слово  
    /// межд   межометие  
    /// предик предикатив  
    /// предл  предлог  
    /// союз   союз  
    /// сравн  сравнительная степень  
    /// част   частица  
    /// ?      куски фразеологизмов и т.п.  
    #[clap(short, long, value_parser, verbatim_doc_comment)]
    pub rps: Option<String>,

    /// Number of returned best matches (doesn't affect speed)
    #[clap(short = 'n', long, value_parser, default_value_t = 100)]
    pub top_n: u32,

    /// Print all subdistances
    #[clap(short, long, value_parser, default_value_t = false)]
    pub debug: bool,

    /// Measure distance to given word (primarly for debug purposes)
    #[clap(short, long, value_parser)]
    pub measure: Option<String>,
}

pub fn string2word(wc: &WordCollector, to_find: &String) -> Result<Word, String> {
    if to_find.chars().all(|c| match c {
        '+' | '!' => true,
        _ => false,
    }) {
        Ok(Word::new(to_find, false))
    } else {
        let to_find = to_find.to_lowercase();
        if !to_find.contains('\'') && !to_find.contains('!') {
            let found = wc.get_word(&to_find);
            if let Some(founded_some) = found {
                Ok(founded_some.clone())
            } else {
                Err("Word not found; Please mind the stress with «'» (and «`» for secondary stresses)".to_string())
            }
        } else {
            let chrs: Vec<char> = to_find.chars().collect();
            // check whether the word is correct
            for i in 0..chrs.len() {
                let c = chrs[i];
                match c {
                    'а'..='я' => {}
                    'ё' => {}
                    '`' | '\'' => {
                        let previous_c = chrs
                            .get(i - 1)
                            .ok_or("Stress symbol at start of the word".to_string())?;
                        if !(ALL_VOWELS.contains(previous_c)) {
                            return Err("Stress not after the vowel".to_owned());
                        }
                    }
                    '+' | '!' => {}
                    _ => return Err(format!("Unknown charachter {}", c)),
                }
            }
            // construct it
            Ok(Word::new(&to_find, false))
        }
    }
}

pub fn split_by_plus(rps: Option<String>) -> Vec<String> {
    rps.map_or(vec![], |s| s.split('+').map(|x| x.to_owned()).collect())
}

pub fn get_theme_by_key(
    wc: &WordCollector,
    mf: &MeanStrThemes,
    key: Option<String>,
) -> Result<Option<MeanTheme>, String> {
    key.map(|k| {
        // -> Result<MF, String>
        let strings_or_err = mf.str_themes.get(&k);
        match strings_or_err {
            Some(strings) => MeanTheme::from_str(wc, strings).map_err(|vs| format!("{:?}", vs)),
            None => Err(format!("Unknown theme: {}", k).to_string()),
        }
    })
    .transpose()
}

/// debug function to get distances between two words
/// don't use it for production purpose (it is rather slow)
pub fn measure(
    wc: &WordCollector,
    mf: &'_ MeanStrThemes,
    gs: &'_ GeneralSettings,
    args: &'_ Args,
) -> Result<String, String> {
    let theme = get_theme_by_key(wc, mf, args.theme.clone())?;
    let word = string2word(wc, &args.to_find.clone())?;
    let info = FindingInfo::new(wc, &word, gs, theme.as_ref());

    let measured_s = args.measure.as_ref().ok_or("No measure value")?;

    let measured = string2word(wc, measured_s)?;
    let mut r = WordDistanceResult::new(&word, &measured, gs);
    if let Some(i) = wc.get_forms(measured_s) {
        r.add_form_dists(&info, *i);
    }

    Ok(serde_yaml::to_string(&r).or(Err("Error in yaml creating"))?)
}
pub fn find_from_args<'a>(
    wc: &'a WordCollector,
    mf: &'_ MeanStrThemes,
    gs: &'_ GeneralSettings,
    args: &'_ Args,
) -> Result<Vec<WordDistanceResult<'a>>, String> {
    let theme = get_theme_by_key(wc, mf, args.theme.clone())?;
    let rps = split_by_plus(args.rps.clone());
    let word = string2word(wc, &args.to_find.clone())?;
    let info = FindingInfo::new(wc, &word, gs, theme.as_ref());
    let words = wc.find_best(&info, rps.iter().map(|s| &**s).collect(), args.top_n)?;
    Ok(words)
}

#[allow(dead_code)]
pub fn find<'a>(
    wc: &'a WordCollector,
    gs: &'_ GeneralSettings,
    to_find: Word,
    theme: Option<&MeanTheme>,
    rps: &Vec<String>,
    top_n: u32,
) -> Result<Vec<WordDistanceResult<'a>>, String> {
    let info = FindingInfo::new(wc, &to_find, gs, theme);
    wc.find_best(&info, rps.iter().map(|s| &**s).collect(), top_n)
}
