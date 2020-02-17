use std::path::Path;
use std::fs::File;
use std::io::{Write, BufReader, BufRead};
use std::process::Command;
pub fn concat(dir_path: &Path, output_file_path: &Path) -> Result<(), String> {
    let index_file_path = Path::new("index.txt");
    let mut index_file = std::fs::File::create(index_file_path).unwrap();

    for entry_result in std::fs::read_dir(dir_path).unwrap() {
        let entry = entry_result.unwrap();
        let file_path = entry.path();
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




mod test{
    use crate::ffmpeg::*;
    use std::path::Path;
    #[test]
    fn test_ffmpeg_concat() {
        let output_file_path = Path::new("output.mp4");

        let dir_path =
            Path::new("tests")
                .join("ressources")
                .join("ffmpeg_concat");

        let result = concat(&dir_path, output_file_path);
        if let Err(e) = result {
            println!("{}", e);
            assert!(false);
        }

    }
}