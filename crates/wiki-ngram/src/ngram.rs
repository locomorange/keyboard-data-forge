use anyhow::Result;
use fst::MapBuilder;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

pub fn extract_ngrams_from_tokens(
    tokens: &[String],
    max_ngram: usize,
    ngram_counts: &mut HashMap<String, usize>,
) {
    // Extract bigrams (n=2) and trigrams (n=3)
    for n in 2..=max_ngram {
        if tokens.len() < n {
            continue;
        }

        for window in tokens.windows(n) {
            let ngram = window.join(" ");
            *ngram_counts.entry(ngram).or_insert(0) += 1;
        }
    }
}

pub fn prune_ngrams(ngram_counts: &mut HashMap<String, usize>, threshold_size: usize) {
    if ngram_counts.len() <= threshold_size {
        return;
    }

    log::info!("Pruning N-grams... (Current size: {})", ngram_counts.len());
    
    // Remove entries with frequency 1
    // This is a simple heuristic to save memory. 
    // Ideally we would use a more sophisticated approach like Count-Min Sketch,
    // but for this use case, removing hapax legomena (once-occurring ngrams) is usually safe enough
    // as we only care about high-frequency patterns for prediction.
    let before_len = ngram_counts.len();
    ngram_counts.retain(|_, &mut count| count > 1);
    let after_len = ngram_counts.len();
    
    log::info!("Pruned {} entries. New size: {}", before_len - after_len, after_len);
}

pub fn filter_ngrams(
    ngram_counts: &HashMap<String, usize>,
    min_frequency: usize,
) -> Vec<(String, u64)> {
    let mut filtered: Vec<(String, u64)> = ngram_counts
        .iter()
        .filter(|(_, &count)| count > min_frequency)
        .map(|(ngram, &count)| {
            // Calculate log score: ln(count) * 1000 for precision
            let log_score = (count as f64).ln() * 1000.0;
            (ngram.clone(), log_score as u64)
        })
        .collect();

    // Sort by key for FST insertion (required by fst::MapBuilder)
    filtered.sort_by(|a, b| a.0.cmp(&b.0));

    filtered
}

pub fn build_fst(data: &[(String, u64)], output_path: &Path) -> Result<()> {
    let file = File::create(output_path)?;
    let writer = BufWriter::new(file);
    let mut builder = MapBuilder::new(writer)?;

    for (key, value) in data {
        builder.insert(key.as_bytes(), *value)?;
    }

    builder.finish()?;
    log::info!("FST built with {} entries", data.len());

    Ok(())
}
