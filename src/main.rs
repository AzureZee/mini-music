use std::{
    fs,
    io::{self, ErrorKind},
};
// use colored::*;
use mini_music::{AnyResult, Args};
use mini_music::Player;

fn main() -> AnyResult<()> {
    Player::clear_screen();
    // 解析cmdline参数
    let mut args = Args::new();
    args.get_dir();
    let mut player = Player::new()?;
    if let Some(dir) = args.dir {
        //  验证目录是否正确
        if !fs::metadata(&dir)?.is_dir() {
            return Err(io::Error::new(ErrorKind::NotFound, "目录未找到!").into());
        }
        // println!(
        //     "{}",
        //     "[Esc]=Exit [Space]=Play/Pause\n[← →/A D]=Prev/Next\n".green()
        // );

        player.initial(dir)?;
        
        Player::run(player)?;
    }
    Player::clear_screen();
    Ok(())
}
