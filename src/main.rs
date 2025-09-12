// #![allow(unused)]
use colored::*;
use mini_music::AnyResult;
mod cli_config;

use cli_config::cli_config;
use mini_music::Player;

/// CLI音乐播放器主程序
///
/// # 功能流程
/// 1. 初始化命令行解析
/// 2. 创建播放器实例
/// 3. 启动主事件循环
///
/// # 支持的快捷键
/// * 空格：播放/暂停
/// * ←/→：切歌
/// * Esc：退出
fn main() -> AnyResult<()> {
    // [更优雅地`ctrl+c`退出]因为在raw mode监听键盘来退出,暂时无用
    /*     ctrlc::set_handler(|| {
        println!("\n{}: Exiting...", "Info".blue());
        exit(0)
    })
    .unwrap(); */

    Player::clear_screen();
    // 解析cmdline参数
    let arg = cli_config().get_matches();
    println!("Music Player!");
    println!("{}", "\n[空格]播放/暂停 | [Esc]退出 | [←/→]切歌 \n".green());

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
