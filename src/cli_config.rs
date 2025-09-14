use std::path::{Path, PathBuf};
use std::{env, fs};

use clap::Parser;
use directories::UserDirs;
use ini::Ini;
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

    /// 无参数或解析失败时, 打开FileDialog选择目录. 无参数打开FileDialog还会保存路径到配置文件
    pub fn get_dir(&mut self) {
        if env::args().len() == 1 {
            self.load_from_conf();
        } else {
            self.dir = match Args::try_parse() {
                Ok(parse) => parse.dir,
                Err(_) => match Args::open_dialog() {
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
    pub fn open_dialog() -> Option<PathBuf> {
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

    /// 从配置文件加载路径
    fn load_from_conf(&mut self) {
        let conf_path = "mini-conf.ini";
        // 文件不存在就新建
        if !Path::new(conf_path).exists() {
            if let Err(e) = fs::File::create(conf_path) {
                eprintln!("创建文件失败: {}", e);
                return;
            }
        }
        let content = match fs::read_to_string(conf_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("读取配置失败: {e}");
                return;
            }
        };
        let conf = Ini::new();
        // 内容为空则打开FileDialog并保存路径到配置文件
        if content.is_empty() {
            match Args::open_dialog() {
                Some(dir) => {
                    self.dir = Some(dir);
                    let dir_value = self.dir.clone().unwrap().to_string_lossy().into_owned();
                    Args::write_conf(conf, conf_path, dir_value);
                }
                _none => {
                    println!("你没有选择任何目录");
                }
            }
        } else {
            // 否则读取本地配置
            let mut dir_buf = PathBuf::new();
            let dir = Args::read_conf(content);
            dir_buf.push(dir);
            self.dir = Some(dir_buf);
        }
    }
    fn read_conf(content: String) -> String {
        let conf = Ini::load_from_str(&content).unwrap();
        let section = conf.section(Some("Directory")).unwrap();
        let dir_value = section.get("dir").unwrap();
        dir_value.to_string()
    }
    fn write_conf(mut conf: Ini, conf_path: &str, dir_value: String) {
        conf.with_section(Some("Directory")).set("dir", dir_value);
        conf.write_to_file(conf_path).unwrap();
    }
}
