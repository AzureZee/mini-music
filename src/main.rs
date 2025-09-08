use clap::Parser;
use rodio::OutputStreamBuilder;
#[allow(unused_imports)]
use std::{fs::File, io::BufReader, path::PathBuf, thread, time};

fn main() {
    println!("Music Player!");
    let arg = Cli::parse();
    println!("File path: {:?}", arg.path);

    // 获取对物理设备的输出流句柄
    let stream_handle = OutputStreamBuilder::open_default_stream().unwrap();
    let file = BufReader::new(File::open(arg.path).unwrap());
    // 创建一个新的接收器，并在流上开始播放。
    let sink = rodio::play(&stream_handle.mixer(), file).unwrap();
    sink.sleep_until_end();
    // thread::sleep(time::Duration::from_secs(10));
}

#[derive(Parser)]
struct Cli {
    /// 要读取的文件路径
    path: PathBuf,
}
