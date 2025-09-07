use std::env;

fn main() {
    let args:Vec<String> = env::args().collect();
    let path = &args[1];
    println!("Music Player!");
    println!("File path: {}",path);
}
