use crate::{AnyResult, anyhow};
use regex::Regex;
use std::{
    fs::{self, File},
    path::Path, time::Duration,
};
use symphonia::core::{
    formats::FormatOptions, io::MediaSourceStream, meta::StandardTagKey, probe::Hint,
};

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
        Err(_) => {
            // 未找到元数据
            // println!("{:?}", e);
            None
        }
    }
}

/// 从音频文件元数据或本地`.lrc`文件提取歌词
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

    // 3. 访问元数据, 遍历tag, 提取lyrics.或者从本地`.lrc`文件提取歌词
    match format.metadata().current() {
        Some(metsdata_rev) => {
            let mut tags = metsdata_rev.tags().iter();
            if let Some(tag_lrc) = tags.find(|tag| tag.std_key == Some(StandardTagKey::Lyrics)) {
                Ok(tag_lrc.value.to_string())
            } else {
                get_local_lrc(path)
            }
        }
        None => get_local_lrc(path),
    }
}

/// 从本地`.lrc`文件提取歌词
fn get_local_lrc(path: &Path) -> AnyResult<String> {
    let lrc_path = path.with_extension("lrc");
    if lrc_path.exists() {
        let lrc_content = fs::read_to_string(lrc_path)?;
        Ok(lrc_content)
    } else {
        Err(anyhow!("未找到歌词"))
    }
}
/// 解析LRC歌词文本

fn parse_lrc(lrc_text: &str) -> Vec<(Duration, String)> {
    // 这个正则表达式只用于匹配和捕获一个时间戳, 不包含后面的文本部分
    let timestamp_rex = Regex::new(r"\[(\d{2}):(\d{2})[.:](\d{2,3})\]").unwrap();
    let mut lyrics = Vec::new();

    for line in lrc_text.lines() {
        // 1. 找出当前行所有的歌词时间戳
        // 使用 captures_iter 来迭代所有匹配项
        let timestamps: Vec<Duration> = timestamp_rex
            .captures_iter(line)
            .filter_map(|caps| {
                // 解析分钟、秒和毫秒
                let minutes: u64 = caps.get(1)?.as_str().parse().ok()?;
                let seconds: u64 = caps.get(2)?.as_str().parse().ok()?;
                let millis_str = caps.get(3)?.as_str();
                let millis: u64 = if millis_str.len() == 2 {
                    // 如果是厘秒 (xx)，则乘以10转为毫秒
                    millis_str.parse().unwrap_or(0) * 10
                } else {
                    // 否则直接解析毫秒 (xxx)
                    millis_str.parse().unwrap_or(0)
                };
                Some(
                    Duration::from_millis(
                        
                        minutes * 60 * 1000 + seconds * 1000 + millis
                    )
                )
            })
            .collect();
        // 如果该行没有任何有效的时间戳 (例如元数据行 [ar: artist]) 则跳过
        if timestamps.is_empty() {
            continue;
        }
        // 2. 获取歌词文本
        // 文本是最后一个时间戳 `]` 之后的所有内容
        if let Some(last_bracket_pos) = line.rfind(']') {
            let text = line[last_bracket_pos + 1..].trim().to_string();
            // 3. 为每个时间戳创建一条歌词记录
            if !text.is_empty() {
                for time in timestamps {
                    // text.clone() 是必需的，因为文本内容需要在多个元组中共享
                    lyrics.push((time, text.clone()));
                }
            }
        }
    }
    lyrics.sort_by(|a, b| a.0.cmp(&b.0));
    lyrics
}
