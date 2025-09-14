use std::env;
use std::path::PathBuf;

use clap::Parser;
use directories::UserDirs;
use rfd::FileDialog;

#[derive(Parser, Debug)]
pub struct Args {
    /// 音频目录
    #[arg(short, long)]
    pub dir: Option<PathBuf>,
}
impl Args {
    /// 新建Args实例
    pub fn new() -> Self {
        Self { dir: None }
    }
    /// 无参数或解析失败时, 打开FileDialog选择目录
    pub fn get_dir(&mut self) {
        if env::args().len() == 1 {
            match self.open_dialog() {
                Some(dir) => self.dir = Some(dir),
                _none => {
                    println!("你没有选择任何目录");
                }
            }
        } else {
            self.dir = match Args::try_parse() {
                Ok(parse) => parse.dir,
                Err(_) => match self.open_dialog() {
                    Some(dir) => Some(dir),
                    none => {
                        println!("你没有选择任何目录");
                        none
                    }
                },
            };
        }
    }
    /// 打开FileDialog选择目录
    fn open_dialog(&self) -> Option<PathBuf> {
        if let Some(user_dirs) = UserDirs::new() {
            if let Some(audio_dir) = user_dirs.audio_dir() {
                // println!("请选择一个文件夹");
                let folder = FileDialog::new()
                    .set_title("请选择一个文件夹")
                    .set_directory(audio_dir)
                    .pick_folder();
                match folder {
                    Some(path) => {
                        // println!("你选择的文件夹是: {}",path.display());
                        return Some(path);
                    }
                    none => return none,
                }
            }
        }
        None
    }
}
