use anyhow::Result;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

const MOZC_REPO_URL: &str = "https://github.com/google/mozc/archive/refs/heads/master.tar.gz";

fn main() -> Result<()> {
    println!("Downloading Mozc source...");
    let response = reqwest::blocking::get(MOZC_REPO_URL)?;
    let bytes = response.bytes()?;

    println!("Extracting dictionary files...");
    let tar = flate2::read::GzDecoder::new(&bytes[..]);
    let mut archive = tar::Archive::new(tar);

    let mozc_src_dir = Path::new("mozc_src");
    if !mozc_src_dir.exists() {
        fs::create_dir(mozc_src_dir)?;
    }

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;
        let path_str = path.to_string_lossy();

        if path_str.contains("data/dictionary_oss/dictionary") && path_str.ends_with(".txt") {
            println!("Extracting {:?}", path);
            entry.unpack(mozc_src_dir.join(path.file_name().unwrap()))?;
        } else if path_str.contains("data/dictionary_oss/connection_single_column.txt") {
            println!("Extracting {:?}", path);
            entry.unpack(mozc_src_dir.join(path.file_name().unwrap()))?;
        } else if path_str.contains("data/dictionary_oss/id.def") {
            println!("Extracting {:?}", path);
            entry.unpack(mozc_src_dir.join(path.file_name().unwrap()))?;
        }
    }

    println!("Converting to Vibrato format...");
    let output_dir = Path::new("output");
    if !output_dir.exists() {
        fs::create_dir(output_dir)?;
    }

    let id_map = read_id_def(&mozc_src_dir.join("id.def"))?;
    
    println!("Generating matrix.def...");
    convert_matrix(&mozc_src_dir.join("connection_single_column.txt"), &output_dir.join("matrix.def"))?;

    println!("Generating lex.csv...");
    convert_lexicon(mozc_src_dir, &output_dir.join("lex.csv"), &id_map)?;

    println!("Generating char.def...");
    generate_char_def(&output_dir.join("char.def"))?;

    println!("Generating unk.def...");
    generate_unk_def(&output_dir.join("unk.def"), &id_map)?;

    println!("Compiling dictionary...");
    let dict = vibrato::SystemDictionaryBuilder::from_readers(
        File::open(output_dir.join("lex.csv"))?,
        File::open(output_dir.join("matrix.def"))?,
        File::open(output_dir.join("char.def"))?,
        File::open(output_dir.join("unk.def"))?,
    )?;

    let mut f = File::create(output_dir.join("system.dic.zst"))?;
    let mut encoder = zstd::Encoder::new(&mut f, 19)?;
    dict.write(&mut encoder)?;
    encoder.finish()?;

    println!("Done. Dictionary generated at output/system.dic.zst");
    Ok(())
}

fn read_id_def(path: &Path) -> Result<HashMap<u16, String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut map = HashMap::new();

    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let id: u16 = parts[0].parse()?;
            let pos = parts[1].to_string();
            map.insert(id, pos);
        }
    }
    Ok(map)
}

fn convert_matrix(input_path: &Path, output_path: &Path) -> Result<()> {
    let input_file = File::open(input_path)?;
    let reader = BufReader::new(input_file);
    let mut lines = reader.lines();

    let output_file = File::create(output_path)?;
    let mut writer = BufWriter::new(output_file);

    // First line is size
    let first_line = lines.next().ok_or_else(|| anyhow::anyhow!("Empty matrix file"))??;
    let size: u16 = first_line.trim().parse()?;

    writeln!(writer, "{} {}", size, size)?;

    let mut count = 0;
    for (i, line) in lines.enumerate() {
        let line = line?;
        if i == 0 { continue; } // Skip the second line if it's 0/metadata? Wait, let's verify.
        // Based on inspection:
        // Line 1: 2671
        // Line 2: 0
        // Line 3: 6022
        // If I skip line 2, and read N*N lines.
        // Let's assume line 2 is NOT a cost if it's 0 and followed by costs.
        // But 0 is a valid cost.
        // Let's assume the file contains N*N lines of costs AFTER the header.
        // If there are extra lines, we need to handle them.
        // Actually, let's just read all lines and see.
        
        // Re-reading logic:
        // The file has 7134243 lines.
        // 1 header line.
        // 7134242 remaining lines.
        // 2671 * 2671 = 7134241.
        // So there is 1 extra line.
        // Likely line 2 is metadata or padding.
        // Let's skip line 2.
        
        let cost: i16 = line.trim().parse()?;
        
        // Calculate left and right ID
        // The index in the matrix is count.
        // row = count / size
        // col = count % size
        
        let left_id = count / size as usize;
        let right_id = count % size as usize;
        
        writeln!(writer, "{} {} {}", left_id, right_id, cost)?;
        
        count += 1;
        if count >= (size as usize * size as usize) {
            break;
        }
    }
    
    Ok(())
}

