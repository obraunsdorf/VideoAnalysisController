use std::collections::BTreeMap;

use std::io::{Write};
use std::path::Path;
use std::process::Command;

pub fn concat(input_dir_path: &Path, output_dir_path: &Path) -> Result<(), String> {
    let index_file_path = Path::new("index.txt");

    let mut sorted_offense = BTreeMap::new();
    let mut sorted_defense = BTreeMap::new();
    let mut sorted_all = BTreeMap::new();
    for entry_result in std::fs::read_dir(input_dir_path).unwrap() {
        let entry = entry_result.unwrap();
        let file_path = entry.path();
        let mut stem = file_path.file_stem().unwrap().to_str().unwrap();
        if stem.ends_with(crate::CLIP_SUFFIX_OFFENSE) {
            stem = stem.trim_end_matches(crate::CLIP_SUFFIX_OFFENSE);
            let order = stem.parse::<u64>().unwrap();
            sorted_offense.insert(order, file_path.clone());
            sorted_all.insert(order, file_path.clone());
        } else if stem.ends_with(crate::CLIP_SUFFIX_DEFENSE) {
            stem = stem.trim_end_matches(crate::CLIP_SUFFIX_DEFENSE);
            let order = stem.parse::<u64>().unwrap();
            sorted_defense.insert(order, file_path.clone());
            sorted_all.insert(order, file_path.clone());
        } else {
            let order = stem.parse::<u64>().unwrap();
            sorted_all.insert(order, file_path.clone());
        }
    }

    let mut index_file = std::fs::File::create(index_file_path).unwrap();

    let mut result = Ok(());
    let off_out = output_dir_path.join("condensed_offense.mp4");
    let def_out = output_dir_path.join("condensed_defense.mp4");
    let all_out = output_dir_path.join("condensed_all.mp4");
    for (sorted, output_file_pathbuf) in [
        (sorted_offense, off_out),
        (sorted_defense, def_out),
        (sorted_all, all_out),
    ]
    .iter()
    {
        let output_file_path = output_file_pathbuf.as_path();
        for file_path in sorted.values() {
            let index_entry = format!("file '{}'\n", file_path.to_str().unwrap());
            index_file.write_all(index_entry.as_bytes()).unwrap();
        }

        index_file.flush().unwrap();

        if let Ok(output) = Command::new("ffmpeg")
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
                let code = match output.status.code() {
                    Some(c) => format!("{}", c),
                    None => "?".to_owned(),
                };
                result = Err(format!(
                    "error from ffmpeg[code:{}]: {}",
                    code,
                    String::from_utf8(output.stderr).unwrap()
                ));
            }
        } else {
            result = Err("could not execute ffmpeg".to_owned());
        }
    }

    std::fs::remove_file(index_file_path).unwrap();

    result
}

mod test {
    use super::*;
    use std::path::Path;
    #[test]
    fn test_ffmpeg_concat() {
        let output_dir_path = Path::new("tests")
            .join("output")
            .join("testvideo.mp4_condensed");
        std::fs::create_dir_all(&output_dir_path).unwrap();

        let input_dir_path = Path::new("tests")
            .join("ressources")
            .join("testvideo.mp4_clips");

        let result = concat(&input_dir_path, &output_dir_path);
        if let Err(e) = result {
            println!("{}", e);
            assert!(false);
        }

        let entries: Vec<std::ffi::OsString> = std::fs::read_dir(&output_dir_path).unwrap()
            . map(|res| res.map(|e| e.file_name()))
            .collect::<Result<Vec<_>, std::io::Error>>().unwrap();
        assert!(entries.iter().any(|e| e == "condensed_all.mp4"));
        assert!(entries.iter().any(|e| e == "condensed_defense.mp4"));
        assert!(entries.iter().any(|e| e == "condensed_offense.mp4"));
    }
}
