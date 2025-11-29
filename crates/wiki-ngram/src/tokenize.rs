use anyhow::Result;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use vibrato::Tokenizer;
use zstd::Decoder;

pub fn load_tokenizer(dict_path: &Path) -> Result<Tokenizer> {
    log::info!("Loading dictionary from {:?}", dict_path);
    
    let file = File::open(dict_path)?;
    let mut decoder = Decoder::new(file)?;
    let mut dict_data = Vec::new();
    decoder.read_to_end(&mut dict_data)?;

    let dict = vibrato::Dictionary::read(&dict_data[..])?;
    let tokenizer = Tokenizer::new(dict);

    log::info!("Dictionary loaded successfully");
    Ok(tokenizer)
}

pub fn tokenize_text(tokenizer: &Tokenizer, text: &str) -> Vec<String> {
    let mut worker = tokenizer.new_worker();
    worker.reset_sentence(text);
    worker.tokenize();

    let mut tokens = Vec::new();
    for i in 0..worker.num_tokens() {
        let token = worker.token(i);
        tokens.push(token.surface().to_string());
    }
    tokens
}
