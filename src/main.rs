use mini_music::{AnyResult, Args, player::App, view::clear_screen};
use std::{
    fs,
    io::{self, ErrorKind},
};

fn main() -> AnyResult<()> {
    clear_screen();
    // 解析cmdline参数
    let mut args = Args::default();
    args.get_dir();
    if let Some(dir) = args.dir {
        //  验证目录是否正确
        if !fs::metadata(&dir)?.is_dir() {
            return Err(io::Error::new(ErrorKind::NotFound, "目录未找到!").into());
        }
        App::run(&dir)?;
    }
    clear_screen();
    Ok(())
}
