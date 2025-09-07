use std::{env, path::PathBuf};

fn main() {
    let path = env::args().nth(1).unwrap();
    let arg = Cli {
        path: PathBuf::from(path),
    };
    println!("Music Player!");
    println!("File path: {:?}", arg.path);
}
struct Cli {
    path: PathBuf,
}
