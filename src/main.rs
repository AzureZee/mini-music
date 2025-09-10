// #![allow(unused)]
use anyhow::Result;
use colored::*;
#[allow(unused)]
use std::process::exit;
mod cli_config;
mod player;

use cli_config::cli_config;
use player::Player;

fn main() -> Result<()> {
    // [更优雅地`ctrl+c`退出]因为在raw mode监听键盘来退出,暂时无用
/*     ctrlc::set_handler(|| {
        println!("\n{}: Exiting...", "Info".blue());
        exit(0)
    })
    .unwrap(); */

    // 解析cmdline参数
    let arg = cli_config().get_matches();
    println!("Music Player!");
    println!("{}", "\n[空格]播放/暂停 | [Esc]退出 | [←/→]切歌 \n".green());
    
    // [获取元数据]暂时禁用, 先实现核心播放功能
    // get_metadata(&arg.path).expect("get_metadata error");

    let mut app = Player::new()?;
    app.run(arg)?;
    Ok(())
}

/* #[derive(Parser)]
struct Cli {
    /// 要读取的文件路径
    path: PathBuf,
} */

/*     let arg = Cli::parse();
println!("Music Player!");
println!("File path: {:?}", arg.path); */

