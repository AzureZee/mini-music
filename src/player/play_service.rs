use crate::{AnyResult, anyhow, utils::*};
use rodio::{Decoder, OutputStream, OutputStreamBuilder, Source};
use std::{
    collections::HashMap,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};

/// CLI音乐播放器核心结构体
pub struct PlayCore {
    /// 音频输出设备的句柄，管理音频流的播放
    sink: rodio::Sink,
    /// 音频输出流句柄
    _stream_handle: OutputStream,
    /// 音乐文件索引映射（索引 -> 文件元数据）
    pub audio_list: Option<HashMap<u32, PathBuf>>,
    /// 当前播放曲目索引
    pub current_audio_idx: u32,
    /// 当前播放文件名（缓存显示用）
    pub file_name: String,
    /// 总曲目数
    pub audio_total: u32,
    /// 当前曲目总时长
    pub src_time: u64,
    /// 当前曲目总时长的格式化字符串
    pub total_time: String,
    /// 解析后的歌词数据（时间戳 -> 歌词文本）
    pub lyrics: Option<Vec<(Duration, String)>>,
    /// 退出标志
    should_exit: bool,
}
pub type SharedCore = Arc<Mutex<PlayCore>>;

impl PlayCore {
    /// 新建播放器PlayCore实例
    pub fn new() -> AnyResult<Self> {
        // 获取链接默认音频设备输出流和其句柄
        let _stream_handle = OutputStreamBuilder::open_default_stream()?;
        // 创建一个接收器Sink
        let sink = rodio::Sink::connect_new(_stream_handle.mixer());
        Ok(Self {
            sink,
            _stream_handle,
            total_time: String::new(),
            file_name: String::new(),
            audio_list: None,
            current_audio_idx: 1,
            audio_total: 0,
            src_time: 0,
            lyrics: None,
            should_exit: false,
        })
    }

    /// 初始化播放器
    pub fn initial(&mut self, dir: &Path) -> AnyResult<()> {
        // 加载音频列表
        self.audio_list = load_audio_list(dir);
        // 计算总曲目数
        self.audio_total = self.audio_list.as_ref().unwrap().len() as u32;
        // 执行首次播放
        self.playback()?;
        Ok(())
    }

    pub fn decoder(&self, audio: &Path) -> AnyResult<Decoder<BufReader<File>>> {
        // 解码音频
        let file = BufReader::new(File::open(audio)?);
        let source = Decoder::new(file)?;
        Ok(source)
    }
    pub fn get_duration(&self, source: &Decoder<BufReader<File>>) -> u64 {
        let src_duration = source
            .total_duration()
            .unwrap_or_else(|| Duration::from_secs(0));
        src_duration.as_secs()
    }
    pub fn get_audio_path(&self) -> AnyResult<PathBuf> {
        if let Some(audio_map) = &self.audio_list
            && let Some(audio) = audio_map.get(&self.current_audio_idx)
        {
            Ok(audio.into())
        } else {
            Err(anyhow!("无效的音频索引"))
        }
    }

    /// 播放指定索引的音频
    pub fn playback(&mut self) -> AnyResult<()> {
        self.hold_state_clear();
        //
        let audio = self.get_audio_path()?;
        // 尝试加载并解析歌词
        self.lyrics = load_and_parse_lrc(&audio);
        // 解码音频
        let source = self.decoder(&audio)?;
        // 获取音频时长
        let src_time = self.get_duration(&source);
        let minutes = src_time / 60;
        let seconds = src_time % 60;
        self.total_time = format!("{:02}:{:02}", minutes, seconds);
        self.src_time = src_time;
        // 音量初始化
        self.set_volume(1.0);
        // 加载音频源, 并开始播放
        self.append(source);
        //获取不含扩展名的文件名
        self.file_name = audio.file_stem().unwrap().to_string_lossy().to_string();
        Ok(())
    }

    /// 定位到当前音频的指定位置
    pub fn seek(&mut self, target_pos: Duration) -> AnyResult<()> {
        self.playback()?;
        let _ = self.sink.try_seek(target_pos);
        Ok(())
    }
    pub fn is_paused(&self) -> bool {
        self.sink.is_paused()
    }
    pub fn is_empty(&self) -> bool {
        self.sink.empty()
    }
    pub fn play(&self) {
        self.sink.play();
    }
    pub fn pause(&self) {
        self.sink.pause();
    }
    pub fn stop(&self) {
        self.sink.stop();
    }
    pub fn get_pos(&self) -> Duration {
        self.sink.get_pos()
    }
    pub fn set_volume(&self, value: f32) {
        self.sink.set_volume(value);
    }

    pub fn append(&self, source: Decoder<BufReader<File>>) {
        self.sink.append(source);
    }

    ///  确保清空Sink后不改变播放状态
    pub fn hold_state_clear(&mut self) {
        if !self.is_paused() {
            self.clear();
            self.play();
        } else {
            self.clear();
        }
    }

    pub fn clear(&mut self) {
        self.sink.clear();
    }
    pub fn is_exit(&self) -> bool {
        self.should_exit
    }
    pub fn exit(&mut self) {
        self.should_exit = true;
    }
}
