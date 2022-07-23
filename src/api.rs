use crate::translator_struct::Word;
use crate::meaner::MeanField;
use crate::reader::MeanStrFields;
use crate::finder::{WordCollector, WordDistanceResult};
use clap::Parser;
use crate::translator_ru::Vowel;
use crate::translator_ru::J_VOWELS;

/// Compex tool for finding ryphms;
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// What to find (use ' to mind the stress)
    #[clap(value_parser)]
    pub to_find: String,

    /// Mean field name
    #[clap(short, long, value_parser)]
    pub field: Option<String>,

    /// Remove some parts of speech
    /// List of available parts of speech (буквы везде русские):
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
    #[clap(short, long, value_parser)]
    pub rps: Option<String>,

    /// Number of selected best matches
    #[clap(short, long, value_parser, default_value_t=100)]
    pub top_n: u32,
}

pub fn string2word(wc: &WordCollector, mut to_find: String) -> Result<Word, String>{
    if to_find.chars().all(|c| match c {'+'|'!' => true, _ => false}){
        Ok(Word::new_abstract(&to_find))
    }
    else{
        to_find = to_find.to_lowercase();
        to_find = auto_stress(&wc, &to_find).ok_or("Word not found; Please mind the stress with «'» (and «`» for secondary stresses)".to_string())?;
        let chrs: Vec<char> = to_find.chars().collect();
        for i in 0..chrs.len(){
            let c = chrs[i];
            match c{
                'а' ..= 'я' => {},
                'ё' => {},
                '`'|'\'' => {
                    let previous_c = chrs.get(i - 1).ok_or("Stress symbol at start of the word".to_string())?;
                    if !(Vowel::ALL.contains(previous_c)||J_VOWELS.contains(previous_c)){
                        return Err("Stress not after the vowel".to_owned());
                    }
                },
                _ => return Err(format!("Unknown charachter {}", c)),
            }
        }
        Ok(Word::new(&to_find, false, None))
    }
}

pub fn split_by_plus(rps: Option<String>) -> Vec<String>{
    rps.map_or(vec![], |s| s.split("+").map(|x| x.to_owned()).collect())
}

pub fn get_field_by_key(wc: &WordCollector, mf: &MeanStrFields, key: Option<String>) -> Result<Option<MeanField>, String>{
    
    key.map(|k| { // -> Result<MF, String>
        let strings_or_err = mf.str_fields.get(&k);
        match strings_or_err {
            Some(strings) => MeanField::from_strings(wc, &strings).map_err(|vs| format!("{:?}", vs)),
            None => Err(format!("Unknown field: {}", k).to_string()),
        }
    }).transpose()
}


pub fn find_from_args<'a>(wc: &'a WordCollector, mf: &'a MeanStrFields, args: Args) -> Result<Vec<WordDistanceResult<'a>>, String>{
    let field = get_field_by_key(wc, mf, args.field)?;
    let rps = split_by_plus(args.rps);
    let word = string2word(wc, args.to_find)?;
    let words = wc.find_best(&word, rps.iter().map(|s| &**s).collect(), args.top_n, field.as_ref());

    Ok(words)
}

pub fn find<'a>(wc: &'a WordCollector, to_find: Word, field: &Option<MeanField>, rps: &Vec<String>, top_n: u32) -> Vec<WordDistanceResult<'a>>{
    wc.find_best(&to_find, rps.iter().map(|s| &**s).collect(), top_n, field.as_ref())
}

pub fn auto_stress(wc: &WordCollector, to_find: &str) -> Option<String>{
    if !to_find.contains('\''){
        let found = wc.get_word(&to_find);
        if let Some(founded_some) = found{
            return Some(founded_some.src.to_string());
        }
        else{
            return None;
        }
    }
    Some(to_find.to_string())
}