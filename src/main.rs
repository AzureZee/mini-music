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

/*     // 获取处理默认音频设备输出流的句柄
   let stream_handle = OutputStreamBuilder::open_default_stream().unwrap();
   // 解码音频, 获取音频时长
   let file = BufReader::new(File::open(arg.path).unwrap());
   let source = Decoder::new(file).unwrap();

   //
   // 创建一个新的接收器Sink，并在流上开始播放。
   let sink = Sink::connect_new(&stream_handle.mixer());

   sink.append(source);

   // 播放状态
   let mut is_pause = false;

   // 进入终端`raw mode`
   enable_raw_mode().expect("Enable Error!");
   println!("{}", "\n空格=播放/暂停, q=退出\n".green());

   loop {
       // 控制播放
       if is_pause {
           sink.pause();
           print!("{}", "\r   󰙣    󰙡   ".on_bright_blue());
           io::stdout().flush().unwrap();
       } else {
           sink.play();
           print!("{}", "\r   󰙣    󰙡   ".on_bright_blue());
           io::stdout().flush().unwrap();
       }
       // 监听KeyEvent
       if event::poll(Duration::from_millis(200)).expect("Poll Error!") {
           if let Event::Key(key) = event::read().expect("Read Error!") {
               if key.kind != KeyEventKind::Press {
                   continue;
               }
               match key.code {
                   KeyCode::Char(' ') => {
                       is_pause = !is_pause;
                   }
                   KeyCode::Char('q') => break,
                   _ => {}
               }
           }
       }
   }
   // 退出`raw mode`
   disable_raw_mode().unwrap();
   println!("\nbye")
*/
// sink.sleep_until_end();
// thread::sleep(time::Duration::from_secs(10));
