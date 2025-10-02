use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

const EXT_LIST: [&str; 7] = ["mp3", "m4a", "flac", "aac", "wav", "ogg", "ape"];

fn is_audio(file: &Path) -> bool {
    let ext = file.extension().expect("Error: Couldn't get the extension!");
    EXT_LIST.contains(&ext.to_str().expect("Error: Couldn't get the extension!"))
}
/// 使用扩展名过滤文件, 使用`WalkDir`递归遍历目录, 加载音频列表
pub fn load_list(dir: &Path) -> Option<HashMap<u32, PathBuf>> {
    //
    let mut audio_index = 1;
    let mut audio_list = HashMap::new();
    let walk_iter = WalkDir::new(dir)
        .min_depth(1)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok());
    for entry in walk_iter {
        let path = entry.path();
        if path.is_file() && is_audio(path) {
            audio_list.insert(audio_index, path.to_path_buf());
            audio_index += 1;
        }
    }

    Some(audio_list)
}

pub fn get_audio_path(audio_list: &HashMap<u32, PathBuf>, idx: u32) -> PathBuf {
    audio_list
        .get(&idx)
        .expect("Error: Couldn't get the audio path!")
        .into()
}
