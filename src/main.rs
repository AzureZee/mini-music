use std::{
    fs,
    io::{self, ErrorKind},
};

// #![allow(unused)]
use colored::*;
use mini_music::AnyResult;
mod cli_config;

use cli_config::Args;
use mini_music::Player;

fn main() -> AnyResult<()> {
    Player::clear_screen();
    // 解析cmdline参数
    let mut args = Args::new();
    args.get_dir();
    if let Some(dir) = args.dir {
        //  验证目录是否正确
        if !fs::metadata(&dir)?.is_dir() {
            return Err(io::Error::new(ErrorKind::NotFound, "目录未找到!").into());
        }
        println!(
            "{}",
            "[Esc]=Exit [Space]=Play/Pause\n[← →/A D]=Prev/Next\n".green()
        );

        let mut app = Player::new()?;
        app.run(dir)?;
    }
    Ok(())
}
