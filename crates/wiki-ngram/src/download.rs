use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::blocking::Client;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

const WIKIPEDIA_URL: &str = "https://dumps.wikimedia.org/jawiki/latest/jawiki-latest-pages-articles.xml.bz2";

pub fn download_wikipedia(cache_dir: &Path) -> Result<PathBuf> {
    fs::create_dir_all(cache_dir)?;
    
    let filename = "jawiki-latest-pages-articles.xml.bz2";
    let output_path = cache_dir.join(filename);

    // Check if already downloaded
    if output_path.exists() {
        log::info!("Wikipedia dump already cached at {:?}", output_path);
        return Ok(output_path);
    }

    log::info!("Downloading from {}", WIKIPEDIA_URL);
    
    let client = Client::new();
    let mut response = client.get(WIKIPEDIA_URL).send()?;
    
    let total_size = response
        .content_length()
        .ok_or_else(|| anyhow::anyhow!("Failed to get content length"))?;

    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")?
            .progress_chars("#>-"),
    );

    let mut file = File::create(&output_path)?;
    let mut downloaded = 0u64;
    let mut buffer = vec![0; 8192];

    loop {
        let bytes_read = response.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        file.write_all(&buffer[..bytes_read])?;
        downloaded += bytes_read as u64;
        pb.set_position(downloaded);

        // Log every 50MB for CI visibility
        if downloaded > 0 && downloaded % (50 * 1024 * 1024) < bytes_read as u64 {
            log::info!("Downloaded {} MB / {} MB", downloaded / 1024 / 1024, total_size / 1024 / 1024);
        }
    }

    pb.finish_with_message("Download complete");
    log::info!("Downloaded to {:?}", output_path);

    Ok(output_path)
}
