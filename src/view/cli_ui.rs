use crate::{AnyResult, player::PlayCore};
use std::{
    io::{self, Write},
    time::Duration,
};
use colored::Colorize;
use crossterm::{
    cursor, execute,
    terminal::{Clear, ClearType},
};

/// 打印详细信息 + 进度条 + 歌词
pub fn update_ui(core: &PlayCore) -> AnyResult<()> {
    // 获取当前播放位置
    let current_pos = core.get_pos();
    let current_lrc = update_lrc(core, current_pos);
    // 准备字符串
    let information = update_info(core, current_pos.as_secs());
    let progress_line = update_progress_line(core, current_pos.as_secs());
    // 每次循环都回到最初保存的锚点
    execute!(io::stdout(), cursor::RestorePosition)?;
    // 清除该行
    execute!(io::stdout(), Clear(ClearType::UntilNewLine),)?;
    // 打印歌曲信息
    print!("{}", information);
    move_and_clear_new_line()?;
    // 打印进度条
    print!("{}", progress_line);
    move_and_clear_new_line()?;
    // 打印歌词
    print!("🎤 {}", current_lrc.cyan().bold());
    move_and_clear_new_line()?;
    io::stdout().flush()?;
    Ok(())
}

/// 清除屏幕内容
pub fn clear_screen() {
    #[cfg(windows)]
    std::process::Command::new("cmd")
        .args(["/C", "cls"])
        .status()
        .ok();

    #[cfg(unix)]
    std::process::Command::new("clear").status().ok();
}
/// 更新当前歌词
fn update_lrc(core: &PlayCore, current_pos: Duration) -> String {
    // 默认无歌词
    let mut lrc_to_display = "".to_string();
    // 查找当前应显示的歌词
    if let Some(lyrics) = &core.lyrics {
        // 查找最后一个时间点小于等于当前播放时间的歌词, `rfind` 从后往前找，效率更高
        if let Some((_time, text)) = lyrics.iter().rfind(|(time, _)| *time <= current_pos) {
            lrc_to_display = text.clone();
        }
    }
    lrc_to_display
}
/// 更新进度条
fn update_progress_line(core: &PlayCore, current_pos: u64) -> String {
    // 进度条打印字符长度
    let progress_total_len = 35;
    // 每个字符对应的时间范围
    let seconds_per_char = core.src_time / progress_total_len;
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
    // 进度条字符串
    match progress_total_len - current_progress {
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
    }
}

/// 更新歌曲信息
fn update_info(core: &PlayCore, current_pos: u64) -> String {
    let minutes = current_pos / 60;
    let seconds = current_pos % 60;
    let now_time = format!("{:02}:{:02}", minutes, seconds);
    format!(
        "📀 {}/{} 🎧{} ⏳{}/{}",
        core.current_audio_idx.to_string().blue(),
        core.audio_total.to_string().yellow(),
        core.file_name.blue(),
        now_time.blue(),
        core.total_time.green()
    )
}

/// 移动到下一行，并清除该行.
fn move_and_clear_new_line() -> AnyResult<()> {
    execute!(
        io::stdout(),
        cursor::MoveToNextLine(1),
        Clear(ClearType::UntilNewLine)
    )?;
    Ok(())
}
