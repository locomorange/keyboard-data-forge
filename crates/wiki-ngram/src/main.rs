use anyhow::Result;
use clap::Parser;
use fst::Streamer;
use std::fs::{self, File};
use std::io::BufWriter;
use std::path::{Path, PathBuf};

mod download;
mod extract;
mod ngram;
mod tokenize;

#[derive(Parser, Debug)]
#[command(name = "wiki-ngram")]
#[command(about = "Generate N-gram FST from Japanese Wikipedia for keyboard prediction")]
struct Args {
    /// Minimum frequency threshold (N-grams appearing ≤ this value will be filtered out)
    #[arg(long, default_value = "2")]
    min_frequency: usize,

    /// Maximum N-gram size (2=bigram, 3=trigram)
    #[arg(long, default_value = "3")]
    max_ngram: usize,

    /// Path to Vibrato dictionary (system.dic.zst from mozc-dict-gen)
    #[arg(long, default_value = "output/system.dic.zst")]
    dict_path: PathBuf,

    /// Output FST path
    #[arg(long, default_value = "output/wiki-ngrams.fst")]
    output: PathBuf,

    /// Download cache directory
    #[arg(long, default_value = "downloads")]
    download_cache: PathBuf,

    /// Run in dummy mode (for testing)
    #[arg(long)]
    dummy_mode: bool,

    /// Show FST statistics only
    #[arg(long)]
    stats: bool,

    /// Limit the number of articles to process (for debugging)
    #[arg(long)]
    limit: Option<usize>,
}

fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();

    if args.stats {
        return show_stats(&args.output);
    }

    if args.dummy_mode {
        return run_dummy_mode(&args.output);
    }

    log::info!("Starting Wikipedia N-gram FST generation");
    log::info!("Min frequency: {}", args.min_frequency);
    log::info!("Max N-gram: {}", args.max_ngram);

    // Ensure output directory exists
    if let Some(parent) = args.output.parent() {
        fs::create_dir_all(parent)?;
    }

    // Step 1: Download Wikipedia dump
    log::info!("Downloading Wikipedia dump...");
    let wiki_path = download::download_wikipedia(&args.download_cache)?;

    // Step 2: Load Vibrato tokenizer
    log::info!("Loading Vibrato dictionary from {:?}", args.dict_path);
    let tokenizer = tokenize::load_tokenizer(&args.dict_path)?;

    // Step 3: Extract text and tokenize
    log::info!("Extracting and tokenizing Wikipedia articles...");
    let ngram_counts = extract::process_wikipedia(&wiki_path, &tokenizer, args.max_ngram, args.limit)?;

    // Step 4: Filter and calculate log scores
    log::info!("Filtering N-grams (min frequency: {})...", args.min_frequency);
    let filtered = ngram::filter_ngrams(&ngram_counts, args.min_frequency);
    
    log::info!("Total N-grams after filtering: {}", filtered.len());

    // Step 5: Build FST
    log::info!("Building FST...");
    ngram::build_fst(&filtered, &args.output)?;

    log::info!("FST generated at {:?}", args.output);
    log::info!("Done!");

    Ok(())
}

fn run_dummy_mode(output_path: &Path) -> Result<()> {
    println!("Running in dummy mode...");
    
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Create a simple dummy FST with a few entries
    let mut data = vec![
        ("今日 は".to_string(), 1000),
        ("こんにちは 世界".to_string(), 800),
        ("おはよう ございます".to_string(), 600),
    ];
    data.sort_by(|a, b| a.0.cmp(&b.0));

    let mut builder = fst::MapBuilder::new(BufWriter::new(File::create(output_path)?))?;
    for (key, value) in data {
        builder.insert(key, value)?;
    }
    builder.finish()?;

    println!("Dummy FST created at {:?}", output_path);
    Ok(())
}

fn show_stats(fst_path: &Path) -> Result<()> {
    let file = File::open(fst_path)?;
    let mmap = unsafe { memmap2::Mmap::map(&file)? };
    let fst = fst::Map::new(mmap)?;

    println!("FST Statistics:");
    println!("  Total entries: {}", fst.len());
    
    // Sample first 10 entries
    println!("\nSample entries:");
    let mut count = 0;
    let mut stream = fst.stream();
    while let Some((key, value)) = stream.next() {
        if count >= 10 {
            break;
        }
        let key_str = String::from_utf8_lossy(key);
        println!("  {} => {}", key_str, value);
        count += 1;
    }

    Ok(())
}
