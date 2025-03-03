use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
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

    #[arg(
        short = 'u',
        long = "unique",
        help = "Merge all input files, sort and remove duplicates (like 'sort -u')"
    )]
    unique_mode: bool,

    #[arg(help = "Destination file", required_unless_present = "unique_mode")]
    filepath: Option<String>,

    #[arg(help = "Additional input files for unique mode", num_args = 0..)]
    additional_files: Vec<String>,
}

fn main() -> io::Result<()> {
    let args = Options::parse();

    if args.unique_mode {
        // Handle unique mode (sort -u equivalent)
        let mut all_lines = IndexSet::new();
        
        // Process files if provided
        if let Some(filepath) = &args.filepath {
            if let Ok(lines) = load_file_content(filepath) {
                for line in lines {
                    if should_add_line(&args, &all_lines, &line) {
                        all_lines.insert(line);
                    }
                }
            }
        }

        // Process additional files
        for file in &args.additional_files {
            if let Ok(lines) = load_file_content(file) {
                for line in lines {
                    if should_add_line(&args, &all_lines, &line) {
                        all_lines.insert(line);
                    }
                }
            }
        }

        // Process stdin
        let stdin = io::stdin();
        let mut handle = stdin.lock();
        let mut buffer = String::new();
        
        while handle.read_line(&mut buffer)? > 0 {
            let line = buffer.trim_end().to_string();
            if should_add_line(&args, &all_lines, &line) {
                all_lines.insert(line);
            }
            buffer.clear();
        }

        // Sort if requested
        let mut final_lines: Vec<_> = all_lines.into_iter().collect();
        if args.sort {
            final_lines.sort_by(|a, b| natsort::compare(a, b, false));
        }

        // Output results
        for line in final_lines {
            println!("{}", line);
        }

        return Ok(());
    }

    // Original functionality for non-unique mode
    let filepath = args.filepath.as_ref().expect("Destination file is required in non-unique mode");
    
    // Ensure the directories in the filepath exist before attempting to open the file
    if let Some(parent) = Path::new(filepath).parent() {
        fs::create_dir_all(parent)?;
    }

    let mut lines = load_file(&args)?;

    if args.rewrite && !args.dry_run {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(filepath)?;
        let mut writer = BufWriter::new(file);

        for line in lines.iter() {
            writeln!(writer, "{}", line)?;
        }
    }

    let stdin = io::stdin();
    let file = OpenOptions::new()
        .append(true)
        .write(true)
        .create(true)
        .open(filepath)?;
    let mut writer = BufWriter::new(file);

    for stdin_line in stdin.lock().lines() {
        let stdin_line = stdin_line?;

        if should_add_line(&args, &lines, &stdin_line) {
            lines.insert(stdin_line.clone());

            if !args.quiet_mode {
                println!("{}", stdin_line);
            }

            if !args.sort && !args.dry_run {
                writeln!(writer, "{}", stdin_line)?;
            }
        }
    }

    if args.sort && !args.dry_run {
        let mut sorted_lines: Vec<_> = lines.into_iter().collect();
        sorted_lines.sort_by(|a, b| natsort::compare(a, b, false));

        let sort_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(filepath)?;
        let mut sort_writer = BufWriter::new(sort_file);

        for line in sorted_lines.iter() {
            writeln!(sort_writer, "{}", line)?;
        }
    }

    Ok(())
}

fn load_file(args: &Options) -> Result<IndexSet<String>, io::Error> {
    let filepath = args.filepath.as_ref().expect("Destination file is required");
    match File::open(filepath) {
        Ok(file) => {
            let reader = BufReader::new(file);
            let mut lines = IndexSet::new();

            for line in reader.lines() {
                let line = line?;
                if should_add_line(args, &lines, &line) {
                    lines.insert(line);
                }
            }

            Ok(lines)
        }
        Err(_) => Ok(IndexSet::new()), // If the file does not exist, return an empty set of lines
    }
}

fn should_add_line(args: &Options, lines: &IndexSet<String>, line: &str) -> bool {
    let trimmed_line = if args.trim { line.trim() } else { line };
    !trimmed_line.is_empty() && !lines.contains(trimmed_line)
}

// New helper function to load file content
fn load_file_content(filepath: &str) -> io::Result<Vec<String>> {
    match File::open(filepath) {
        Ok(file) => {
            let reader = BufReader::new(file);
            let mut lines = Vec::new();
            for line in reader.lines() {
                lines.push(line?);
            }
            Ok(lines)
        }
        Err(_) => Ok(Vec::new()),
    }
}
