use anyhow::anyhow;
use anyhow::{Error, Ok, Result};
use clap::ArgMatches;
use colored::Colorize;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink};
use std::time::Duration;
use std::{
    collections::HashMap,
    fs::{self, DirEntry, File, read_dir},
    io::{self, BufReader, ErrorKind},
    path::Path,
    time::Instant,
};
use symphonia::core::conv::IntoSample;

/// 主结构体,代表CLI音乐播放器.
/// 维护状态`State`并处理所有播放操作`PlaybackOperation`
pub struct Player {
    /// 用于播放的音频接收器`Sink`
    sink: rodio::Sink,
    /// 音频输出流`OutputStream`和`OutputStreamHandle`
    stream_handle: OutputStream,
    /// 包含音频文件的目录`Directory`
    main_dir: Option<String>,
    /// 目录下所有音频的`HashMap`
    audio_list: Option<HashMap<u32, DirEntry>>,
    /// 当前播放的文件名
    current_audio: Option<String>,
    /// 当前播放的开始时间
    start_time: Option<Instant>,
    /*
    [是否正在播放]直接使用Sink自有方法.is_paused判断,无需手动维护状态
    is_playing: bool,
    */
    /// 是否开启循环
    is_loop: bool,
}

// 占位,暂时无用
/// 表示`Player`可处理地所有可能键盘事件`KeyEvent`
/// 对应播放操作`PlaybackOperation`的枚举
pub enum KeyAction {
    /// Plays a track
    Play,
    /// Pauses current track
    Paused,
    /// Stops playback
    Stop,
    /// Switch to the previous audio
    Back,
    /// Switch to the next audio
    Next,
    /// Loop single track
    SingleLoop,
    /// Loop playlist
    ListLoop,
    /// Shuffle playlist
    RandomLoop,
    /// Adjust the volume
    Volume(f32),
    /// Keys not within the monitoring range
    InvalidKey,
}

impl Player {
    pub fn new() -> Result<Self> {
        // 获取链接默认音频设备输出流和其句柄
        let stream_handle = OutputStreamBuilder::open_default_stream()?;
        // 创建一个接收器Sink
        let sink = rodio::Sink::connect_new(&stream_handle.mixer());
        sink.pause();
        Ok(Self {
            sink,
            stream_handle,
            audio_list: Some(HashMap::new()),
            // is_playing: false,
            is_loop: false,//占位
            main_dir: None,//占位
            current_audio: None,//占位
            start_time: None,//占位
        })
    }

    /// 运行播放器
    /// 处理初始化和命令解析
    pub fn run(&mut self, arg: ArgMatches) -> Result<()> {
        //
        let dir: &String = arg
            .get_one("music-dir")
            .ok_or_else(|| io::Error::new(ErrorKind::InvalidInput, "缺少音频目录!"))?;

        if !fs::metadata(dir)?.is_dir() {
            return Err(io::Error::new(ErrorKind::NotFound, "目录未找到!").into());
        }

        self.main_dir = Some(dir.to_string());
        self.load_audio()?;

        println!(
            "Found {} audio.",
            self.audio_list.as_ref().unwrap().len().to_string().yellow()
        );
        // 进入终端`raw mode`
        enable_raw_mode()?;

        // Test 目前只能固定播放第一首歌
        self.play(1)?;

        println!("{}", "\n空格=播放/暂停, q=退出\n".green());

        self.key_event();

        // 退出`raw mode`
        disable_raw_mode()?;
        println!("\nBye");

        Ok(())
    }

    /// 根据索引执行播放
    pub fn play(&mut self, audio_idx: u32) -> Result<()> {
        //
        if let Some(audio_map) = &self.audio_list {
            if let Some(audio) = audio_map.get(&audio_idx) {
                let file = BufReader::new(File::open(audio.path())?);
                let source = Decoder::new(file)?;
                self.sink.set_volume(1.0);
                self.sink.append(source);
                // self.is_playing = true;
                self.current_audio = Some(audio.file_name().to_string_lossy().to_string());
                self.start_time = Some(Instant::now());
                println!(
                    "{}: Playing {}",
                    "Now playing".green().bold(),
                    self.current_audio.as_ref().unwrap().blue()
                );
                Ok(())
            } else {
                Err(anyhow!("{}: 无效的音频索引", "Error".red()))
            }
        } else {
            Err(anyhow!("{}","未加载音频列表".red()))
        }
    }

    /// 监听键盘,控制播放
    pub fn key_event(&mut self) {
        loop {

            if event::poll(Duration::from_millis(200)).unwrap() {
                if let Event::Key(key) = event::read().unwrap() {
                    if key.kind!=KeyEventKind::Press {
                        continue;
                    }
                    match key.code {
                        KeyCode::Char(' ')=>{
                            if self.sink.is_paused() {
                                self.sink.play();
                            }else {
                                self.sink.pause();
                            }
                        }
                        KeyCode::Char('q')=>break,
                        _=>{}
                    }
                }
            }
            
        }
    }

    /// 过滤后加载音频列表
    pub fn load_audio(&mut self) -> Result<()> {
        let ext_list = ["mp3", "m4a", "flac", "aac", "wav", "ogg", "ape"];
        let mut index = 1;
        if let Some(dir) = &self.main_dir {
            if let Some(audio_map) = &mut self.audio_list {
                for entry in read_dir(dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(ext) = path.extension() {
                            if ext_list.contains(&ext.to_str().unwrap()) {
                                audio_map.insert(index, entry);
                                index += 1;
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }


}

#[cfg(test)]
mod tests {
    use super::*;
    pub fn open(path: &Path) -> String {
        if !fs::metadata(path).unwrap().is_dir() {
            let err = format!("Error");
            return err;
        }
        let ok = format!("Ok");
        ok
    }

    /// 测试扩展名过滤
    pub fn filter(path: &Path) -> i32 {
        let exts = ["mp3", "m4a", "flac", "aac", "wav", "ogg", "ape"];
        let mut counter = 0;
        for entry in read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                let ext = path.extension().unwrap();
                if exts.contains(&ext.to_str().unwrap()) {
                    counter += 1;
                }
            }
        }
        counter
    }
    #[test]
    fn it_works() {
        let path = Path::new("C:\\Users\\Admin\\Music");
        assert_eq!("Ok", open(path))
    }
    #[test]
    fn contains_ext() {
        // let path = Path::new("C:\\Users\\Admin\\Downloads\\mp3\\15");
        let path = Path::new("C:\\Users\\Admin\\Music");
        assert_eq!(2, filter(path))
    }
}
