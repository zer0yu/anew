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

    // 创建集合存储已有行
    let mut existing_lines = IndexSet::new();
    let mut added_lines = Vec::new();
    
    // 如果提供了目标文件路径，读取目标文件内容
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
    
    // 处理从标准输入读取的内容
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    
    // 先读取所有内容到一个字符串
    let mut input = String::new();
    handle.read_to_string(&mut input)?;
    
    // 按照换行符分割输入字符串
    let lines_iter = input.split('\n');
    
    // 处理每一行
    for line_raw in lines_iter {
        let line = if args.trim {
            line_raw.trim().to_string()
        } else {
            line_raw.to_string()
        };
        
        // 跳过空行
        if line.is_empty() {
            continue;
        }
        
        // 如果这行是新的（不在 existing_lines 中），添加它
        if !existing_lines.contains(&line) {
            existing_lines.insert(line.clone());
            added_lines.push(line);
        }
    }
    
    // 如果请求排序，则对新添加的行进行排序
    if args.sort {
        added_lines.sort_by(|a, b| natsort::compare(a, b, false));
    }
    
    // 如果不是干运行模式，并且有目标文件，写入文件
    if !args.dry_run && args.filepath.is_some() {
        let filepath = args.filepath.as_ref().unwrap();
        
        // 确保目录存在
        if let Some(parent) = Path::new(filepath).parent() {
            fs::create_dir_all(parent)?;
        }
        
        // 根据是否是重写模式，选择写入方式
        if args.rewrite {
            // 重写模式：先收集所有行，排序（如果需要），然后写入文件
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
            // 追加模式：只追加新行
            
            // 检查文件是否存在且末尾是否缺少换行符
            let need_newline = if Path::new(filepath).exists() {
                // 读取文件最后一个字节检查是否为换行符
                match fs::read(filepath) {
                    Ok(content) if !content.is_empty() => {
                        // 检查最后一个字符是否为换行符
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
            
            // 如果需要，先添加一个换行符
            if need_newline {
                writeln!(writer)?;
            }
            
            for line in &added_lines {
                writeln!(writer, "{}", line)?;
            }
        }
    }
    
    // 如果不是安静模式，输出新行到标准输出
    if !args.quiet_mode {
        for line in added_lines {
            println!("{}", line);
        }
    }
    
    Ok(())
}

// 辅助函数：加载文件内容
fn load_file_content(filepath: &str) -> io::Result<Vec<String>> {
    let file = File::open(filepath)?;
    let reader = BufReader::new(file);
    let mut lines = Vec::new();
    
    for line in reader.lines() {
        lines.push(line?);
    }
    
    Ok(lines)
}
