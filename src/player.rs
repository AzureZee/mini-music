use crate::{AnyResult, anyhow};
use colored::Colorize;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode};
use crossterm::{cursor, execute};
use rodio::{Decoder, OutputStream, OutputStreamBuilder, Source};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufReader},
};
use walkdir::{DirEntry as WalkDirEntry, WalkDir};

use tool::load_and_parse_lrc;
mod tool;

/// CLI音乐播放器核心结构体
///
pub struct Player {
    /// 音频播放引擎，管理音频流的播放/暂停/停止
    sink: rodio::Sink,
    /// 音频输出流句柄，用于创建新的Sink实例
    _stream_handle: OutputStream,
    /// 音乐文件存储目录路径
    audio_dir: String,
    /// 音乐文件索引映射（索引 -> 文件元数据）
    audio_list: Option<HashMap<u32, WalkDirEntry>>,
    /// 当前播放曲目索引
    current_audio_idx: u32,
    /// 当前播放文件名（缓存显示用）
    current_audio: String,
    /// 总曲目数
    audio_total: u32,
    /// 当前曲目总时长
    src_time: u64,
    /// 当前曲目总时长的格式化字符串
    total_time: String,
    /// 解析后的歌词数据（时间戳 -> 歌词文本）
    lyrics: Option<Vec<(Duration, String)>>,
    /// 当前应显示的歌词行
    current_lrc: String,
    /// 退出标志
    should_exit: bool,
}
type SharedPlayer = Arc<Mutex<Player>>;

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
    /// 快进
    Forward,
    /// 后退
    Backward,
    /// 退出播放器
    Exit,
    /// 手动清屏
    Clean,
}

impl Player {
    /// 新建播放器Player实例
    pub fn new() -> AnyResult<Self> {
        // 获取链接默认音频设备输出流和其句柄
        let _stream_handle = OutputStreamBuilder::open_default_stream()?;
        // 创建一个接收器Sink
        let sink = rodio::Sink::connect_new(&_stream_handle.mixer());
        Ok(Self {
            sink,
            _stream_handle,
            audio_dir: String::new(),
            total_time: String::new(),
            current_audio: String::new(),
            audio_list: Some(HashMap::new()),
            current_audio_idx: 1,
            audio_total: 0,
            src_time: 0,
            lyrics: None,
            current_lrc: String::new(),
            should_exit: false,
        })
    }

    /// 初始化播放器
    pub fn initial(&mut self, dir: PathBuf) -> AnyResult<()> {
        // 缓存目录
        self.audio_dir = dir.to_string_lossy().into_owned().to_string();
        // 加载音频列表
        self.load_audio()?;
        // 计算总曲目数
        let total = self.audio_list.as_ref().unwrap().len();
        self.audio_total = total as u32;
        // 执行首次播放
        self.play()?;
        Ok(())
    }

    /// 运行播放器
    ///
    pub fn run(player: Player) -> AnyResult<()> {
        let shared_player = Arc::new(Mutex::new(player));
        // 进入终端`raw mode`
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        // 隐藏光标以防止闪烁
        execute!(stdout, cursor::Hide)?;
        // 在进入循环前，保存一次初始光标位置。
        // 这是两行UI的“锚点”。
        execute!(stdout, cursor::SavePosition)?;
        let ui_handle = Player::ui_thread(Arc::clone(&shared_player));
        let key_handle = Player::monitor_key_thread(Arc::clone(&shared_player));
        // 主线程执行循环播放
        while !shared_player.lock().unwrap().should_exit {
            {
                let mut player = shared_player.lock().unwrap();
                if player.sink.empty() {
                    // eprintln!("sink is empty");
                    if player.current_audio_idx == player.audio_total {
                        player.current_audio_idx = 1;
                        player.play()?;
                    } else {
                        player.current_audio_idx += 1;
                        player.play()?;
                    }
                }
            }
            thread::sleep(Duration::from_millis(200));
        }
        // 等待子线程结束
        ui_handle.join().unwrap()?;
        key_handle.join().unwrap()?;
        // --- 退出清理 ---
        execute!(
            io::stdout(),
            cursor::RestorePosition, // 回到锚点
            // Clear(ClearType::All),
            cursor::Show // 最后显示光标
        )?;
        disable_raw_mode()?;

        Ok(())
    }

