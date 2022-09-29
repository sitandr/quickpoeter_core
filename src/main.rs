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


Module that works with cli (isn't included in library)
*/

use clap::Parser;

mod translator_struct;
mod translator_ru;
mod finder;
mod reader;
mod meaner;
mod api;

#[cfg(test)]
mod tests;

use crate::api::measure;
use crate::finder::WordCollector;
use crate::reader::MeanStrThemes;
use crate::reader::GeneralSettings;
use crate::api::{Args, find_from_args};

fn main() {
    let wc = WordCollector::load_default(".");
    let mf = MeanStrThemes::load_default(".");
    let gs = GeneralSettings::load_default(".");
    let a = Args::parse();
    

    if a.measure.is_some(){
        let r = measure(&wc, &mf, &gs, &a);
        println!("{}", match  r{
            Ok(r) => r,
            Err(r) => r
        });
        return;
    }

    let words = find_from_args(&wc, &mf, &gs, &a);

    if a.debug{
        println!("{:?}", words);
    }
    else{
        match words.map(|v| v.iter().map(|r| &*r.word.src).collect::<Vec<&str>>()){
            Ok(v) => println!("{:?}", v),
            Err(s) => eprintln!("{}", s)
        }
    }
    
}