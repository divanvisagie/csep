use anyhow::Result;
use std::{
    fs::{self, File},
    path::PathBuf,
};

use ignore::WalkBuilder;
use memmap2::Mmap;

fn is_binary_file(file: &str) -> bool {
    if !file.contains(".") {
        return true;
    }

    let banned_extensions = vec![
        "png", "jpg", "jpeg", "gif", "bmp", "ico", "tiff", "webp", "svg", "mp3", "mp4", "webm",
        "ogg", "flac", "wav", "avi", "mov", "wmv", "mpg", "flv", "swf", "zip", "gz", "tar", "rar",
        "7z", "bz2", "xz", "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "eot", "ttf", "woff",
        "woff2", "otf", "swf", "wasm", "webm", "webp", "mp4", "mp3", "ogg", "flac", "wav", "avi",
        "mov", "wmv", "mpg", "flv", "swf", "zip", "gz", "tar", "rar", "7z", "bz2", "xz", "pdf",
        "doc", "docx", "xls", "xlsx", "ppt", "pptx", "eot", "ttf", "woff", "woff2", "otf", "swf",
        "wasm", "webm", "webp", "mp4", "mp3", "ogg", "flac", "wav", "avi", "mov", "wmv", "mpg",
        "flv", "swf", "zip", "gz", "tar", "rar", "7z", "bz2", "xz", "pdf", "doc", "docx", "xls",
        "xlsx", "ppt", "pptx", "eot", "ttf", "woff", "woff2", "otf", "swf", "wasm", "webm", "webp",
        "mp4", "mp3", "ogg", "flac", "wav", "avi", "mov", "wmv", "mpg", "flv", "swf", "zip", "gz",
        "tar", "rar", "7z", "bz2", "xz", "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "eot",
    ];

    if let Some(extension) = file.split('.').last() {
        return banned_extensions.contains(&extension);
    } else {
        return false;
    }
}

pub fn get_all_files_in_directory(dir: &str) -> Vec<String> {
    let mut files = Vec::new();

    for result in WalkBuilder::new(dir).build() {
        match result {
            Ok(entry) => {
                let path = entry.path();
                // Check if it's a file
                if path.is_file() {
                    if let Some(path_str) = path.to_str() {
                        if !is_binary_file(path_str) {
                            files.push(path_str.to_string());
                        }
                    }
                }
            }
            Err(err) => eprintln!("ERROR: {:?}", err),
        }
    }

    files
}

pub fn read_file_with_fallback(file: &str) -> Result<String> {
    // Try to memory map the file first
    if let Ok(file_handle) = File::open(file) {
        if let Ok(mmap) = unsafe { Mmap::map(&file_handle) } {
            if let Ok(text) = std::str::from_utf8(&mmap) {
                return Ok(text.to_string());
            }
        }
    }

    // Fallback to standard read_to_string if memory mapping fails
    fs::read_to_string(file).map_err(|e| e.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_all_files_in_directory() {
        let files = get_all_files_in_directory("data");
        // print the list of files
        for file in &files {
            println!("{}", file);
        }
        assert_eq!(files.contains(&"data/subdir/more.txt".to_string()), true);
        assert_eq!(files.contains(&"data/typescript.txt".to_string()), true);
        assert_eq!(files.contains(&"data/rust.txt".to_string()), true);
        assert_eq!(files.len(), 3);
    }

    #[test]
    fn test_is_binary() {
        assert_eq!(is_binary_file("file.png"), true);
        assert_eq!(is_binary_file("file.txt"), false);
        assert_eq!(is_binary_file("file.rs"), false);
        assert_eq!(is_binary_file("file"), true);
    }
}
