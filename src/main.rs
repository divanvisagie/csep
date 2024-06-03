use files::{get_cache_path, read_file_with_fallback};
use std::{fs, io::{self, BufRead}};

use crate::{clients::EmbeddingsClient, files::get_all_files_in_directory};
use args::Args;
use atty::Stream;
use clap::Parser;
use clients::OllamaEmbeddingsClient;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use text_splitter::{ChunkConfig, TextSplitter};
use tiktoken_rs::cl100k_base;

mod files;
mod args;
mod clients;

fn cosine_similarity(v1: &Vec<f32>, v2: &Vec<f32>) -> f32 {
    let dot_product = v1.par_iter().zip(v2).map(|(a, b)| a * b).sum::<f32>();
    let magnitude_v1 = (v1.par_iter().map(|a| a.powi(2)).sum::<f32>()).sqrt();
    let magnitude_v2 = (v2.par_iter().map(|a| a.powi(2)).sum::<f32>()).sqrt();
    let magnitude_product = magnitude_v1 * magnitude_v2;
    dot_product / magnitude_product
}

#[derive(Serialize, Deserialize)]
struct Chunk {
    text: String,
    embeddings: Vec<f32>,
}

struct Document {
    path: String,
    chunks: Vec<Chunk>,
}

fn chunk_file_with_embeddings(file: &str, oec: &OllamaEmbeddingsClient) -> Vec<Chunk> {
    let text = match read_file_with_fallback(file) {
        Ok(text) => text,
        Err(err) => {
            eprintln!("Error reading file {}: {}", file, err);
            return Vec::new();
        }
    };

    let hash_of_file = Sha256::digest(text.as_bytes());
    let cache_file_name = format!("{:x}.cache", hash_of_file);
    let file_path = get_cache_path().join(cache_file_name);

    if file_path.exists() {
        let chunks: Vec<Chunk> = bincode::deserialize(&fs::read(file_path).unwrap()).unwrap();
        return chunks;
    }

    let tokenizer = cl100k_base().unwrap();
    let max_tokens = 100;
    let splitter = TextSplitter::new(ChunkConfig::new(max_tokens).with_sizer(tokenizer));
    let chunks = splitter.chunks(&text).map(|chunk| {
        let embeddings = oec.get_embeddings(&chunk.to_string()).unwrap_or_default();
        Chunk {
            text: chunk.to_string(),
            embeddings,
        }
    });

    // write to cache
    let chunks: Vec<Chunk> = chunks.collect();
    // make sure directory exists
    fs::create_dir_all(get_cache_path()).unwrap();
    fs::write(file_path, bincode::serialize(&chunks).unwrap()).unwrap();

    chunks
}

fn run(search_phrase: &String, floor: &f32) {
    let oec = OllamaEmbeddingsClient::new();
    let search_chunk = Chunk {
        text: search_phrase.clone(),
        embeddings: oec.get_embeddings(&search_phrase).unwrap(),
    };

    let current_dir = std::env::current_dir().unwrap().clone();
    let current_directory = match current_dir.to_str() {
        Some(dir) => dir,
        None => panic!("Could not get current directory"),
    };
    let files = get_all_files_in_directory(current_directory);
    let documents = files
        .iter()
        .filter_map(|file| {
            let chunks = chunk_file_with_embeddings(file, &oec);

            Some(Document {
                path: file.to_string(),
                chunks,
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

