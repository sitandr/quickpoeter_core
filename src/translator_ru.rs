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


Module that keeps the specific knowledge of russian language rules. :)
*/

use std::fmt::Debug;

use crate::{
    reader::{ConsonantDistanceSettings, VowelDistanceSettings},
    translator_struct::*,
};
/*use lazy_static::lazy_static;
use regex::Regex;*/

// use smallvec::SmallVec;
// type Vec<T> = SmallVec<[T;15]>;

const J_MARKERS: [char; 10] = ['а', 'о', 'э', 'и', 'ы', 'у', 'ь', 'ъ', '\'', '`']; // ' and ` mean there is a vowel before => marker

const J_VOWELS: [char; 4] = ['е', 'ё', 'ю', 'я'];
pub const ALL_VOWELS: [char; 10] = ['а', 'о', 'э', 'и', 'ы', 'у', 'е', 'ё', 'ю', 'я'];

macro_rules! J_MAP {
    ($x:expr) => {
        match $x {
            'ё' => 'о',
            'е' => 'э',
            'ю' => 'у',
            'я' => 'а',
            _ => unreachable!(),
        }
    };
}

// const J_VOWELS: Vec<&char> = J_MAP.keys().collect();
const SOFTABLE: [char; 10] = ['с', 'х', 'ф', 'к', 'т', 'п', 'р', 'л', 'н', 'м'];
const REMOVING_VOICE: [char; 6] = ['п', 'ф', 'к', 'т', 'ш', 'с'];

//0      Ф Т С   Ч
//1 РЛНМ П     Ш
//2        К Х
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
    (3.0, 0.0), // ф
];
/*
 о
  а  э
        и
у     ы
*/
const ASSONANSES: [(f32, f32); 6] = [
    (5.0, 5.0), // а
    (4.0, 6.0), // о
    (7.0, 5.0), // э
    (9.0, 4.0), // и
    (7.0, 3.0), // ы
    (3.0, 3.0), // у
];

/// important symbols used in code
macro_rules! symbol_id {
    (!) => {
        6
    };
    (+) => {
        7
    };
    (й) => {
        12
    };
}

pub(crate) use symbol_id;

#[derive(Clone)]
pub struct Vowel {
    pub letter: u8,
    pub accent: Accent, // 0 if None, 2 if secondary, 1 if primary\
}

impl Vowel {
    pub const ALL: [char; 8] = ['а', 'о', 'э', 'и', 'ы', 'у', '!', '+'];
}

impl Voweable for Vowel {
    fn distance(&self, other: &Self, sett: &VowelDistanceSettings) -> f32 {
        if self.letter == symbol_id!(+)
            || self.letter == symbol_id!(!)
            || other.letter == symbol_id!(+)
            || other.letter == symbol_id!(!)
        {
            return 1.0;
        }
        if cfg!(feature = "edit_distances") {
            let (x1, y1) = sett.map[self.letter as usize];
            let (x2, y2) = sett.map[other.letter as usize];

            1.0_f32.min(
                ((x1 - x2).abs().powf(sett.pow) + (y1 - y2).abs().powf(sett.pow))
                    / sett.denominator,
            )
        } else {
            let (x1, y1) = ASSONANSES[self.letter as usize];
            let (x2, y2) = ASSONANSES[other.letter as usize];

            1.0_f32.min(((x1 - x2).abs().powf(0.5) + (y1 - y2).abs().powf(0.5)) / 3.0)
        }
    }

    fn accent(&self) -> Accent {
        self.accent
    }
}

impl Debug for Vowel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            Self::ALL[self.letter as usize],
            match self.accent {
                Accent::Primary => "\'",
                Accent::Secondary => "`",
                Accent::NoAccent => "",
            }
        )
    }
}

#[derive(Clone)]
pub struct Consonant {
    pub letter: u8,
    pub voiced: bool,      // звонкая
    pub palatalized: bool, // мягкая
}
impl Consonant {
    /// order is extremely important
    pub const ALL: [char; 13] = [
        'р', 'л', 'н', 'м', 'п', 'т', 'к', 'с', 'х', 'ш', 'ч', 'ф', 'й',
    ];
}

impl Phonable for Vowel {
    fn contains_char(c: &char) -> bool {
        Self::ALL.contains(c)
    }
}

impl Phonable for Consonant {
    fn contains_char(c: &char) -> bool {
        Self::ALL.contains(c)
    }
}

impl Consonantable for Consonant {
    fn distance(&self, other: &Self, sett: &ConsonantDistanceSettings) -> f32 {
        if self.letter == other.letter {
            return 0.0;
        }
        if self.letter != symbol_id!(й) && other.letter != symbol_id!(й) {
            if cfg!(feature = "edit_distances") {
                let (x1, y1) = sett.map[self.letter as usize];
                let (x2, y2) = sett.map[other.letter as usize];
                let mut d: f32 = 0.0;
                if self.voiced == other.voiced {
                    d += 0.5
                }
                if self.palatalized == other.palatalized {
                    d += 0.5
                };

                1.0_f32.min(
                    ((x1 - x2).abs().powf(sett.pow) + (y1 - y2).abs().powf(sett.pow) + d)
                        / sett.denominator,
                )
            } else {
                let (x1, y1) = ALLITERATION[self.letter as usize];
                let (x2, y2) = ALLITERATION[other.letter as usize];
                let mut d: f32 = 0.0;
                if self.voiced == other.voiced {
                    d += 0.5
                }
                if self.palatalized == other.palatalized {
                    d += 0.5
                };

                1.0_f32.min(((x1 - x2).abs().powf(0.5) + (y1 - y2).abs().powf(0.5) + d) / 3.0)
            }
        } else {
            // й + …? — already checked they are not equal
            1.0
        }
    }
}

