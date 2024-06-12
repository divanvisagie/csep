use std::{fs, path::PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use text_splitter::{ChunkConfig, TextSplitter};
use tiktoken_rs::cl100k_base;

use crate::{
    clients::{EmbeddingsClient, EmbeddingsClientImpl},
    files::read_file_with_fallback,
};

#[derive(Serialize, Deserialize)]
pub struct Chunk {
    pub line: usize,
    pub text: String,
    pub embeddings: Vec<f32>,
}

fn get_cache_path() -> PathBuf {
    let tmp_dir = dirs::cache_dir().unwrap();
    tmp_dir.join("csep")
}

pub fn chunk_file_with_embeddings(
    file: &str,
    embeddings_client: &EmbeddingsClientImpl,
) -> Result<Vec<Chunk>> {
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
    let mut line_count = 0;
    let chunks = splitter.chunks(&text).map(|chunk| {
        // count newline instances in chunk
        let embeddings = embeddings_client
            .get_embeddings(&chunk.to_string())
            .unwrap_or_default();
        let to_return = Chunk {
            line: line_count.clone(),
            text: chunk.to_string(),
            embeddings,
        };
        line_count += chunk.matches('\n').count();
        to_return
    });

    let chunks: Vec<Chunk> = chunks.collect();
    fs::create_dir_all(get_cache_path())?;
    fs::write(file_path, bincode::serialize(&chunks)?)?;

    Ok(chunks)
}
