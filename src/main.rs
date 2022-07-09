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

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// What to find (use ' to mind the stress)
    #[clap(value_parser)]
    to_find: String,

    /// Mean field name
    #[clap(short, long, value_parser)]
    mean: Option<String>,

    /// Remove parts of speech
    #[clap(short, long, value_parser)]
    rps: Option<String>,

    /// Remove parts of speech
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

    println!("{:?}", wc.find_best(&Word::new(&args.to_find, false, None), rps.iter().map(|s| &**s).collect(), args.top_n.into(), field_ref));
    
}