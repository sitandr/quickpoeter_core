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
    pub mean: Option<String>,

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



pub fn find_from_args<'a>(wc: &'a WordCollector, mf: &'a MeanStrFields, args: Args) -> Result<Vec<WordDistanceResult<'a>>, String>{
    let field;

    let field_ref = match args.mean{
        Some(mean) => {
            field = MeanField::from_strings(wc, &mf.str_fields[&mean]).unwrap();
            Some(&field)
        },
        None => None
    };

    let rps = match args.rps{
        Some(s) => s.split("+").map(|x| x.to_owned()).collect(),
        None => vec![]
    };

    let mut to_find = args.to_find;
    let word;

    if to_find.chars().all(|c| match c {'+'|'!' => true, _ => false}){
        word = Word::new_abstract(&to_find);
        dbg!(word.sylls.len());
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
        word = Word::new(&to_find, false, None);
    }

    
    let words = wc.find_best(&word, rps.iter().map(|s| &**s).collect(), args.top_n.into(), field_ref);

    Ok(words)
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