impl Debug for Consonant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            Self::ALL[self.letter as usize],
            if self.voiced { "*" } else { "" },
            if self.palatalized { "^" } else { "" }
        )
    }
}

pub fn transcript(w: &str, is_adj: bool) -> String {
    // returns a postfix transcript like "к*ара'ш"
    let mut w: Vec<char> = w.to_lowercase().chars().collect();
    if is_adj {
        replace_g_in_adj(&mut w);
    }
    j_replace(&mut w);
    i_soften(&mut w);
    letter_replace(&mut w);
    remove_voice(&mut w);
    replace_oa(&mut w);
    w.into_iter().collect()
}

fn j_replace(w: &mut Vec<char>) {
    if J_VOWELS.contains(&w[0]) {
        // енот — "й" в начале слова
        if w[0] == 'ё' {
            w.insert(1, '\'');
        }
        w[0] = J_MAP!(&w[0]);
        w.insert(0, 'й')
    }

    let mut offset: usize = 0;
    for i in 1..w.len() {
        // starting with 1 because already checked the start
        let ind = i + offset;
        let val = w[ind];

        if J_VOWELS.contains(&val) {
            w[ind] = J_MAP!(&val); // е —> э
            if val == 'ё' {
                w.insert(ind + 1, '\'');
                offset += 1;
            }

            if J_MARKERS.contains(&w[ind - 1]) {
                w.insert(ind, 'й')
            } else {
                w.insert(ind, '^')
            }
            offset += 1; // shift indexes
        } else if val == 'о' && w[i - 1] == 'ь' {
            // бульон
            w.insert(ind, 'й');
            offset += 1;
        }
    }
}

fn i_soften(w: &mut Vec<char>) {
    // "и" is the only non-j vowel that softens those before
    let mut offset: usize = 0;
    for i in 1..w.len() {
        // starting with 1 because can't soften the first letter
        let ind = i + offset;
        let val = &w[ind];

        if *val == 'и' && SOFTABLE.contains(&w[i - 1]) {
            w.insert(i, '^');
            offset += 1; // shift indexes
        }
    }
}

fn letter_replace(w: &mut Vec<char>) {
    let mut offset: usize = 0;
    for i in 0..w.len() {
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
            _ => None,
        };

        if let Some(val) = replacement {
            offset = offset.wrapping_add(val.len().wrapping_sub(1));
            w.splice(ind..ind + 1, val.into_iter());
        }
    }
}

fn check_previous_voice(w: &mut Vec<char>, pos: usize) -> Option<usize> {
    if pos <= 0 {
        return None;
    }
    if w[pos - 1] == '*' {
        return Some(pos - 1);
    } else if pos >= 2 && w[pos - 1] == '^' && w[pos - 2] == '*' {
        return Some(pos - 2);
    }
    None
}

fn is_voiced(w: &mut Vec<char>, pos: usize) -> bool {
    if pos >= w.len() - 2 {
        return false;
    }
    if w[pos + 1] == '*' {
        return true;
    } else if pos <= w.len() - 3 && w[pos + 1] == '^' && w[pos + 2] == '*' {
        return true;
    }
    false
}

fn remove_voice(w: &mut Vec<char>) {
    if let Some(pos) = check_previous_voice(w, w.len()) {
        w.remove(pos);
    }

    let mut offset = 0;
    for i in 1..w.len() {
        // can't remove voice from nothing
        let ind: usize = i.wrapping_add(offset);
        let val = &w[ind];

        if REMOVING_VOICE.contains(val) && (!is_voiced(w, ind)) {
            if let Some(pos) = check_previous_voice(w, ind) {
                w.remove(pos);
                offset = offset.wrapping_sub(1);
            }
        }
    }
}

fn replace_oa(w: &mut Vec<char>) {
    for i in 0..w.len() - 1 {
        if w[i] == 'о' && !(w[i + 1] == '\'' || w[i + 1] == '`') {
            w[i] = 'а';
        }
    }
}

fn replace_g_in_adj(w: &mut Vec<char>) {
    if w.len() < 3 {
        return;
    }
    if w[w.len() - 3..w.len()] == ['е', 'г', 'о'] {
        w.splice(w.len() - 3..w.len(), ['е', 'в', 'о'].into_iter());
    } else if w[w.len() - 3..w.len()] == ['о', 'г', 'о'] {
        w.splice(w.len() - 3..w.len(), ['о', 'в', 'о'].into_iter());
    }
}

#[cfg(test)]
#[test]
fn j_replace_check() {
    assert_eq!(transcript("а'+", false), "а'+");
    assert_eq!(transcript("Я", false), "йа");
    assert_eq!(transcript("Митя Ляпин", false), "м^ит^а л^апин");
    assert_eq!(transcript("Митя Льяпин", false), "м^ит^а л^йапин");
    assert_eq!(transcript("Енёня`яя", false), "йэн^о'н^а`йайа");
    assert_eq!(transcript("миньо'н", false), "м^ин^йо'н");
    assert_eq!(transcript("бабузжка", false), "п*ап*ус*шка");
    assert_eq!(transcript("гро'б", false), "к*ро'п");
    assert_eq!(transcript("до'ждь", false), "т*о'шт^");
    assert_eq!(transcript("его", true), "йэф*о");
    assert_eq!(transcript("кроманьо'нец", false), "краман^йо'н^этс");
    assert_eq!(transcript("Ёжик", false), "йо'ш*ик");
}

#[cfg(test)]
#[test]
fn testing() {
    use std::time::Instant;

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
