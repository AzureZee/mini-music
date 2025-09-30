use crate::{AnyResult, anyhow,view::*, utils::{load_and_parse_lrc, load_audio_list}};
use colored::Colorize;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use rodio::{Decoder, OutputStream, OutputStreamBuilder, Source};
use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufReader},
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use walkdir::DirEntry as WalkDirEntry;


/// CLI音乐播放器核心结构体
pub struct Player {
    /// 音频播放引擎，管理音频流的播放/暂停/停止
    pub sink: rodio::Sink,
    /// 音频输出流句柄，用于创建新的Sink实例
    _stream_handle: OutputStream,
    /// 音乐文件存储目录路径
    pub audio_dir: String,
    /// 音乐文件索引映射（索引 -> 文件元数据）
    pub audio_list: Option<HashMap<u32, WalkDirEntry>>,
    /// 当前播放曲目索引
    pub current_audio_idx: u32,
    /// 当前播放文件名（缓存显示用）
    pub current_audio: String,
    /// 总曲目数
    pub audio_total: u32,
    /// 当前曲目总时长
    pub src_time: u64,
    /// 当前曲目总时长的格式化字符串
    pub total_time: String,
    /// 解析后的歌词数据（时间戳 -> 歌词文本）
    pub lyrics: Option<Vec<(Duration, String)>>,
    /// 当前应显示的歌词行
    pub current_lrc: String,
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
        let sink = rodio::Sink::connect_new(_stream_handle.mixer());
        Ok(Self {
            sink,
            _stream_handle,
            audio_dir: String::new(),
            total_time: String::new(),
            current_audio: String::new(),
            audio_list: None,
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
        self.audio_list = load_audio_list(&self.audio_dir);
        // 计算总曲目数
        self.audio_total = self.audio_list.as_ref().unwrap().len() as u32;
        // 执行首次播放
        self.play()?;
        Ok(())
    }

    /// 运行播放器
    pub fn run(player: Player) -> AnyResult<()> {
        let shared_player = Arc::new(Mutex::new(player));
        // 进入终端`raw mode`
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        // 隐藏光标以防止闪烁
        execute!(stdout, cursor::Hide)?;
        // 保存初始光标位置。
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
        // 退出终端`raw mode`
        execute!(
            io::stdout(),
            cursor::RestorePosition, // 回到锚点
            cursor::Show             // 最后显示光标
        )?;
        disable_raw_mode()?;

        Ok(())
    }

    /// 播放指定索引的音频
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
                //TODO: cut
                // -- 在这里加载歌词 --
                // 每次播放新歌曲时，先清空旧歌词
                self.lyrics = None;
                self.current_lrc = "".to_string();
                // 尝试加载并解析歌词
                self.lyrics = load_and_parse_lrc(audio.path());
                // -- 歌词加载结束 --
                //TODO: end
                // 解码音频
                let file = BufReader::new(File::open(audio.path())?);
                let source = Decoder::new(file)?;
                // 获取音频时长
                let src_duration = source
                    .total_duration()
                    .unwrap_or_else(|| Duration::from_secs(0));
                let src_time = src_duration.as_secs();
                //TODO: cut
                let src_minutes = src_time / 60;
                let src_seconds = src_time % 60;
                self.total_time = format!("{:02}:{:02}", src_minutes, src_seconds);
                //TODO: end
                self.src_time = src_time;
                // 音量初始化
                self.sink.set_volume(1.0);
                // 加载音频源, 并开始播放
                self.sink.append(source);
                //TODO: cut
                //获取不含扩展名的文件名
                self.current_audio = audio
                    .path()
                    .file_stem()
                    .unwrap()
                    .to_string_lossy()
                    .to_string();
                //TODO: end
                Ok(())
            } else {
                Err(anyhow!("{}: 无效的音频索引", "Error".red()))
            }
        } else {
            Err(anyhow!("{}", "未加载音频列表".red()))
        }
    }

    /// 派生子线程, 刷新UI
    fn ui_thread(shared_player: SharedPlayer) -> thread::JoinHandle<AnyResult<()>> {
        thread::spawn(move || -> AnyResult<()> {
            while !shared_player.lock().unwrap().should_exit {
                // shared_player.lock().unwrap().update_ui()?;
                //TODO: 完善此函数
                update_ui(&shared_player.lock().unwrap())?;
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
                if event::poll(Duration::from_millis(100))?
                    && let Event::Key(key) = event::read()?
                    && key.kind == KeyEventKind::Press
                {
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
                clear_screen();
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
    fn seek(&mut self, target_pos: Duration) -> AnyResult<()> {
        self.play()?;
        let _ = self.sink.try_seek(target_pos);
        Ok(())
    }
}
