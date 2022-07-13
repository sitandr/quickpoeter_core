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
use crate::reader::MeanStrFields;
use crate::api::{Args, find_from_args};

fn main() {
    let wc = WordCollector::load_default();
    let mf = MeanStrFields::load_default();
    let words = find_from_args(&wc, &mf, Args::parse());
    println!("{:?}", words);
    
}