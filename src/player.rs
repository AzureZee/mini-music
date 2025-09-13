use crate::{AnyResult, anyhow};
use clap::ArgMatches;
use colored::Colorize;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode};
use crossterm::{cursor, execute};
use rodio::{Decoder, OutputStream, OutputStreamBuilder, Source};
use std::io::Write;
use std::process::exit;
use std::time::Duration;
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, BufReader, ErrorKind},
};
use walkdir::{DirEntry as WalkDirEntry, WalkDir};

use tool::load_and_parse_lrc;
mod tool;

/// CLI音乐播放器核心结构体
///
/// # 字段说明
/// * `sink` - 音频播放引擎，管理音频流的播放/暂停/停止
/// * `stream_handle` - 音频输出流句柄，用于创建新的Sink实例
/// * `audio_dir` - 音乐文件存储目录路径
/// * `audio_list` - 音乐文件索引映射（索引 -> 文件元数据）
/// * `current_audio_idx` - 当前播放曲目索引
/// * `current_audio` - 当前播放文件名（缓存显示用）
/// * `audio_total` - 总曲目数
/// * `total_time` - 当前曲目总时长（格式化字符串）
/// * `lyrics` - 解析后的歌词数据（时间戳 -> 歌词文本）
/// * `current_lrc` - 当前应显示的歌词行
pub struct Player {
    ///`sink` - 音频播放引擎，管理音频流的播放/暂停/停止
    sink: rodio::Sink,
    /// `stream_handle` - 音频输出流句柄，用于创建新的Sink实例
    _stream_handle: OutputStream,
    /// `audio_dir` - 音乐文件存储目录路径
    audio_dir: String,
    /// `audio_list` - 音乐文件索引映射（索引 -> 文件元数据）
    audio_list: Option<HashMap<u32, WalkDirEntry>>,
    /// `current_audio_idx` - 当前播放曲目索引
    current_audio_idx: u32,
    /// `current_audio` - 当前播放文件名（缓存显示用）
    current_audio: String,
    /// `audio_total` - 总曲目数
    audio_total: u32,
    /// `total_time` - 当前曲目总时长（格式化字符串）
    total_time: String,
    /// `lyrics` - 解析后的歌词数据（时间戳 -> 歌词文本）
    lyrics: Option<Vec<(Duration, String)>>,
    /// `current_lrc` - 当前应显示的歌词行
    current_lrc: String,
    /// 是否首次运行, 是就不清空Sink
    first_run: bool,
}

/// 键盘操作映射
///
/// 每个枚举值对应特定的播放控制功能
enum Operation {
    /// 切换播放/暂停状态
    TogglePaused,
    /// 切换到上一首
    Prev,
    /// 切换到下一首
    Next,
    /// 退出播放器
    Exit,
}

