use clap::Parser;

mod translator_struct;
mod translator_ru;
mod finder;
mod reader;
mod meaner;
mod api;

#[cfg(test)]
mod tests;

use crate::finder::WordCollector;
use crate::reader::MeanStrThemes;
use crate::reader::GeneralSettings;
use crate::api::{Args, find_from_args};

fn main() {
    let wc = WordCollector::load_default();
    let mf = MeanStrThemes::load_default();
    let gs = GeneralSettings::load_default();
    let a = Args::parse();
    let words = find_from_args(&wc, &mf, &gs, &a);

    if !a.debug{
        match words.map(|v| v.iter().map(|r| &*r.word.src).collect::<Vec<&str>>()){
            Ok(v) => println!("{:?}", v),
            Err(s) => eprintln!("{}", s)
        }
    }
    else{
        println!("{:?}", words);
    }
    
}