//! 工具模块
//!
//! 提供以下核心功能：
//! * 从音频元数据中提取歌词
//! * 解析LRC格式歌词文件
//! * 时间戳转换与排序
use crate::{AnyResult, anyhow};
use regex::Regex;
use std::fs::File;
use std::path::Path;
use std::time::Duration;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::StandardTagKey;
use symphonia::core::probe::Hint;

/// 从音频文件元数据提取歌词
///
/// # 依赖说明
/// 使用symphonia库解析音频元数据
///
/// # 返回值
/// * 成功返回歌词字符串
/// * 失败返回错误信息（未找到元数据）
fn get_lyrics(path: &Path) -> AnyResult<String> {
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
    //
    let metadata_opts = Default::default();

    // 探测媒体源的格式
    let probed =
        symphonia::default::get_probe().format(&hint, mss, &format_opts, &metadata_opts)?;

    let mut format = probed.format;

    // 3. 访问元数据, 遍历tag, 提取lyrics
    if let Some(metsdata_rev) = format.metadata().current() {
        for tag in metsdata_rev.tags() {
            // 优先检查标准 Key，以兼容其他写入器
            if let Some(StandardTagKey::Lyrics) = tag.std_key {
                return Ok(tag.value.to_string());
            } /*  else if &tag.key == "USLT" {
            return Ok(tag.value.to_string());
            }; */
        }
    }
    Err(anyhow!("未找到元数据"))
}

/// 解析LRC歌词文件
///
/// # 格式支持
/// 支持标准LRC格式及扩展时间戳：
/// [mm:ss.SS] 或 [mm:ss:SSS]
///
/// # 返回值
/// 返回排序后的(时间戳, 歌词)元组向量
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
    match get_lyrics(path) {
        Ok(lrc_string) => {
            let parsed = parse_lrc(&lrc_string);
            if parsed.is_empty() {
                None
            } else {
                Some(parsed)
            }
        }
        Err(e) => {
            // 未找到元数据
            println!("{:?}", e);
            None
        }
    }
}

/* pub fn get_metadata(path:&Path) -> AnyResult<()> {
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
    // let mut lyrics_found = false;

    // 3. 访问并打印元数据
    //
    if let Some(metadata_rev) = format.metadata().current() {
        //
        let tags = metadata_rev.tags();
        if tags.is_empty() {
            println!("未找到元数据标签。");
        } else {
            println!("找到的元数据标签:");
            for (i,tag) in tags.iter().enumerate() {
                println!(
                    "  标签 #{}: Key={:?}, StdKey={:?}, Value='{}...'",
                    i + 1,
                    tag.key,
                    tag.std_key,
                    tag.value.to_string().chars().take(70).collect::<String>()
                );

            }
        }
        // if !lyrics_found {
        //     println!("No Found Lricis");
        // }
    } else {
        println!("此文件没有元数据。");
    }

    Ok(())
} */
