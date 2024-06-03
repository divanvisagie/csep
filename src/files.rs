use anyhow::Result;
use std::{
    fs::{self, File},
    io::Read,
    path::PathBuf,
};

use ignore::WalkBuilder;
use memmap2::Mmap;

pub fn is_binary_file(file: &str) -> bool {
    const SAMPLE_SIZE: usize = 8000;
    let mut buffer = [0; SAMPLE_SIZE];

    if let Ok(mut f) = File::open(file) {
        if let Ok(size) = f.read(&mut buffer) {
            for byte in &buffer[..size] {
                if *byte == 0 || (*byte > 127 && byte != &9 && byte != &10 && byte != &13) {
                    return true;
                }
            }
        }
    }

    false
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
pub fn get_cache_path() -> PathBuf {
    let tmp_dir = dirs::cache_dir().unwrap();
    tmp_dir.join("csep")
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
        assert_eq!(files.len(), 3);
    }
}
