use clap::Parser;
use rodio::{Decoder, OutputStreamBuilder, Sink};
use std::{fs::File, path::PathBuf, thread, time};

fn main() {
    println!("Music Player!");
    let arg = Cli::parse();
    println!("File path: {:?}", arg.path);

    // 获取对物理设备的输出流句柄
    let stream_handle = OutputStreamBuilder::open_default_stream().unwrap();
    let file = File::open(arg.path).unwrap();
    // 创建一个新的接收器，并在流上开始播放。
    let _sink = Sink::connect_new(&stream_handle.mixer());
    let source = Decoder::try_from(file).unwrap();
    stream_handle.mixer().add(source);

    thread::sleep(time::Duration::from_secs(10));
}

#[derive(Parser)]
struct Cli {
    /// 要读取的文件路径
    path: PathBuf,
}
