#![allow(unused)]
use anyhow::Result;
use std::fs::File;
use std::io::{self, BufReader};
use std::path::{Path, PathBuf};
use anyhow::Context;
use crossterm::cursor::MoveToColumn;
use crossterm::execute;
use crossterm::terminal::{Clear, ClearType};
use symphonia::core::errors::Error;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::{MetadataOptions, StandardTagKey};
use symphonia::core::probe::Hint;

/// 清理之前的残留字符
pub fn clear_line()->Result<()>{
    execute!(
        io::stdout(),
        MoveToColumn(0), // 移动光标到行首
        Clear(ClearType::CurrentLine), // 清除当前行
    ).context("Failed to clear terminal line")?;
    Ok(())
}
pub fn get_metadata(path:&PathBuf) -> Result<(), Error> {
    // 1. 创建媒体源流
    let src = File::open(path).expect("无法打开文件");
    let mss = MediaSourceStream::new(Box::new(src), Default::default());

    // 2. 探测格式
    // 创建一个 Hint 来帮助探测。如果文件有扩展名，这会很有用。
    let mut hint = Hint::new();
    if let Some(extension) = path.extension().and_then(|s| s.to_str()) {
        hint.with_extension(extension);
    }

    // 默认的探测选项和元数据选项
    let format_opts: FormatOptions = Default::default();
    let metadata_opts: MetadataOptions = Default::default();

    // 探测媒体源的格式
    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &format_opts, &metadata_opts)?;

    let mut format = probed.format;

    //
    let mut lyrics_found = false;

    // 3. 访问并打印元数据
    // 优先使用最新的元数据或内嵌的元数据
    if let Some(metadata_rev) = format.metadata().current() {
        //
        let tags = metadata_rev.tags();
        if tags.is_empty() {
            println!("未找到元数据标签。");
        } else {
            println!("找到的元数据标签:");
            for tag in tags {
                // 打印Lricis
                if let Some(StandardTagKey::Lyrics) = tag.std_key {
                    println!("--- 歌词 ---");
                    println!("{}", tag.value);
                    println!("------------");
                    lyrics_found = true;
                    break;
                }
            }
        }
        if !lyrics_found {
            println!("No Found Lricis");
        }
    } else {
        println!("此文件没有元数据。");
    }

    Ok(())
}