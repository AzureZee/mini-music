# `mini-music` CLI 音乐播放器

## 简介
一个基于 Rust 的命令行音乐播放器，支持基础播放控制、歌词显示和目录扫描功能。

## 功能特性
- 🎵 支持常见音频格式（MP3/WAV/OGG等）
- 📄 实时歌词解析显示（.lrc文件）
- ⌨️ 快捷键控制播放/暂停/切歌
- 🔍 自动扫描指定目录音频文件
- 📊 播放进度条可视化

## 安装指南
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
cargo run -- --dir ~/Music

# 快捷键说明
[空格] 播放/暂停      [←/→] 上一首/下一首
[Esc]  退出播放器
```

## 项目结构
```rust
src/
├── main.rs      // 程序入口
├── cli_config.rs// 命令行参数解析
├── player.rs    // 核心播放逻辑
└── lib.rs       // 公共模块导出
```

## 依赖库
- `rodio` 音频播放引擎
- `clap` 命令行参数解析
- `crossterm` 终端控制
- `colored` 终端着色

## 许可证
[MIT License](LICENSE) © 2023 AzureZee