    /// 播放指定索引的音频
    ///
    fn play(&mut self) -> AnyResult<()> {
        //  切换前清空Sink
        if !self.sink.is_paused() {
            self.sink.clear();
            self.sink.play();
        } else {
            self.sink.clear();
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
                // 解码音频
                let file = BufReader::new(File::open(audio.path())?);
                let source = Decoder::new(file)?;
                // 获取音频时长
                let src_duration = source
                    .total_duration()
                    .unwrap_or_else(|| Duration::from_secs(0));
                let src_time = src_duration.as_secs();
                let src_minutes = src_time / 60;
                let src_seconds = src_time % 60;
                self.total_time = format!("{:02}:{:02}", src_minutes, src_seconds);
                self.src_time = src_time;
                // 音量初始化
                self.sink.set_volume(1.0);
                // 加载音频源, 并开始播放
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
    /// 更新当前歌词并返回当前播放位置
    fn update_lrc(&mut self) -> Duration {
        // 获取当前播放位置
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
        current_pos
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
        let current_pos = self.update_lrc().as_secs();
        // 进度条打印字符长度
        let progress_total_len = 35;
        // 每个字符对应的时间范围
        let seconds_per_char = self.src_time / progress_total_len;
        // 当前进度字符长度
        let current_progress = match current_pos / seconds_per_char {
            result if result >= 1 => {
                if result <= progress_total_len {
                    result
                } else {
                    progress_total_len
                }
            }
            _ => 0,
        };
        // 打印 详细信息 + 进度条 + 歌词
        // 准备字符串
        let minutes = current_pos / 60;
        let seconds = current_pos % 60;
        let now_time = format!("{:02}:{:02}", minutes, seconds);
        let information = format!(
            "📀 {}/{} 🎧{} ⏳{}/{}",
            self.current_audio_idx.to_string().blue(),
            self.audio_total.to_string().yellow(),
            self.current_audio.blue(),
            now_time.blue(),
            self.total_time.green()
        );
        // 进度条字符串
        let progress_line = match progress_total_len - current_progress {
            // 剩余进度字符长度
            remaining_progress if remaining_progress >= 1 => {
                if current_progress >= 1 {
                    format!(
                        "<>{}{}<>",
                        "#".repeat(current_progress as usize).blue(),
                        "-".repeat(remaining_progress as usize)
                    )
                } else {
                    format!(
                        "{}{}<>",
                        "<>".blue(),
                        "-".repeat(remaining_progress as usize)
                    )
                }
            }
            _ => {
                format!("<>{}<>", "#".repeat(current_progress as usize).blue())
            }
        };

        // 每次循环都回到最初保存的锚点
        execute!(io::stdout(), cursor::RestorePosition)?;
        //
        execute!(
            io::stdout(),
            // 清除第一行内容
            Clear(ClearType::UntilNewLine),
        )?;
        // 打印歌曲信息
        print!("{}", information);
        execute!(
            io::stdout(),
            cursor::MoveToNextLine(1),
            Clear(ClearType::UntilNewLine)
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

    /// 派生子线程, 刷新UI
    fn ui_thread(shared_player: SharedPlayer) -> thread::JoinHandle<AnyResult<()>> {
        thread::spawn(move || -> AnyResult<()> {
            while !shared_player.lock().unwrap().should_exit {
                shared_player.lock().unwrap().update_ui()?;
                thread::sleep(Duration::from_millis(100));
            }
            Ok(())
        })
    }

    /// 派生子线程, 监听键盘事件,调用`key_action`执行具体操作
    fn monitor_key_thread(shared_player: SharedPlayer) -> thread::JoinHandle<AnyResult<()>> {
        use Operation::*;
        thread::spawn(move || -> AnyResult<()> {
            while !shared_player.lock().unwrap().should_exit {
                if event::poll(Duration::from_millis(100))? {
                    if let Event::Key(key) = event::read()? {
                        if key.kind == KeyEventKind::Press {
                            let op = match key.code {
                                KeyCode::Char(' ') => Some(TogglePaused),
                                KeyCode::Char('c') => Some(Clean),
                                KeyCode::Left => Some(Backward),
                                KeyCode::Right => Some(Forward),
                                KeyCode::Up => Some(Prev),
                                KeyCode::Down => Some(Next),
                                KeyCode::Esc => Some(Exit),
                                _ => None,
                            };
                            if let Some(op) = op {
                                shared_player.lock().unwrap().key_action(op)?;
                            }
                        }
                    }
                }
            }
            Ok(())
        })
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
                self.should_exit = true;
            }
            Clean => {
                Player::clear_screen();
            }
            Forward => {
                let current_pos = self.sink.get_pos();
                let jump_duration = Duration::from_secs(5);
                let target_pos = current_pos + jump_duration;
                if target_pos.as_secs() <= self.src_time {
                    self.seek(target_pos)?;
                } else {
                    let target_pos = Duration::from_secs(self.src_time - 1);
                    self.seek(target_pos)?;
                }
            }
            Backward => {
                let current_pos = self.sink.get_pos();
                let jump_duration = Duration::from_secs(5);
                let target_pos;
                if current_pos.as_secs() <= jump_duration.as_secs() {
                    target_pos = Duration::from_secs(1);
                    self.seek(target_pos)?;
                } else {
                    target_pos = current_pos - jump_duration;
                    self.seek(target_pos)?;
                }
            }
        }
        Ok(())
    }

    /// 定位到当前音频的指定位置
    /// 
    fn seek(&mut self, target_pos: Duration) -> AnyResult<()> {
        self.play()?;
        let _ = self.sink.try_seek(target_pos);
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
