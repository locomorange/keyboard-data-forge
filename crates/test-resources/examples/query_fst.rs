use anyhow::Result;
use fst::{IntoStreamer, Map, Streamer};
use memmap2::Mmap;
use std::env;
use std::fs::File;
use std::path::Path;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: cargo run -p test-resources --example query_fst -- <prefix> [limit]");
        return Ok(());
    }

    let prefix = &args[1];
    let limit = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(20);
    let fst_path = "output/wiki-ngrams.fst";

    if !Path::new(fst_path).exists() {
        eprintln!("Error: {} not found. Run wiki-ngram first.", fst_path);
        return Ok(());
    }

    println!("Querying FST for prefix: '{}' (limit: {})", prefix, limit);

    let file = File::open(fst_path)?;
    let mmap = unsafe { Mmap::map(&file)? };
    let map = Map::new(mmap)?;

    let mut stream = map.range().ge(prefix).into_stream();
    let mut entries = Vec::new();

    while let Some((key, value)) = stream.next() {
        let s = std::str::from_utf8(key)?;
        if !s.starts_with(prefix) {
            break;
        }
        entries.push((s.to_string(), value));
    }

    // Sort by score (descending)
    entries.sort_by(|a, b| b.1.cmp(&a.1));

    println!("Found {} matches", entries.len());
    println!("Top {} results:", limit);
    println!("{:<4} | {:<20} | {:<10} | {:<10}", "Rank", "N-gram", "LogScore", "ApproxFreq");
    println!("{:-<4}-+-{:-<20}-+-{:-<10}-+-{:-<10}", "", "", "", "");

    for (i, (key, value)) in entries.iter().take(limit).enumerate() {
        // Score is log(freq) * 1000. Convert back to approx freq for display
        let approx_freq = (value.clone() as f64 / 1000.0).exp() as u64;
        println!("{:<4} | {:<20} | {:<10} | {:<10}", i + 1, key, value, approx_freq);
    }

    Ok(())
}
