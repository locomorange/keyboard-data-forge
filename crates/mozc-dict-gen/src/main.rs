use anyhow::Result;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

const MOZC_REPO_URL: &str = "https://github.com/google/mozc/archive/refs/heads/master.tar.gz";

fn main() -> Result<()> {
    println!("Downloading Mozc source...");
    let response = reqwest::blocking::get(MOZC_REPO_URL)?;
    let bytes = response.bytes()?;

    println!("Extracting dictionary files...");
    let tar = flate2::read::GzDecoder::new(&bytes[..]);
    let mut archive = tar::Archive::new(tar);

    // Placeholder for extraction logic
    // In a real scenario, we would iterate over entries and extract specific files
    // archive.entries()?.filter_map(|e| e.ok())...

    println!("Converting to Vibrato format...");
    let output_dir = Path::new("output");
    if !output_dir.exists() {
        fs::create_dir(output_dir)?;
    }

    let mut file = File::create(output_dir.join("mozc.dict"))?;
    writeln!(file, "dummy vibrato dictionary")?;

    println!("Done.");
    Ok(())
}
