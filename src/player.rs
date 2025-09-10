use anyhow::{Result, anyhow};
use clap::ArgMatches;
use colored::Colorize;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink, Source};
use std::io::Write;
use std::time::Duration;
use std::{
    collections::HashMap,
    fs::{self, DirEntry, File, read_dir},
    io::{self, BufReader, ErrorKind},
};

use crate::player::tool::clear_line;
mod tool;

/// 主结构体,代表CLI音乐播放器.
/// 维护状态`State`并处理所有播放操作`PlaybackOperation`
pub struct Player {
    /// 用于播放的音频接收器`Sink`
    sink: rodio::Sink,
    /// 音频输出流`OutputStream`和`OutputStreamHandle`
    stream_handle: OutputStream,
    /// 包含音频文件的目录`Directory`
    main_dir: String,
    /// 目录下所有音频的`HashMap`
    audio_list: Option<HashMap<u32, DirEntry>>,
    /// 当前选择的文件索引
    current_audio_idx: u32,
    /// 当前选择的文件名
    current_audio: String,
    /// 音频总数
    audio_total: u32,
    /// 当前播放的总时长
    total_time: String,
    /*
    [是否正在播放]直接使用Sink自有方法.is_paused判断,无需手动维护状态
    is_playing: bool,
    */
    /// 是否开启循环
    #[allow(unused)]
    is_loop: bool,
}

// 占位,暂时无用
/// 表示`Player`可处理地所有可能键盘事件`KeyEvent`
/// 对应播放操作`PlaybackOperation`的枚举
#[allow(unused)]
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
    /// 初始化Player新实例
    pub fn new() -> Result<Self> {
        // 获取链接默认音频设备输出流和其句柄
        let stream_handle = OutputStreamBuilder::open_default_stream()?;
        // 创建一个接收器Sink
        let sink = rodio::Sink::connect_new(&stream_handle.mixer());
        // sink.pause();
        Ok(Self {
            sink,
            stream_handle,
            main_dir: String::new(),
            total_time: String::new(),
            current_audio: String::new(),
            audio_list: Some(HashMap::new()),
            current_audio_idx: 1,
            audio_total: 0,
            is_loop: false, //占位
        })
    }

    /// 运行播放器
    /// 处理初始化和命令解析
    pub fn run(&mut self, arg: ArgMatches) -> Result<()> {
        //  验证目录参数是否正确
        let dir: &String = arg
            .get_one("music-dir")
            .ok_or_else(|| io::Error::new(ErrorKind::InvalidInput, "缺少音频目录!"))?;

        if !fs::metadata(dir)?.is_dir() {
            return Err(io::Error::new(ErrorKind::NotFound, "目录未找到!").into());
        }

        self.main_dir = dir.to_string();
        self.load_audio()?;
        let total = self.audio_list.as_ref().unwrap().len();
        self.audio_total = total as u32;
        println!("Found {} audio.", total.to_string().yellow());
        // 进入终端`raw mode`
        enable_raw_mode()?;
        //
        self.play()?;
        //
        self.key_event()?;

        // 退出`raw mode`
        disable_raw_mode()?;
        println!("\nBye");

        Ok(())
    }

    /// 根据索引执行播放
    pub fn play(&mut self) -> Result<()> {
        //
        if !self.sink.is_paused() {
            self.sink.stop();
            self.sink = Sink::connect_new(&self.stream_handle.mixer());
        } /* else {
        self.sink.clear();
        self.sink = Sink::connect_new(&self.stream_handle.mixer());
        self.sink.pause();
        } */
        //
        if let Some(audio_map) = &self.audio_list {
            if let Some(audio) = audio_map.get(&self.current_audio_idx) {
                let file = BufReader::new(File::open(audio.path())?);
                let source = Decoder::new(file)?;
                let src_time = source.total_duration().unwrap().as_secs();
                // 获取音频时长
                let src_minutes = src_time / 60;
                let src_seconds = src_time % 60;
                self.total_time = format!("{:02}:{:02}", src_minutes, src_seconds);

                self.sink.set_volume(1.0);
                self.sink.append(source);

                self.current_audio = audio.file_name().to_string_lossy().to_string();

                Ok(())
            } else {
                Err(anyhow!("{}: 无效的音频索引", "Error".red()))
            }
        } else {
            Err(anyhow!("{}", "未加载音频列表".red()))
        }
    }

    /// 监听键盘,控制播放
    pub fn key_event(&mut self) -> anyhow::Result<()> {
        loop {
            // 打印播放进度
            let minutes = self.sink.get_pos().as_secs() / 60;
            let seconds = self.sink.get_pos().as_secs() % 60;
            let now_time = format!("{:02}:{:02}", minutes, seconds);
            print!(
                "\r {}🎶 {} ⌛{}/{}",
                "Playing".green().bold(),
                self.current_audio.blue(),
                now_time.blue(),
                self.total_time.green()
            );
            io::stdout().flush()?;

            // 自动切歌
            if self.sink.empty() {
                if self.current_audio_idx == self.audio_total {
                    self.current_audio_idx = 1;
                    self.play()?;
                    clear_line()?;
                } else {
                    self.current_audio_idx += 1;
                    self.play()?;
                    clear_line()?;
                }
            }

            if event::poll(Duration::from_millis(200))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }
                    match key.code {
                        // 空格
                        KeyCode::Char(' ') => {
                            if self.sink.is_paused() {
                                self.sink.play();
                            } else {
                                self.sink.pause();
                            }
                        }
                        // 右方向键
                        KeyCode::Right => {
                            if self.current_audio_idx == self.audio_total {
                                self.current_audio_idx = 1
                            } else {
                                self.current_audio_idx += 1;
                            }
                            // self.sink.get_pos();
                            self.play()?;
                            clear_line()?;
                            // println!("sink len:{:?}",self.sink.get_pos())
                        }
                        // 左方向键
                        KeyCode::Left => {
                            if self.current_audio_idx == 1 {
                                self.current_audio_idx = self.audio_total;
                            } else {
                                self.current_audio_idx -= 1;
                            }

                            self.play()?;
                            clear_line()?;
                        }
                        KeyCode::Up => {}
                        KeyCode::Down => {}
                        // Esc
                        KeyCode::Esc => break,
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    /// 过滤后加载音频列表
    pub fn load_audio(&mut self) -> Result<()> {
        let ext_list = ["mp3", "m4a", "flac", "aac", "wav", "ogg", "ape"];
        //
        let mut index = 1;
        let dir = &self.main_dir;

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

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
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
