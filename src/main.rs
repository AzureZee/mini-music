use std::{
    fs,
    io::{self, ErrorKind},
};
use mini_music::{player::Player, AnyResult, Args};

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

        player.initial(dir)?;
        
        Player::run(player)?;
    }
    Player::clear_screen();
    Ok(())
}
