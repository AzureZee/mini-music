use std::{
    fs,
    io::{self, ErrorKind},
};
use mini_music::{player::Player, AnyResult, Args,view::clear_screen};

fn main() -> AnyResult<()> {
    clear_screen();
    // 解析cmdline参数
    let mut args = Args::default();
    args.get_dir();
    let mut player = Player::new()?;
    if let Some(dir) = args.dir {
        //  验证目录是否正确
        if !fs::metadata(&dir)?.is_dir() {
            return Err(io::Error::new(ErrorKind::NotFound, "目录未找到!").into());
        }

        player.initial(&dir)?;
        
        Player::run(player)?;
    }
    clear_screen();
    Ok(())
}
