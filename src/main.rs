use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};
use std::path::Path;
use std::str;

use clap::Parser;
use indexmap::IndexSet;
use anew::natsort;

#[derive(Parser, Debug)]
#[command(author = "zer0yu", version, about = "A tool for adding new lines to files, skipping duplicates", long_about = None)]
struct Options {
    #[arg(short, long, help = "Do not output new lines to stdout")]
    quiet_mode: bool,

    #[arg(short, long, help = "Sort lines (natsort)")]
    sort: bool,

    #[arg(short, long, help = "Trim whitespaces")]
    trim: bool,

    #[arg(
        short,
        long,
        help = "Rewrite existing destination file to remove duplicates"
    )]
    rewrite: bool,

    #[arg(
        short,
        long,
        help = "Do not write to file, only output what would be written"
    )]
    dry_run: bool,

    #[arg(help = "Destination file")]
    filepath: Option<String>,
}

fn main() -> io::Result<()> {
    let args = Options::parse();

    // Create a collection to store existing lines
    let mut existing_lines = IndexSet::new();
    let mut added_lines = Vec::new();
    
    // If a target file path is provided, read the target file content
    if let Some(filepath) = &args.filepath {
        if Path::new(filepath).exists() {
            if let Ok(lines) = load_file_content(filepath) {
                for line in lines {
                    let processed_line = if args.trim {
                        line.trim().to_string()
                    } else {
                        line
                    };
                    
                    existing_lines.insert(processed_line);
                }
            }
        }
    }
    
    // Process content read from standard input
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    
    // First read all content into a string
    let mut input = String::new();
    handle.read_to_string(&mut input)?;
    
    // Split the input string by newline character
    let lines_iter = input.split('\n');
    
    // Process each line
    for line_raw in lines_iter {
        let line = if args.trim {
            line_raw.trim().to_string()
        } else {
            line_raw.to_string()
        };
        
        // Skip empty lines
        if line.is_empty() {
            continue;
        }
        
        // If this line is new (not in existing_lines), add it
        if !existing_lines.contains(&line) {
            existing_lines.insert(line.clone());
            added_lines.push(line);
        }
    }
    
    // If sorting is requested, sort the newly added lines
    if args.sort {
        added_lines.sort_by(|a, b| natsort::compare(a, b, false));
    }
    
    // If not in dry run mode and there is a target file, write to the file
    if !args.dry_run && args.filepath.is_some() {
        let filepath = args.filepath.as_ref().unwrap();
        
        // Ensure directory exists
        if let Some(parent) = Path::new(filepath).parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Choose writing method based on whether it's rewrite mode
        if args.rewrite {
            // Rewrite mode: collect all lines, sort (if needed), then write to file
            let mut all_lines: Vec<_> = existing_lines.into_iter().collect();
            
            if args.sort {
                all_lines.sort_by(|a, b| natsort::compare(a, b, false));
            }
            
            let file = File::create(filepath)?;
            let mut writer = BufWriter::new(file);
            
            for line in all_lines {
                writeln!(writer, "{}", line)?;
            }
        } else {
            // Append mode: only append new lines
            
            // Check if file exists and if the end is missing a newline character
            let need_newline = if Path::new(filepath).exists() {
                // Read the last byte of the file to check if it's a newline character
                match fs::read(filepath) {
                    Ok(content) if !content.is_empty() => {
                        // Check if the last character is a newline
                        content.last().map_or(false, |&byte| byte != b'\n')
                    },
                    _ => false
                }
            } else {
                false
            };
            
            let file = OpenOptions::new()
                .write(true)
                .append(true)
                .create(true)
                .open(filepath)?;
            let mut writer = BufWriter::new(file);
            
            // If needed, add a newline first
            if need_newline {
                writeln!(writer)?;
            }
            
            for line in &added_lines {
                writeln!(writer, "{}", line)?;
            }
        }
    }
    
    // If not in quiet mode, output new lines to standard output
    if !args.quiet_mode {
        for line in added_lines {
            println!("{}", line);
        }
    }
    
    Ok(())
}

// Helper function: load file content
fn load_file_content(filepath: &str) -> io::Result<Vec<String>> {
    let file = File::open(filepath)?;
    let reader = BufReader::new(file);
    let mut lines = Vec::new();
    
    for line in reader.lines() {
        lines.push(line?);
    }
    
    Ok(lines)
}
