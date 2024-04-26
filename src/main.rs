use std::fs;

use embeddings::OllamaEmbeddingsClient;

use crate::embeddings::EmbeddingsClient;
use text_splitter::{ChunkConfig, TextSplitter};

use tiktoken_rs::cl100k_base;

mod embeddings;

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
        files.push(path_str);
    }
    files
}

fn main() {
    let oec = OllamaEmbeddingsClient::new();

    let search_phrase = "rust programming language".to_string();
    let search_chunk = Chunk {
        text: search_phrase.clone(),
        embeddings: oec.get_embeddings(&search_phrase).unwrap(),
    };

    let tokenizer = cl100k_base().unwrap();
    let max_tokens = 100;
    let splitter = TextSplitter::new(ChunkConfig::new(max_tokens).with_sizer(tokenizer));

    let files = get_all_files_in_directory("data");
    let documents = files
        .iter()
        .map(|file| {
            let text = fs::read_to_string(file).unwrap();
            let chunks = splitter.chunks(&text).map(|chunk| chunk.to_string());
            let chunks = chunks.map(|chunk| {
                let embeddings = oec.get_embeddings(&chunk).unwrap();
                Chunk {
                    text: chunk,
                    embeddings,
                }
            });
            
            Document {
                path: file.to_string(),
                chunks: chunks.collect(),
            }
        })
        .collect::<Vec<Document>>();

    for document in documents {
        println!("{}", document.path);
        for chunk in document.chunks {
            println!("{}", chunk.text);
            println!("{}", cosine_similarity(&search_chunk.embeddings, &chunk.embeddings));
        }
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_all_files_in_directory() {
        let files = get_all_files_in_directory("data");
        assert_eq!(files.len(), 2);
        assert_eq!(files[0], "data/rust.txt");
        assert_eq!(files[1], "data/typescript.txt");
    }
}
