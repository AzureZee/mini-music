// #![allow(unused)]
use anyhow::{Result, anyhow};
use regex::Regex;
use std::fs::File;
use std::path::Path;
use std::time::Duration;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::{MetadataOptions, StandardTagKey};
use symphonia::core::probe::Hint;

/// 从音频文件元数据中提取原始歌词字符串
fn get_lyrics(path: &Path) -> Result<String> {
    // 1. 创建媒体源流
    let src = File::open(path)?;
    let mss = MediaSourceStream::new(Box::new(src), Default::default());
    // 2. 探测格式
    // 创建一个 Hint 来帮助探测。如果文件有扩展名，这会很有用。
    let mut hint = Hint::new();
    if let Some(extension) = path.extension().and_then(|s| s.to_str()) {
        hint.with_extension(extension);
    }
    //
    // 默认的探测选项和元数据选项
    let format_opts: FormatOptions = Default::default();
    let metadata_opts: MetadataOptions = Default::default();
    // 探测媒体源的格式
    let probed =
        symphonia::default::get_probe().format(&hint, mss, &format_opts, &metadata_opts)?;
    let mut format = probed.format;
    // 3. 访问元数据, 遍历tag, 提取lyrics
    if let Some(metsdata_rev) = format.metadata().current() {
        for tag in metsdata_rev.tags() {
            if let Some(StandardTagKey::Lyrics) = tag.std_key {
                return Ok(tag.value.to_string());
            }
        }
    }
    Err(anyhow!("未找到歌词"))
}

/// 解析LRC格式的歌词字符串
/// 返回一个元组向量 `(时间点, 歌词文本)`，并按时间排序
fn parse_lrc(lrc_text: &str) -> Vec<(Duration, String)> {
    let rex = Regex::new(r"\[(\d{2}):(\d{2})[.:](\d{2,3})\](.*)").unwrap();
    let mut lyrics = Vec::new();

    for line in lrc_text.lines() {
        if let Some(caps) = rex.captures(line) {
            let minutes: u64 = caps[1].parse().unwrap_or(0);
            let seconds: u64 = caps[2].parse().unwrap_or(0);
            let millis_str = &caps[3];
            let millis: u64 = if millis_str.len() == 2 {
                // 如果是厘秒, 转为毫秒
                millis_str.parse().unwrap_or(0) * 10 
            } else {
                millis_str.parse().unwrap_or(0)
            };

            let time = Duration::from_millis(minutes * 60 * 1000 + seconds * 1000 + millis);
            let text = caps[4].trim().to_string();
            if !text.is_empty() {
                lyrics.push((time, text));
            }
        }
    }
    lyrics.sort_by(|a, b| a.0.cmp(&b.0));
    lyrics
}

///  加载并解析一个音频文件的歌词
pub fn load_and_parse_lrc(path: &Path) -> Option<Vec<(Duration, String)>> {
    if let Ok(lrc_string) = get_lyrics(path) {
        let parsed = parse_lrc(&lrc_string);
        if parsed.is_empty() {
            None
        } else {
            Some(parsed)
        }
    } else {
        None
    }
}
/* pub fn get_metadata(path:&PathBuf) -> Result<(), Error> {
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
} */
