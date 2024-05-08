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

    #[arg(help = "Destination file")]
    filepath: String,
}

fn main() -> io::Result<()> {
    let args = Options::parse();

    // Ensure the directories in the filepath exist before attempting to open the file
    if let Some(parent) = Path::new(&args.filepath).parent() {
        fs::create_dir_all(parent)?;
    }

    let mut lines = load_file(&args)?;

    if args.rewrite && !args.dry_run {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&args.filepath)?;
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
        .open(&args.filepath)?;
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

        let soet_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&args.filepath)?;
        let mut soet_writer = BufWriter::new(soet_file);

        for line in sorted_lines.iter() {
            writeln!(soet_writer, "{}", line)?;
        }
    }

    Ok(())
}

fn load_file(args: &Options) -> Result<IndexSet<String>, io::Error> {
    match File::open(&args.filepath) {
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