impl Player {
    /// 初始化播放器实例
    pub fn new() -> AnyResult<Self> {
        // 获取链接默认音频设备输出流和其句柄
        let _stream_handle = OutputStreamBuilder::open_default_stream()?;
        // 创建一个接收器Sink
        let sink = rodio::Sink::connect_new(&_stream_handle.mixer());
        // sink.pause();
        Ok(Self {
            sink,
            _stream_handle,
            audio_dir: String::new(),
            total_time: String::new(),
            current_audio: String::new(),
            audio_list: Some(HashMap::new()),
            current_audio_idx: 1,
            audio_total: 0,
            lyrics: None,
            current_lrc: String::new(),
            first_run: true,
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

        self.audio_dir = dir.to_string();
        self.load_audio()?;
        let total = self.audio_list.as_ref().unwrap().len();
        self.audio_total = total as u32;

        self.play()?;

        self.first_run = false;

        self.run_event_loop()?;
        Ok(())
    }

    /// 播放指定索引的音频
    ///
    /// # 流程说明
    /// 1. 清理现有播放状态（停止/重置Sink）
    /// 2. 加载新音频文件并解析元数据
    /// 3. 初始化播放参数：
    ///    - 设置初始音量
    ///    - 更新总时长显示
    ///    - 缓存文件名
    fn play(&mut self) -> AnyResult<()> {
        // 首次运行不需要清空
        if !self.first_run {
            //  切换前清空并新建Sink
            if !self.sink.is_paused() {
                self.sink.clear();
                self.sink.play();
                // self.sink = Sink::connect_new(&self.stream_handle.mixer());
            } else {
                self.sink.clear();
                // self.sink = Sink::connect_new(&self.stream_handle.mixer());
                self.sink.pause();
            }
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
                //获取不含扩展名的文件名
                self.current_audio = audio
                    .path()
                    .file_stem()
                    .unwrap()
                    .to_string_lossy()
                    .to_string();

                Ok(())
            } else {
                Err(anyhow!("{}: 无效的音频索引", "Error".red()))
            }
        } else {
            Err(anyhow!("{}", "未加载音频列表".red()))
        }
    }

    /// 清除屏幕内容
    ///
    /// 根据操作系统类型调用相应的清屏命令
    /// Windows系统使用"cls"命令，Unix系统使用"clear"命令
    pub fn clear_screen() {
        #[cfg(windows)]
        std::process::Command::new("cmd")
            .args(&["/C", "cls"])
            .status()
            .ok();

        #[cfg(unix)]
        std::process::Command::new("clear").status().ok();
    }
    /// UI渲染核心方法
    ///
    /// # 功能说明
    /// 1. 计算当前播放位置
    /// 2. 更新歌词显示
    /// 3. 渲染进度条和歌词界面
    ///
    /// # 界面布局
    /// 采用双行锚定模式：
    /// 1. 第一行：播放进度条
    /// 2. 第二行：当前歌词
    fn update_ui(&mut self) -> AnyResult<()> {
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
            "📀 {}/{} 🎧{} ⏳{}/{}",
            self.current_audio_idx.to_string().blue(),
            self.audio_total.to_string().yellow(),
            self.current_audio.blue(),
            now_time.blue(),
            self.total_time.green()
        );
        // --- 2. 渲染UI ---
        // 每次循环都回到最初保存的锚点
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
        print!("🎤 {}", self.current_lrc.cyan().bold());
        io::stdout().flush()?;
        Ok(())
    }
    /// 主事件循环驱动器
    ///
    /// # 功能说明
    /// 1. 初始化终端raw模式
    /// 2. 维护UI渲染锚点
    /// 3. 驱动以下核心循环：
    ///    - UI刷新
    ///    - 自动切歌
    ///    - 键盘事件监听
    fn run_event_loop(&mut self) -> AnyResult<()> {
        // 进入终端`raw mode`
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        // 隐藏光标以防止闪烁
        execute!(stdout, cursor::Hide)?;
        // 在进入循环前，保存一次初始光标位置。
        // 这是两行UI的“锚点”。
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

    /// 监听键盘事件,调用`key_action`执行具体操作
    fn monitor_key(&mut self) -> AnyResult<()> {
        use Operation::*;
        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char(' ') => self.key_action(TogglePaused)?,
                        KeyCode::Char('a') => self.key_action(Prev)?,
                        KeyCode::Char('d') => self.key_action(Next)?,
                        KeyCode::Left => self.key_action(Prev)?,
                        KeyCode::Right => self.key_action(Next)?,
                        KeyCode::Esc => self.key_action(Exit)?,
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    /// 执行`Operation`变体对应的具体操作
    fn key_action(&mut self, op: Operation) -> AnyResult<()> {
        use Operation::*;
        match op {
            TogglePaused => {
                if self.sink.is_paused() {
                    self.sink.play();
                } else {
                    self.sink.pause();
                }
            }
            Next => {
                if self.current_audio_idx == self.audio_total {
                    self.current_audio_idx = 1
                } else {
                    self.current_audio_idx += 1;
                }
                self.play()?;
            }
            Prev => {
                if self.current_audio_idx == 1 {
                    self.current_audio_idx = self.audio_total;
                } else {
                    self.current_audio_idx -= 1;
                }
                self.play()?;
            }
            Exit => {
                self.sink.stop();
                // --- 4. 退出清理 ---
                // 循环结束后，清理用过的两行UI
                execute!(
                    io::stdout(),
                    cursor::RestorePosition,        // 回到锚点
                    Clear(ClearType::UntilNewLine), // 清除第一行
                    cursor::MoveToNextLine(1),      // 移动到第二行
                    Clear(ClearType::UntilNewLine), // 清除第二行
                    cursor::RestorePosition,        // 再次回到锚点，以防万一
                    cursor::Show                    // 最后显示光标
                )?;
                Player::clear_screen();
                disable_raw_mode()?;
                exit(0);
            }
        }
        Ok(())
    }
    /// 使用扩展名过滤文件, 使用`WalkDir`递归遍历目录, 加载音频列表
    fn load_audio(&mut self) -> AnyResult<()> {
        let ext_list = ["mp3", "m4a", "flac", "aac", "wav", "ogg", "ape"];
        //
        let mut index = 1;
        let dir = &self.audio_dir;

        if let Some(audio_map) = &mut self.audio_list {
            // 使用 WalkDir 递归遍历目录
            for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
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
    use std::fs::read_dir;
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
        let path = Path::new("C:\\Users\\Admin\\Music");
        assert_eq!(2, filter(path))
    }
}
