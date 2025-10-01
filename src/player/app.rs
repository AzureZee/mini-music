use std::{
    io,
    path::Path,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};

use crate::{AnyResult, PlayCore, SharedCore, utils::*, view::*};

pub struct App;
impl App {
    /// 运行播放器
    pub fn run(dir: &Path) -> AnyResult<()> {
        let mut core = PlayCore::new()?;
        core.initial(dir)?;
        let shared_core = Arc::new(Mutex::new(core));
        // 进入终端`raw mode`
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        // 隐藏光标以防止闪烁
        execute!(stdout, cursor::Hide)?;
        // 保存初始光标位置。
        execute!(stdout, cursor::SavePosition)?;
        let ui_handle = ui_thread(Arc::clone(&shared_core));
        let key_handle = monitor_key_thread(Arc::clone(&shared_core));
        // 主线程执行循环播放
        while !shared_core.lock().unwrap().is_exit() {
            {
                let mut core = shared_core.lock().unwrap();
                if core.is_empty() {
                    switch(&mut core, true);
                    core.playback()?;
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
}

/// 派生子线程, 刷新UI
fn ui_thread(shared_core: SharedCore) -> thread::JoinHandle<AnyResult<()>> {
    thread::spawn(move || -> AnyResult<()> {
        while !shared_core.lock().unwrap().is_exit() {
            {
                update_ui(&shared_core.lock().unwrap())?;
            }
            thread::sleep(Duration::from_millis(100));
        }
        Ok(())
    })
}

/// 派生子线程, 监听键盘事件,调用`key_action`执行具体操作
fn monitor_key_thread(shared_core: SharedCore) -> thread::JoinHandle<AnyResult<()>> {
    use Operation::*;
    thread::spawn(move || -> AnyResult<()> {
        while !shared_core.lock().unwrap().is_exit() {
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
                    let mut core = shared_core.lock().unwrap();
                    key_action(&mut core, op)?;
                }
            }
        }
        Ok(())
    })
}
