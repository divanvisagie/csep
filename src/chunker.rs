use std::fs;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use text_splitter::{ChunkConfig, TextSplitter};
use tiktoken_rs::cl100k_base;
use tracing::warn;

use crate::{
    cache::db::CacheDB,
    clients::{EmbeddingsClient, EmbeddingsClientImpl},
    files::read_file_with_fallback,
};

#[derive(Serialize, Deserialize)]
pub struct Chunk {
    pub line: usize,
    pub text: String,
    pub embeddings: Vec<f32>,
}


pub fn count_lines_in_text(text: &str) -> usize {
    text.lines().count()
}

/// Chunk a file into smaller pieces and get embeddings for each chunk
/// using TextSplitter and the provided embeddings client
pub async fn get_chunks_and_embeddings_or_load_from_cache<'a>(
    file: &'a str,
    embeddings_client: &EmbeddingsClientImpl,
) -> Result<(String, Vec<Chunk>)> {
    let file_text = match read_file_with_fallback(file) {
        Ok(text) => text,
        Err(_err) => {
            warn!("Error reading file {}", file);
            return Ok((file.to_string(), Vec::new()));
        }
    };

    let hash_of_file = Sha256::digest(file_text.as_bytes());
    let hash_string = format!("{:x}", hash_of_file);
    let db = CacheDB::new()?;
    let model = embeddings_client.model_name();

    if let Some(chunks) = db.get_chunks(file, &hash_string, &model)? {
        return Ok((file.to_string(), chunks));
    }

    let tokenizer = cl100k_base()?;
    let max_tokens = 100;
    let splitter = TextSplitter::new(ChunkConfig::new(max_tokens).with_sizer(tokenizer));

    let str_chunks: Vec<&str> = splitter.chunks(&file_text).collect();
    let embeddings_batch = embeddings_client.get_embeddings(&str_chunks[..]).await?;

    let mut lc = 1;
    let chunks = str_chunks
        .iter()
        .zip(embeddings_batch.iter())
        .enumerate()
        .map(|(_i, (chunk, embeddings))| {
            lc += count_lines_in_text(chunk);
            Chunk {
                line: lc,
                text: chunk.to_string(),
                embeddings: embeddings.to_owned(),
            }
        })
        .collect();

    db.upsert_chunks(file, &hash_string, &model, &chunks)?;

    Ok((file.to_string(), chunks))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test_count_lines_in_text() {
        let text = "Hello\nWorld\n";
        assert_eq!(count_lines_in_text(text), 2);
    }
}

