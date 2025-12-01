use fst::{IntoStreamer, Map, Streamer};
use memmap2::Mmap;
use std::fs::File;

use std::path::PathBuf;

fn get_fst_path() -> PathBuf {
    std::env::var("WIKI_NGRAM_FST_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("../../output/wiki-ngrams.fst"))
}

#[test]
fn test_load_ngram_fst() {
    let file = File::open(get_fst_path())
        .expect("Failed to open FST file. Set WIKI_NGRAM_FST_PATH env var or run 'cargo run -p wiki-ngram --release' first.");
    let mmap = unsafe { Mmap::map(&file) }
        .expect("Failed to mmap FST file");
    let fst = Map::new(mmap)
        .expect("Failed to load FST");
    
    // If we got here, the FST loaded successfully
    assert!(true);
}

#[test]
fn test_query_common_ngrams() {
    let file = File::open(get_fst_path())
        .expect("Failed to open FST file. Set WIKI_NGRAM_FST_PATH env var or run 'cargo run -p wiki-ngram --release' first.");
    let mmap = unsafe { Mmap::map(&file) }
        .expect("Failed to mmap FST file");
    let fst = Map::new(mmap)
        .expect("Failed to load FST");
    
    // Test common bigrams/trigrams that should exist in Wikipedia
    let test_cases = vec![
        "東京 都",
        "日本 の",
        "こと が",
        "する こと",
        "ある ので",
    ];
    
    let mut found_count = 0;
    for ngram in test_cases {
        if let Some(score) = fst.get(ngram) {
            found_count += 1;
            assert!(score > 0, "Score for '{}' should be greater than 0", ngram);
            println!("Found '{}' with score: {}", ngram, score);
        }
    }
    
    // At least some common n-grams should be found
    assert!(found_count > 0, "Should find at least some common n-grams in the FST");
}

#[test]
fn test_fst_scores_are_reasonable() {
    let file = File::open(get_fst_path())
        .expect("Failed to open FST file. Set WIKI_NGRAM_FST_PATH env var or run 'cargo run -p wiki-ngram --release' first.");
    let mmap = unsafe { Mmap::map(&file) }
        .expect("Failed to mmap FST file");
    let fst = Map::new(mmap)
        .expect("Failed to load FST");
    
    // Test that scores are within a reasonable range
    // Scores should be positive integers representing log-frequencies
    let test_ngrams = vec!["東京 都", "日本 の", "こと が"];
    
    for ngram in test_ngrams {
        if let Some(score) = fst.get(ngram) {
            // Scores should be reasonable (not absurdly large)
            // Log-frequency scores typically range from 0 to a few thousand
            assert!(score < 1_000_000, "Score for '{}' seems unreasonably large: {}", ngram, score);
            println!("'{}': score = {}", ngram, score);
        }
    }
}

#[test]
fn test_fst_contains_entries() {
    let file = File::open(get_fst_path())
        .expect("Failed to open FST file. Set WIKI_NGRAM_FST_PATH env var or run 'cargo run -p wiki-ngram --release' first.");
    let mmap = unsafe { Mmap::map(&file) }
        .expect("Failed to mmap FST file");
    let fst = Map::new(mmap)
        .expect("Failed to load FST");
    
    // The FST should contain at least some entries
    let mut count = 0;
    let mut stream = fst.stream();
    for _ in 0..10 {
        if stream.next().is_some() {
            count += 1;
        } else {
            break;
        }
    }
    
    assert!(count > 0, "FST should contain at least some entries");
    println!("FST contains entries (verified at least {} entries)", count);
}

#[test]
fn test_fst_iteration() {
    let file = File::open(get_fst_path())
        .expect("Failed to open FST file. Set WIKI_NGRAM_FST_PATH env var or run 'cargo run -p wiki-ngram --release' first.");
    let mmap = unsafe { Mmap::map(&file) }
        .expect("Failed to mmap FST file");
    let fst = Map::new(mmap)
        .expect("Failed to load FST");
    
    // Test that we can iterate through the FST
    let mut total_entries = 0;
    let mut sample_entries = Vec::new();
    let mut stream = fst.stream();
    
    while let Some((key, value)) = stream.next() {
        total_entries += 1;
        
        // Collect first 5 entries as samples
        if total_entries <= 5 {
            if let Ok(key_str) = String::from_utf8(key.to_vec()) {
                sample_entries.push((key_str, value));
            }
        }
        
        // Don't iterate through everything, just verify iteration works
        if total_entries >= 1000 {
            break;
        }
    }
    
    assert!(total_entries > 0, "Should be able to iterate through FST entries");
    println!("Verified {} entries in FST", total_entries);
    println!("Sample entries:");
    for (key, value) in sample_entries {
        println!("  '{}': {}", key, value);
    }
}

#[test]
fn test_predictive_search() {
    let file = File::open(get_fst_path())
        .expect("Failed to open FST file. Set WIKI_NGRAM_FST_PATH env var or run 'cargo run -p wiki-ngram --release' first.");
    let mmap = unsafe { Mmap::map(&file) }
        .expect("Failed to mmap FST file");
    let fst = Map::new(mmap)
        .expect("Failed to load FST");
    
    // Test predictive search for "今日" (Today)
    // Should find "今日 は" (Today is/Today, topic marker)
    let prefix = "今日";
    let mut stream = fst.range().ge(prefix).into_stream();
    
    let mut found_prediction = false;
    while let Some((key, _)) = stream.next() {
        let s = std::str::from_utf8(key).unwrap();
        if !s.starts_with(prefix) {
            break;
        }
        
        // Check if we found a valid prediction (prefix + space + something)
        if s.starts_with("今日 ") && s.len() > "今日 ".len() {
            found_prediction = true;
            println!("Found prediction: {}", s);
            break;
        }
    }
    
    assert!(found_prediction, "Should find predictive candidates for '{}'", prefix);
}
