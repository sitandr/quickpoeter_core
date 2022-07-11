mod translator_struct;
mod translator_ru;
mod finder;
mod reader;
mod meaner;

#[cfg(test)]
mod tests;

use crate::translator_struct::Word;
use crate::meaner::MeanField;
use crate::reader::MeanStrFields;
use crate::finder::WordCollector;
use clap::Parser;

/// Compex tool for finding ryphms;
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// What to find (use ' to mind the stress)
    #[clap(value_parser)]
    to_find: String,

    /// Mean field name
    #[clap(short, long, value_parser)]
    mean: Option<String>,

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
    rps: Option<String>,

    /// Number of selected best matches
    #[clap(short, long, value_parser, default_value_t=100)]
    top_n: u8,
}

fn main() {
    let args = Args::parse();

    let wc = WordCollector::load_default();
    let mf = MeanStrFields::load_default();

    let field;

    let field_ref = match args.mean{
        Some(mean) => {
            field = MeanField::from_strings(&wc, &mf.str_fields[&mean]).unwrap();
            Some(&field)
        },
        None => None
    };

    let rps = match args.rps{
        Some(s) => s.split("+").map(|x| x.to_owned()).collect(),
        None => vec![]
    };

    let mut to_find = args.to_find;

    if !to_find.contains('\''){
        let found = wc.get_word(&to_find);
        if let Some(founded_some) = found{
            to_find = founded_some.src.to_string();
        }
        else{
            println!("Word not found; Please mind the stress with «'» (and «`» for secondary stresses)");
            return;
        }
    }

    println!("{:?}", wc.find_best(&Word::new(&to_find, false, None), rps.iter().map(|s| &**s).collect(), args.top_n.into(), field_ref));
    
}