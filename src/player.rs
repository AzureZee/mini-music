use crate::{AnyResult,anyhow};
use clap::ArgMatches;
use colored::Colorize;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode};
use crossterm::{cursor, execute};
use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink, Source};
use std::io::Write;
use std::process::exit;
use std::time::Duration;
use std::{
    collections::HashMap,
    fs::{self, DirEntry, File, read_dir},
    io::{self, BufReader, ErrorKind},
};

use tool::load_and_parse_lrc;
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
    /// 解析后的当前歌曲歌词列表 (时间, 文本)
    lyrics: Option<Vec<(Duration, String)>>,
    /// 当前显示的歌词行
    current_lrc: String,
}


/// 表示`Player`可处理地所有可能键盘事件`KeyEvent`
/// 所对应的播放操作`PlaybackOperation`的枚举
// #[allow(unused)]
pub enum Operation {
    //
    TogglePaused,
    /// Switch to the previous audio
    Back,
    /// Switch to the next audio
    Next,
    //
    Exit,
}

impl Player {
    /// 初始化Player新实例
    pub fn new() -> AnyResult<Self> {
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
            lyrics: None,
            current_lrc: String::new(),

        })
    }

    /// 运行播放器
    /// 处理初始化和命令解析
    pub fn run(&mut self, arg: ArgMatches) -> AnyResult<()> {
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
        //
        self.play()?;
        //
        self.run_event_loop()?;

        println!("\nBye");

        Ok(())
    }

    /// 根据索引执行播放
    pub fn play(&mut self) -> AnyResult<()> {
        //  切换前清空并新建Sink
        if !self.sink.is_paused() {
            self.sink.stop();
            self.sink = Sink::connect_new(&self.stream_handle.mixer());
        } else {
        self.sink.clear();
        self.sink = Sink::connect_new(&self.stream_handle.mixer());
        self.sink.pause();
        }
        //
        if let Some(audio_map) = &self.audio_list {
            if let Some(audio) = audio_map.get(&self.current_audio_idx) {
                // -- 在这里加载歌词 --
                // 每次播放新歌曲时，先清空旧歌词
                self.lyrics = None;
                self.current_lrc = "".to_string();
                // 尝试加载并解析歌词
                self.lyrics = load_and_parse_lrc(&audio.path());
                // -- 歌词加载结束 --

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

    fn update_ui(&mut self) ->AnyResult<()>{
        // --- 1. 数据准备 ---
        // -- 歌词更新逻辑 --
        // 获取当前播放位置 self.sink.get_pos()
        let current_pos = self.sink.get_pos();
        // 默认无歌词
        let mut lrc_to_display = "".to_string();
        // 查找当前应显示的歌词
        if let Some(lyrics) = &self.lyrics {
            // 查找最后一个时间点小于等于当前播放时间的歌词, `rfind` 从后往前找，效率更高
            if let Some((_time, text)) = lyrics.iter().rfind(|(time, _)| *time <= current_pos) {
                lrc_to_display = text.clone();
            }
        }
        self.current_lrc = lrc_to_display;
        // -- 歌词更新逻辑结束 --

        // 打印 播放进度 + 歌词
        // 准备进度条字符串
        let minutes = current_pos.as_secs() / 60;
        let seconds = current_pos.as_secs() % 60;
        let now_time = format!("{:02}:{:02}", minutes, seconds);
        let progress_line = format!(
            "{}🎶 {} ⌛{}/{}",
            "Music🌀".green().bold(),
            self.current_audio.blue(),
            now_time.blue(),
            self.total_time.green()
        );
        // --- 2. 渲染UI ---
        // 每次循环都回到我们最初保存的锚点
        execute!(io::stdout(), cursor::RestorePosition)?;
        //
        execute!(
            io::stdout(),
            // 清除第一行内容
            Clear(ClearType::UntilNewLine),
        )?;
        // 打印进度条
        print!("{}", progress_line);
        // 移动到下一行，并清除该行，然后打印歌词
        // MoveToNextLine(1) 将光标移动到下一行的第0列
        execute!(
            io::stdout(),
            cursor::MoveToNextLine(1),
            Clear(ClearType::UntilNewLine)
        )?;
        // 打印歌词
        print!("Lyrics🌀{}", self.current_lrc.cyan().bold());
        io::stdout().flush()?;
        Ok(())
    }
    /// 监听键盘,控制播放
    pub fn run_event_loop(&mut self) -> AnyResult<()> {
        // 进入终端`raw mode`
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        // 隐藏光标以防止闪烁
        execute!(stdout, cursor::Hide)?;
        // 在进入循环前，保存一次初始光标位置。
        // 这是我们两行UI的“锚点”。
        execute!(stdout, cursor::SavePosition)?;
        loop {
            self.update_ui()?;

            // 自动切歌, 列表循环
            if self.sink.empty() {
                if self.current_audio_idx == self.audio_total {
                    self.current_audio_idx = 1;
                    self.play()?;

                } else {
                    self.current_audio_idx += 1;
                    self.play()?;
                }
            }
            self.monitor_key()?;
        }
    }

    pub fn monitor_key(&mut self)->AnyResult<()> {
        use Operation::*;
        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char(' ') =>{self.key_action(TogglePaused)?},
                        KeyCode::Right =>{self.key_action(Next)?},
                        KeyCode::Left =>{self.key_action(Back)?},
                        KeyCode::Esc => {self.key_action(Exit)?},
                        _ => {},
                    }
                }
            }
        }
        Ok(())
    }
    pub fn key_action(&mut self,op:Operation)->AnyResult<()> {
        use Operation::*;
        match op {
            TogglePaused => {
                if self.sink.is_paused() {
                    self.sink.play();
                } else {
                    self.sink.pause();
                }
            },
            Next=>{
                if self.current_audio_idx == self.audio_total {
                    self.current_audio_idx = 1
                } else {
                    self.current_audio_idx += 1;
                }
                self.play()?;
            },
            Back=>{
                if self.current_audio_idx == 1 {
                    self.current_audio_idx = self.audio_total;
                } else {
                    self.current_audio_idx -= 1;
                }
                self.play()?;
            }, 
            Exit=>{
                self.sink.stop();
                // --- 4. 退出清理 ---
                // 循环结束后，清理我们用过的两行UI
                execute!(
                    io::stdout(),
                    cursor::RestorePosition,      // 回到锚点
                    Clear(ClearType::UntilNewLine), // 清除第一行
                    cursor::MoveToNextLine(1),      // 移动到第二行
                    Clear(ClearType::UntilNewLine), // 清除第二行
                    cursor::RestorePosition,      // 再次回到锚点，以防万一
                    cursor::Show                  // 最后显示光标
                )?;
                disable_raw_mode()?;
                exit(0);
            },           
        }
        Ok(())
    }
    /// 过滤后加载音频列表
    pub fn load_audio(&mut self) -> AnyResult<()> {
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
    use super::*;
    use std::path::Path;
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
