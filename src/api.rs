use crate::translator_struct::Word;
use crate::meaner::MeanTheme;
use crate::reader::{MeanStrThemes, GeneralSettings};
use crate::finder::{WordCollector, WordDistanceResult, FindingInfo};
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

    /// Mean theme name
    #[clap(short, long, value_parser)]
    pub theme: Option<String>,

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
    #[clap(short='n', long, value_parser, default_value_t=100)]
    pub top_n: u32,

    /// Print all subdistances
    #[clap(short, long, value_parser, default_value_t=false)]
    pub debug: bool,

    #[clap(short, long, value_parser)]
    pub measure: Option<String>
}

pub fn string2word(wc: &WordCollector, to_find: &String) -> Result<Word, String>{
    if to_find.chars().all(|c| match c {'+'|'!' => true, _ => false}){
        Ok(Word::new_abstract(&to_find))
    }
    else{
        let to_find = to_find.to_lowercase();
        if !to_find.contains('\''){
            let found = wc.get_word(&to_find);
            if let Some(founded_some) = found{
                Ok(founded_some.clone())
            }
            else{
                Err("Word not found; Please mind the stress with «'» (and «`» for secondary stresses)".to_string())
            }
        }
        else{
            let chrs: Vec<char> = to_find.chars().collect();
            // check whether the word is correct
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
            // construct it
            Ok(Word::new(&to_find, false))
        }
    }
}

pub fn split_by_plus(rps: Option<String>) -> Vec<String>{
    rps.map_or(vec![], |s| s.split("+").map(|x| x.to_owned()).collect())
}


pub fn get_theme_by_key(wc: &WordCollector, mf: &MeanStrThemes, key: Option<String>) -> Result<Option<MeanTheme>, String>{
    
    key.map(|k| { // -> Result<MF, String>
        let strings_or_err = mf.str_themes.get(&k);
        match strings_or_err {
            Some(strings) => MeanTheme::from_str(wc, &strings).map_err(|vs| format!("{:?}", vs)),
            None => Err(format!("Unknown theme: {}", k).to_string()),
        }
    }).transpose()
}

/// debug function to get distances between two words
/// don't use it for production purpose
pub fn measure<'a>(wc: &'a WordCollector, mf: &'_ MeanStrThemes, gs: &'_ GeneralSettings, args: &'_ Args) -> Result<String, String>{
    let theme = get_theme_by_key(wc, mf, args.theme.clone())?;
    let word = string2word(wc, &args.to_find.clone())?;
    let info = FindingInfo::new(wc, &word, &gs, theme.as_ref());

    let measured_s = args.measure.as_ref().ok_or("No measure value")?;

    let measured = string2word(wc, measured_s)?;
    let mut r = WordDistanceResult::new(&word, &measured, &gs);
    if let Some(i) = wc.get_forms(measured_s){
        r.add_form_dists(&info, *i);
    }
        
    Ok(serde_yaml::to_string(&r).or(Err("Error in yaml creating"))?)
}
pub fn find_from_args<'a>(wc: &'a WordCollector, mf: &'_ MeanStrThemes, gs: &'_ GeneralSettings, args: &'_ Args) -> Result<Vec<WordDistanceResult<'a>>, String>{
    let theme = get_theme_by_key(wc, mf, args.theme.clone())?;
    let rps = split_by_plus(args.rps.clone());
    let word = string2word(wc, &args.to_find.clone())?;
    let info = FindingInfo::new(wc, &word, &gs, theme.as_ref());
    let words = wc.find_best(&info, rps.iter().map(|s| &**s).collect(), args.top_n);
    Ok(words)
}

#[allow(dead_code)]
pub fn find<'a>(wc: &'a WordCollector, gs: &'_ GeneralSettings, to_find: Word, theme: Option<&MeanTheme>, rps: &Vec<String>, top_n: u32) -> Vec<WordDistanceResult<'a>>{
    let info = FindingInfo::new(wc, &to_find, &gs, theme);
    wc.find_best(&info, rps.iter().map(|s| &**s).collect(), top_n)
}
