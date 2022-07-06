mod translator_struct;
mod translator_ru;
mod finder;
mod reader;

#[cfg(test)]
mod tests;

fn main() {
    // println!("{:#?}", reader::read_settings());
    let rd = reader::RawData::load_default();
    dbg!(&rd.index2word[10]);
    dbg!(&rd.word2index["слово"]);
//    dbg!(&rd.min_zaliz["слово"]);
}
