use anyhow::Result;
use bzip2::read::BzDecoder;
use indicatif::{ProgressBar, ProgressStyle};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use vibrato::Tokenizer;

use crate::ngram::extract_ngrams_from_tokens;
use crate::tokenize::tokenize_text;

pub fn process_wikipedia(
    wiki_bz2_path: &Path,
    tokenizer: &Tokenizer,
    max_ngram: usize,
    limit: Option<usize>,
) -> Result<HashMap<String, usize>> {
    let file = File::open(wiki_bz2_path)?;
    let decoder = BzDecoder::new(BufReader::new(file));
    let buf_reader = BufReader::new(decoder);
    let mut reader = Reader::from_reader(buf_reader);
    reader.config_mut().trim_text(true);

    let mut ngram_counts: HashMap<String, usize> = HashMap::new();
    let mut buf = Vec::new();
    let mut in_text = false;
    let mut current_text = String::new();
    let mut article_count = 0;

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} [{elapsed_precise}] Articles: {pos} | N-grams: {msg}")?
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                if e.name().as_ref() == b"text" {
                    in_text = true;
                    current_text.clear();
                }
            }
            Ok(Event::End(ref e)) => {
                if e.name().as_ref() == b"text" && in_text {
                    in_text = false;
                    
                    // Process the extracted text
                    let clean_text = clean_wiki_markup(&current_text);
                    if !clean_text.is_empty() {
                        process_article(&clean_text, tokenizer, max_ngram, &mut ngram_counts);
                        article_count += 1;

                        if article_count % 1000 == 0 {
                            pb.set_position(article_count);
                            pb.set_message(format!("{}", ngram_counts.len()));
                        }

                        if let Some(l) = limit {
                            if article_count >= l as u64 {
                                break;
                            }
                        }
                    }
                }
            }
            Ok(Event::Text(e)) => {
                if in_text {
                    if let Ok(text) = e.unescape() {
                        current_text.push_str(&text);
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                log::warn!("XML parse error at position {}: {:?}", reader.buffer_position(), e);
                break;
            }
            _ => {}
        }
        buf.clear();
    }

    pb.finish_with_message(format!("Processed {} articles, {} unique N-grams", article_count, ngram_counts.len()));

    Ok(ngram_counts)
}

fn process_article(
    text: &str,
    tokenizer: &Tokenizer,
    max_ngram: usize,
    ngram_counts: &mut HashMap<String, usize>,
) {
    // Split into sentences (simple split by periods and newlines)
    for sentence in text.split(|c| c == '。' || c == '\n' || c == '.' || c == '！' || c == '？') {
        let sentence = sentence.trim();
        if sentence.len() < 3 {
            continue;
        }

        // Tokenize
        let tokens = tokenize_text(tokenizer, sentence);
        if tokens.len() < 2 {
            continue;
        }

        // Extract N-grams
        extract_ngrams_from_tokens(&tokens, max_ngram, ngram_counts);
    }
}

fn clean_wiki_markup(text: &str) -> String {
    let mut result = String::new();
    let mut chars = text.chars().peekable();
    let mut in_template = 0;
    let mut in_link = false;

    while let Some(ch) = chars.next() {
        match ch {
            '{' => {
                if chars.peek() == Some(&'{') {
                    chars.next();
                    in_template += 1;
                    continue;
                }
            }
            '}' => {
                if chars.peek() == Some(&'}') {
                    chars.next();
                    if in_template > 0 {
                        in_template -= 1;
                    }
                    continue;
                }
            }
            '[' => {
                if chars.peek() == Some(&'[') {
                    chars.next();
                    in_link = true;
                    continue;
                }
            }
            ']' => {
                if chars.peek() == Some(&']') {
                    chars.next();
                    in_link = false;
                    continue;
                }
            }
            '|' if in_link => {
                // Skip link prefix, keep display text
                continue;
            }
            _ => {}
        }

        if in_template == 0 && !in_link {
            result.push(ch);
        } else if in_link && ch != '[' && ch != ']' {
            // Keep link text but remove markup
            result.push(ch);
        }
    }

    result
}
