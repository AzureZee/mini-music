use std::collections::HashMap;
use walkdir::{DirEntry as WalkDirEntry, WalkDir};

const EXT_LIST: [&str; 7] = ["mp3", "m4a", "flac", "aac", "wav", "ogg", "ape"];
/// 使用扩展名过滤文件, 使用`WalkDir`递归遍历目录, 加载音频列表
pub fn load_audio_list(dir: &String) -> Option<HashMap<u32, WalkDirEntry>> {
    //
    let mut index = 1;
    let mut audio_map = HashMap::new();

    // 使用 WalkDir 递归遍历目录
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file()
            && let Some(ext) = path.extension()
            && EXT_LIST.contains(&ext.to_str().unwrap())
        {
            audio_map.insert(index, entry);
            index += 1;
        }
    }

    Some(audio_map)
}
