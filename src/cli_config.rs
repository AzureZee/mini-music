use clap::{Arg, Command};

/// 设置用于目录指定和帮助的必需参数与标志
pub fn cli_config() -> Command {
    Command::new("musicplayer").arg(
        Arg::new("music-dir")
            .short('d')
            .long("dir")
            .value_name("DIRECTORY")
            .help("Sets the music directory"),
    )
}
