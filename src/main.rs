use files::{get_cache_path, read_file_with_fallback};
use std::{
    fs,
    io::{self, BufRead},
};

use crate::{clients::EmbeddingsClient, files::get_all_files_in_directory};
use anyhow::Result;
use args::Args;
use atty::Stream;
use clap::Parser;
use clients::{EmbeddingsClientImpl, OllamaEmbeddingsClient};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use text_splitter::{ChunkConfig, TextSplitter};
use tiktoken_rs::cl100k_base;

mod args;
mod clients;
mod files;

const DEFAULT_FLOOR: f32 = 0.2;

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

struct PrintableChunk {
    chunk: String,
    similarity: f32,
}
struct PrintableFile {
    file: String,
    chunks: Vec<PrintableChunk>,
}

impl PrintableFile {
    fn print(&self) {
        println!("file: {}", self.file);
        for chunk in &self.chunks {
            println!("chunk: {}", chunk.chunk);
            println!("similarity: {}\n", chunk.similarity);
        }
    }
}

fn chunk_file_with_embeddings(file: &str, embeddings_client: &EmbeddingsClientImpl) -> Result<Vec<Chunk>> {
    let text = match read_file_with_fallback(file) {
        Ok(text) => text,
        Err(err) => {
            eprintln!("Error reading file {}: {}", file, err);
            return Ok(Vec::new());
        }
    };

    let hash_of_file = Sha256::digest(text.as_bytes());
    let cache_file_name = format!("{:x}.cache", hash_of_file);
    let file_path = get_cache_path().join(cache_file_name);

    if file_path.exists() {
        let chunks: Vec<Chunk> = bincode::deserialize(&fs::read(file_path)?)?;
        return Ok(chunks);
    }

    let tokenizer = cl100k_base()?;
    let max_tokens = 100;
    let splitter = TextSplitter::new(ChunkConfig::new(max_tokens).with_sizer(tokenizer));
    let chunks = splitter.chunks(&text).map(|chunk| {
        let embeddings = embeddings_client
            .get_embeddings(&chunk.to_string())
            .unwrap_or_default();
        Chunk {
            text: chunk.to_string(),
            embeddings,
        }
    });

    let chunks: Vec<Chunk> = chunks.collect();
    fs::create_dir_all(get_cache_path())?;
    fs::write(file_path, bincode::serialize(&chunks)?)?;

    Ok(chunks)
}

fn run_standard(
    embeddings_client: &EmbeddingsClientImpl,
    search_phrase: &String,
    floor: &f32,
    no_query: &bool,
) -> Result<()> {
    let search_chunk = Chunk {
        text: search_phrase.clone(),
        embeddings: embeddings_client.get_embeddings(&search_phrase)?,
    };

    let current_dir = std::env::current_dir()?.clone();
    let current_directory = match current_dir.to_str() {
        Some(dir) => dir,
        None => panic!("Could not get current directory"),
    };
    let files = get_all_files_in_directory(current_directory);
    let documents = files
        .iter()
        .filter_map(|file| {
            let chunks = match chunk_file_with_embeddings(file, &embeddings_client) {
                Ok(chunks) => chunks,
                Err(err) => {
                    eprintln!("Error chunking file {}: {}", file, err);
                    return None;
                }
            };

            Some(Document {
                path: file.to_string(),
                chunks,
            })
        })
        .collect::<Vec<Document>>();

    if !no_query {
        println!("Results for search phrase: {}\n", search_phrase);
    }

    for document in documents {
        let chunks = document.chunks.iter().filter(|chunk| {
            let similarity = cosine_similarity(&search_chunk.embeddings, &chunk.embeddings);
            similarity > *floor
        });
        if chunks.clone().count() == 0 {
            continue;
        }

        let printable_chunks = chunks
            .map(|chunk| PrintableChunk {
                chunk: chunk.text.clone(),
                similarity: cosine_similarity(&search_chunk.embeddings, &chunk.embeddings),
            })
            .collect::<Vec<PrintableChunk>>();

        let printable_file = PrintableFile {
            file: document.path.clone(),
            chunks: printable_chunks,
        };

        printable_file.print();
    }
    Ok(())
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

fn do_comparison(first: String, second: String) -> Result<()> {
    let oec = OllamaEmbeddingsClient::new();
    let first_chunk = Chunk {
        text: first.clone(),
        embeddings: oec.get_embeddings(&first)?,
    };
    let second_chunk = Chunk {
        text: second.clone(),
        embeddings: oec.get_embeddings(&second)?,
    };

    let similarity = cosine_similarity(&first_chunk.embeddings, &second_chunk.embeddings);
    println!("first: {}", first);
    println!("second: {}", second);
    println!("similarity: {}", similarity);
    Ok(())
}

fn main() {
    let args = Args::parse();
    let mut search_phrase = args.query.unwrap_or("".to_string());

    if let Some(comparison) = args.comparison {
        match do_comparison(search_phrase, comparison) {
            Ok(_) => return,
            Err(err) => eprintln!("Error while doing comparison: {}", err),
        }
        return;
    }

    let stdin_text = get_stdin();
    if !stdin_text.is_empty() {
        search_phrase = stdin_text;
    }

    let floor = args.floor.unwrap_or(DEFAULT_FLOOR);
    let oec = OllamaEmbeddingsClient::new();
    let embeddings_client = EmbeddingsClientImpl::Ollama(oec);
    match run_standard(&embeddings_client, &search_phrase, &floor, &args.no_query) { 
        Ok(_) => return,
        Err(err) => eprintln!("Error while running: {}", err),
    }
}
