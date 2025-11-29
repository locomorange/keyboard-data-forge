use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.contains(&"--dummy-mode".to_string()) {
        println!("Running in dummy mode...");
        let output_dir = Path::new("output");
        if !output_dir.exists() {
            fs::create_dir(output_dir)?;
        }
        
        let mut file = File::create(output_dir.join("wiki.fst"))?;
        writeln!(file, "dummy fst data")?;
        println!("Dummy FST created at output/wiki.fst");
    } else {
        println!("Real mode not implemented yet.");
    }
    
    Ok(())
}