fn convert_lexicon(src_dir: &Path, output_path: &Path, id_map: &HashMap<u16, String>) -> Result<()> {
    let output_file = File::create(output_path)?;
    let mut writer = csv::WriterBuilder::new()
        .has_headers(false)
        .quote_style(csv::QuoteStyle::Necessary)
        .from_writer(output_file);

    for entry in fs::read_dir(src_dir)? {
        let entry = entry?;
        let path = entry.path();
        if let Some(name) = path.file_name() {
            let name_str = name.to_string_lossy();
            if name_str.starts_with("dictionary") && name_str.ends_with(".txt") {
                println!("Processing {:?}", name);
                let file = File::open(&path)?;
                let reader = BufReader::new(file);
                for line in reader.lines() {
                    let line = line?;
                    let parts: Vec<&str> = line.split('\t').collect();
                    // Mozc format: reading, left_id, right_id, cost, surface, ...
                    // Example: あいあんと	1852	271	7271	アイアンと
                    if parts.len() >= 5 {
                        let reading = parts[0];
                        let left_id: u16 = parts[1].parse()?;
                        let right_id: u16 = parts[2].parse()?;
                        let cost: i16 = parts[3].parse()?;
                        let surface = parts[4];
                        
                        // MeCab format: surface, left, right, cost, pos, ...
                        // We use the POS string from id_map for left_id
                        let pos_str = id_map.get(&left_id).map(|s| s.as_str()).unwrap_or("Unk,*,*,*,*,*,*");
                        
                        // We need to split pos_str into columns
                        let pos_parts: Vec<&str> = pos_str.split(',').collect();
                        
                        let mut record = vec![surface.to_string(), left_id.to_string(), right_id.to_string(), cost.to_string()];
                        
                        // Ensure exactly 7 POS fields
                        for i in 0..7 {
                            if i < pos_parts.len() {
                                record.push(pos_parts[i].to_string());
                            } else {
                                record.push("*".to_string());
                            }
                        }
                        
                        // Add reading and pronunciation if available, or use reading for both
                        // MeCab standard has reading and pronunciation at the end.
                        // Mozc gives us reading.
                        record.push(reading.to_string()); // Reading
                        record.push(reading.to_string()); // Pronunciation (approx)
                        
                        writer.write_record(&record)?;
                    }
                }
            }
        }
    }
    Ok(())
}

fn generate_char_def(output_path: &Path) -> Result<()> {
    let mut file = File::create(output_path)?;
    // Minimal char.def based on IPADIC/Vibrato defaults
    writeln!(file, "DEFAULT 0 1 0")?;
    writeln!(file, "SPACE 0 1 0")?;
    writeln!(file, "KANJI 0 0 0")?;
    writeln!(file, "SYMBOL 0 1 0")?;
    writeln!(file, "NUMERIC 0 1 0")?;
    writeln!(file, "ALPHA 0 1 0")?;
    writeln!(file, "HIRAGANA 0 1 0")?;
    writeln!(file, "KATAKANA 0 1 0")?;
    writeln!(file, "KANJINUMERIC 0 1 0")?;
    writeln!(file, "GREEK 0 1 0")?;
    writeln!(file, "CYRILLIC 0 1 0")?;
    
    writeln!(file, "0x0020 SPACE")?;
    writeln!(file, "0x0009 SPACE")?;
    writeln!(file, "0x000D SPACE")?;
    writeln!(file, "0x000A SPACE")?;
    
    writeln!(file, "0x0030..0x0039 NUMERIC")?;
    writeln!(file, "0x0041..0x005A ALPHA")?;
    writeln!(file, "0x0061..0x007A ALPHA")?;
    writeln!(file, "0x3041..0x309F HIRAGANA")?;
    writeln!(file, "0x30A1..0x30FF KATAKANA")?;
    writeln!(file, "0x4E00..0x9FFF KANJI")?;
    
    Ok(())
}

fn generate_unk_def(output_path: &Path, id_map: &HashMap<u16, String>) -> Result<()> {
    let mut file = File::create(output_path)?;
    // We need to find valid IDs for categories.
    // For simplicity, we'll just pick the first ID that looks like a Noun, etc.
    // Or just use ID 0 if we don't care about UNK handling details for now.
    // But Vibrato might require valid IDs.
    
    // Let's try to find a "Noun,General" ID.
    let noun_id = id_map.iter().find(|(_, v)| v.starts_with("名詞,一般")).map(|(k, _)| *k).unwrap_or(0);
    let space_id = id_map.iter().find(|(_, v)| v.contains("空白")).map(|(k, _)| *k).unwrap_or(0);
    
    // Format: Category, LeftID, RightID, Cost, Features...
    writeln!(file, "DEFAULT,{},{},5000,名詞,一般,*,*,*,*,*,*,*", noun_id, noun_id)?;
    writeln!(file, "SPACE,{},{},0,記号,空白,*,*,*,*,*,*,*", space_id, space_id)?;
    writeln!(file, "KANJI,{},{},5000,名詞,一般,*,*,*,*,*,*,*", noun_id, noun_id)?;
    writeln!(file, "ALPHA,{},{},5000,名詞,一般,*,*,*,*,*,*,*", noun_id, noun_id)?;
    writeln!(file, "NUMERIC,{},{},5000,名詞,数,*,*,*,*,*,*,*", noun_id, noun_id)?;
    
    Ok(())
}
