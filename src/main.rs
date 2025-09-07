use clap::Parser;
use std::path::PathBuf;

fn main() {
    let arg = Cli::parse();
    println!("Music Player!");
    println!("File path: {:?}", arg.path);
}

#[derive(Parser)]
struct Cli {
    /// 要读取的文件路径
    path: PathBuf,
}
