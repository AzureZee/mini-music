use std::time::Duration;

use crate::{player::Player, view::clear_screen, AnyResult};

/// 键盘操作映射
///
/// 每个枚举值对应特定的播放控制功能
pub enum Operation {
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
/// 执行`Operation`变体对应的具体操作
pub fn key_action(core: &mut Player, op: Operation) -> AnyResult<()> {
    use Operation::*;
    match op {
        TogglePaused => {
            if core.is_paused() {
                core.play();
            } else {
                core.pause();
            }
        }
        Next => {
            switch(core,true);
            core.playback()?;
        }
        Prev => {
            switch(core,false);
            core.playback()?;
        }
        Exit => {
            core.stop();
            core.exit();
        }
        Clean => {
            clear_screen();
        }
        Forward => {
            forward(core)?;
        }
        Backward => {
            backward(core)?;
        }
    }
    Ok(())
}
pub fn switch(core: &mut Player, is_next: bool) {
    match is_next {
        true => {
            if core.current_audio_idx == core.audio_total {
                core.current_audio_idx = 1
            } else {
                core.current_audio_idx += 1;
            }
        }
        false => {
            if core.current_audio_idx == 1 {
                core.current_audio_idx = core.audio_total;
            } else {
                core.current_audio_idx -= 1;
            }
        }
    }
}
pub fn forward(core: &mut Player) -> AnyResult<()> {
    let span = Duration::from_secs(5);
    let target_pos = core.get_pos().saturating_add(span);
    if (0..core.src_time).contains(&target_pos.as_secs()) {
        core.seek(target_pos)?;
    } else {
        let target_pos = Duration::from_secs(core.src_time - 1);
        core.seek(target_pos)?;
    }
    Ok(())
}
pub fn backward(core: &mut Player) -> AnyResult<()> {
    let span = Duration::from_secs(5);
    let target_pos = core.get_pos().saturating_sub(span);
    if (0..core.src_time).contains(&target_pos.as_secs()) {
        core.seek(target_pos)?;
    }
    Ok(())
}