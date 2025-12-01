use std::fs::File;
use std::io::BufReader;
use vibrato::{Dictionary, Tokenizer};

use std::path::PathBuf;

fn get_dict_path() -> PathBuf {
    std::env::var("MOZC_DICT_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("../../output/system.dic.zst"))
}

#[test]
fn test_load_mozc_dictionary() {
    let file = File::open(get_dict_path())
        .expect("Failed to open dictionary file. Set MOZC_DICT_PATH env var or run 'cargo run -p mozc-dict-gen --release' first.");
    let reader = BufReader::new(file);
    let decoder = zstd::stream::read::Decoder::new(reader)
        .expect("Failed to create zstd decoder");
    let dict = Dictionary::read(decoder)
        .expect("Failed to read dictionary");
    
    // If we got here, the dictionary loaded successfully
    assert!(true);
}

#[test]
fn test_tokenize_common_phrases() {
    let file = File::open(get_dict_path())
        .expect("Failed to open dictionary file. Set MOZC_DICT_PATH env var or run 'cargo run -p mozc-dict-gen --release' first.");
    let reader = BufReader::new(file);
    let decoder = zstd::stream::read::Decoder::new(reader)
        .expect("Failed to create zstd decoder");
    let dict = Dictionary::read(decoder)
        .expect("Failed to read dictionary");
    
    let tokenizer = Tokenizer::new(dict);
    let mut worker = tokenizer.new_worker();
    
    // Test case 1: "東京" (Tokyo)
    worker.reset_sentence("東京");
    worker.tokenize();
    assert!(worker.num_tokens() > 0, "Should tokenize '東京'");
    
    // Test case 2: "こんにちは" (Hello)
    worker.reset_sentence("こんにちは");
    worker.tokenize();
    assert!(worker.num_tokens() > 0, "Should tokenize 'こんにちは'");
    
    // Test case 3: "日本語" (Japanese language)
    worker.reset_sentence("日本語");
    worker.tokenize();
    assert!(worker.num_tokens() > 0, "Should tokenize '日本語'");
    
    // Test case 4: Mixed - "東京に行きます" (I will go to Tokyo)
    worker.reset_sentence("東京に行きます");
    worker.tokenize();
    assert!(worker.num_tokens() >= 3, "Should tokenize '東京に行きます' into multiple tokens");
}

#[test]
fn test_tokenize_edge_cases() {
    let file = File::open(get_dict_path())
        .expect("Failed to open dictionary file. Set MOZC_DICT_PATH env var or run 'cargo run -p mozc-dict-gen --release' first.");
    let reader = BufReader::new(file);
    let decoder = zstd::stream::read::Decoder::new(reader)
        .expect("Failed to create zstd decoder");
    let dict = Dictionary::read(decoder)
        .expect("Failed to read dictionary");
    
    let tokenizer = Tokenizer::new(dict);
    let mut worker = tokenizer.new_worker();
    
    // Test empty string
    worker.reset_sentence("");
    worker.tokenize();
    assert_eq!(worker.num_tokens(), 0, "Empty string should produce no tokens");
    
    // Test single character
    worker.reset_sentence("あ");
    worker.tokenize();
    assert!(worker.num_tokens() > 0, "Single character should be tokenized");
    
    // Test mixed scripts (Hiragana, Katakana, Kanji)
    worker.reset_sentence("ひらがなカタカナ漢字");
    worker.tokenize();
    assert!(worker.num_tokens() > 0, "Mixed scripts should be tokenized");
}

#[test]
fn test_tokenization_produces_features() {
    let file = File::open(get_dict_path())
        .expect("Failed to open dictionary file. Set MOZC_DICT_PATH env var or run 'cargo run -p mozc-dict-gen --release' first.");
    let reader = BufReader::new(file);
    let decoder = zstd::stream::read::Decoder::new(reader)
        .expect("Failed to create zstd decoder");
    let dict = Dictionary::read(decoder)
        .expect("Failed to read dictionary");
    
    let tokenizer = Tokenizer::new(dict);
    let mut worker = tokenizer.new_worker();
    
    worker.reset_sentence("東京");
    worker.tokenize();
    
    for i in 0..worker.num_tokens() {
        let token = worker.token(i);
        assert!(!token.surface().is_empty(), "Token surface should not be empty");
        assert!(!token.feature().is_empty(), "Token feature should not be empty");
    }
}
