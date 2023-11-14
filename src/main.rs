use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, Write};
use std::{fs, str};

use clap::Parser;
use indexmap::IndexSet;

mod utils;

#[derive(Parser, Debug)]
#[command(author = "zer0yu", version, about = "A tool for adding new lines to files, skipping duplicates", long_about = None)]
struct Options {
    #[arg(short, long, help = "do not output new lines to stdout")]
    quiet_mode: bool,

    #[arg(short, long, help = "sort lines (natsort)")]
    sort: bool,

    #[arg(short, long, help = "trim whitespaces")]
    trim: bool,

    #[arg(
        short,
        long,
        help = "rewrite existing destination file to remove duplicates"
    )]
    rewrite: bool,

    #[arg(long, help = "do not write to file, only output what would be written")]
    dry_run: bool,

    #[arg(help = "destination file")]
    filepath: String,
}

fn main() -> io::Result<()> {
    let args = Options::parse();
    let filepath = &args.filepath;

    let mut file = OpenOptions::new()
        .append(true)
        .write(true)
        .create(true)
        .open(filepath)?;

    let mut lines = match load_file(&args) {
        Ok(value) => value,
        Err(err) => {
            eprintln!("failed to load file: {}", err);
            return Err(err);
        }
    };

    if !args.dry_run && args.rewrite {
        for line in lines.iter() {
            writeln!(file, "{}", line)?;
        }
    }

    let stdin = io::stdin();
    for stdin_line in stdin.lock().lines() {
        let stdin_line = stdin_line?;

        if should_add_line(&args, &lines, &stdin_line) {
            lines.insert(stdin_line.clone());

            if !args.quiet_mode {
                println!("{}", stdin_line);
            }

            if !args.dry_run {
                // 懒加载文件
                let mut f: Option<File> = None;
                if f.is_none() {
                    f = Some(
                        OpenOptions::new()
                            .append(true)
                            .write(true)
                            .create(true)
                            .open(filepath)
                            .expect("failed to open file"),
                    );
                }

                if let Some(mut file) = f {
                    writeln!(file, "{}", stdin_line)?;
                }
            }
        }
    }

    if args.sort && !args.dry_run {
        let mut f = OpenOptions::new()
            .append(false)
            .write(true)
            .create(false)
            .open(&args.filepath)?;

        lines.sort_by(|a, b| utils::natsort::compare(a, b, false));

        for line in lines.iter() {
            writeln!(f, "{}", line)?;
        }
    }

    Ok(())
}

fn load_file(args: &Options) -> Result<IndexSet<String>, io::Error> {
    let mut lines = IndexSet::new();
    match fs::read_to_string(&args.filepath) {
        Ok(data) => {
            for line in data.lines() {
                if should_add_line(args, &lines, &line) {
                    lines.insert(line.to_string());
                }
            }
        }
        Err(err) if err.kind() != io::ErrorKind::NotFound => {
            eprintln!("failed to open file for reading: {}", err);
            return Err(err);
        }
        _ => {}
    }

    Ok(lines)
}

fn should_add_line(args: &Options, lines: &IndexSet<String>, line: &str) -> bool {
    let trimmed_line = if args.trim { line.trim() } else { line };
    !trimmed_line.is_empty() && !lines.contains(trimmed_line)
}
