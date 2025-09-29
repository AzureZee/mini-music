# `mini-music` CLI 音乐播放器

## 简介
一个基于 Rust 的命令行音乐播放器，支持基础播放控制、歌词显示。

## 截图
![截图](https://img.cdn1.vip/i/68da549feceb3_1759138975.webp)

## 功能特性
- 🎵 支持常见音频格式（MP3/FLAC/M4A等）
- 📄 实时歌词解析显示（.lrc文件）
- ⌨️ 快捷键控制播放/暂停/切歌
- 📊 播放进度显示


## 构建项目
> 你可能需要先安装[Rust](https://www.rust-lang.org/tools/install)
```bash
# 克隆项目
git clone https://github.com/AzureZee/mini-music.git
cd mini-music

# 构建项目
cargo build --release
```

## 使用方法
```bash
# 指定音乐目录启动
cargo run -- --dir ~/Path
```
## 快捷键说明
```
[Esc] = Exit [Space]= Play/Pause

[↑/↓]= Prev/Next [←/→] = Forward/Backward
```

## 许可证
[MIT License](LICENSE) © 2025 AzureZee