use std::collections::BTreeMap;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::Command;

pub fn concat(dir_path: &Path, output_file_path: &Path) -> Result<(), String> {
    let index_file_path = Path::new("index.txt");

    let mut sorted = BTreeMap::new();
    for entry_result in std::fs::read_dir(dir_path).unwrap() {
        let entry = entry_result.unwrap();
        let file_path = entry.path();
        let mut stem = file_path.file_stem().unwrap().to_str().unwrap();
        stem = stem.trim_end_matches(crate::CLIP_SUFFIX_OFFENSE);
        stem = stem.trim_end_matches(crate::CLIP_SUFFIX_DEFENSE);
        println!("looking at file path {:?}", stem);
        let order = stem.parse::<u64>().unwrap();
        sorted.insert(order, file_path);
    }

    let mut index_file = std::fs::File::create(index_file_path).unwrap();
    for file_path in sorted.values() {
        let index_entry = format!("file '{}'\n", file_path.to_str().unwrap());
        index_file.write(index_entry.as_bytes()).unwrap();
    }

    index_file.flush().unwrap();

    let result = if let Ok(output) = Command::new("ffmpeg")
        .arg("-f")
        .arg("concat")
        .arg("-safe")
        .arg("0")
        .arg("-i")
        .arg(index_file_path.to_str().unwrap())
        .arg("-c")
        .arg("copy")
        .arg("-y")
        .arg(output_file_path.to_str().unwrap())
        .output()
    {
        if output.status.success() == false {
            Err(String::from_utf8(output.stderr).unwrap())
        } else {
            Ok(())
        }
    } else {
        Err(("could not execute ffmpeg".to_owned()))
    };

    std::fs::remove_file(index_file_path).unwrap();

    result
}

mod test {
    use crate::ffmpeg::*;
    use std::path::Path;
    #[test]
    fn test_ffmpeg_concat() {
        let output_file_path = Path::new("output.mp4");

        let dir_path = Path::new("tests").join("ressources").join("ffmpeg_concat");

        let result = concat(&dir_path, output_file_path);
        if let Err(e) = result {
            println!("{}", e);
            assert!(false);
        }

        std::fs::remove_file(output_file_path);
    }
}
