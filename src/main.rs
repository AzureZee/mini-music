// #![allow(unused)]
use colored::*;
use mini_music::AnyResult;
mod cli_config;

use cli_config::cli_config;
use mini_music::Player;


fn main() -> AnyResult<()> {
    Player::clear_screen();
    // 解析cmdline参数
    let arg = cli_config().get_matches();
    println!("{}", "[Esc]=Exit [Space]=Play/Pause\n[← →/A D]=Prev/Next\n".green());
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
