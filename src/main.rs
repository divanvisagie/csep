use std::{
    fs,
    io::{self, BufRead},
};

use args::Args;
use atty::Stream;
use clap::Parser;
use embeddings::OllamaEmbeddingsClient;

use crate::embeddings::EmbeddingsClient;
use text_splitter::{ChunkConfig, TextSplitter};

use tiktoken_rs::cl100k_base;

mod embeddings;
mod args;

fn cosine_similarity(v1: &Vec<f32>, v2: &Vec<f32>) -> f32 {
    let dot_product = v1.iter().zip(v2).map(|(a, b)| a * b).sum::<f32>();
    let magnitude_v1 = (v1.iter().map(|a| a.powi(2)).sum::<f32>()).sqrt();
    let magnitude_v2 = (v2.iter().map(|a| a.powi(2)).sum::<f32>()).sqrt();
    let magnitude_product = magnitude_v1 * magnitude_v2;
    dot_product / magnitude_product
}

struct Chunk {
    text: String,
    embeddings: Vec<f32>,
}

struct Document {
    path: String,
    chunks: Vec<Chunk>,
}

fn get_all_files_in_directory(dir: &str) -> Vec<String> {
    let paths = fs::read_dir(dir).unwrap();
    let mut files = Vec::new();
    for path in paths {
        let path = path.unwrap().path();
        let path_str = path.to_str().unwrap().to_string();
        if path.is_dir() {
            let mut nested_files = get_all_files_in_directory(&path_str);
            files.append(&mut nested_files);
            continue;
        }
        files.push(path_str);
    }
    files
}
fn run(search_phrase: &String, floor: &f32) {
    let oec = OllamaEmbeddingsClient::new();
    let search_chunk = Chunk {
        text: search_phrase.clone(),
        embeddings: oec.get_embeddings(&search_phrase).unwrap(),
    };
    let tokenizer = cl100k_base().unwrap();
    let max_tokens = 100;
    let splitter = TextSplitter::new(ChunkConfig::new(max_tokens).with_sizer(tokenizer));

    let current_dir = std::env::current_dir().unwrap().clone();
    let current_directory = match current_dir.to_str() {
        Some(dir) => dir,
        None => panic!("Could not get current directory"),
    };
    let files = get_all_files_in_directory(current_directory);
    let documents = files
        .iter()
        .filter_map(|file| {
            let text = match fs::read_to_string(file) {
                Ok(text) => text,
                Err(_) => return None,
            };
            let chunks = splitter.chunks(&text).map(|chunk| chunk.to_string());
            let chunks = chunks.map(|chunk| {
                let embeddings = oec.get_embeddings(&chunk).unwrap();
                Chunk {
                    text: chunk,
                    embeddings,
                }
            });

            Some(Document {
                path: file.to_string(),
                chunks: chunks.collect(),
            })
        })
        .collect::<Vec<Document>>();

    println!("Results for search phrase: {}\n", search_phrase);
    for document in documents {
        let chunks = document.chunks.iter().filter(|chunk| {
            let similarity = cosine_similarity(&search_chunk.embeddings, &chunk.embeddings);
            similarity > *floor
        });
        if chunks.clone().count() == 0 {
            continue;
        }
        println!("file: {}", document.path);
        for chunk in chunks {
            println!("chunk: {}", chunk.text);
            let similarity = cosine_similarity(&search_chunk.embeddings, &chunk.embeddings);
            println!("similarity: {}\n", similarity);
        }
    }
}

pub fn get_stdin() -> String {
    let mut lines: Vec<String> = Vec::new();

    // Check if stdin is attached to a terminal or is being piped from another process
    if !atty::is(Stream::Stdin) {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            match line {
                Ok(line) => lines.push(line),
                Err(err) => println!("Error reading line: {}", err),
            }
        }
    }

    lines.join("\n")
}
fn main() {
    let args = Args::parse();
    let stdin_text = get_stdin();
    let mut search_phrase = args.query.unwrap_or("".to_string());
    if !stdin_text.is_empty() {
        search_phrase = stdin_text;
    }
    let floor = args.floor.unwrap_or(0.2);
    run(&search_phrase, &floor);
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
