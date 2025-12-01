use std::fs::File;
use std::io::BufReader;
use vibrato::Dictionary;
use vibrato::Tokenizer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("output/system.dic.zst")?;
    let reader = BufReader::new(file);
    let decoder = zstd::stream::read::Decoder::new(reader)?;
    let dict = Dictionary::read(decoder)?;
    
    let tokenizer = Tokenizer::new(dict);
    let mut worker = tokenizer.new_worker();
    
    let text = "今日はいい天気です";
    worker.reset_sentence(text);
    worker.tokenize();
    
    println!("Tokenizing: {}", text);
    for i in 0..worker.num_tokens() {
        let t = worker.token(i);
        println!("{}: {} ({})", i, t.surface(), t.feature());
    }
    
    Ok(())
}
