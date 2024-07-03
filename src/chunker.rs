use std::{fs, path::PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use text_splitter::{ChunkConfig, TextSplitter};
use tiktoken_rs::cl100k_base;
use tracing::warn;

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

pub fn get_cache_path() -> PathBuf {
    let tmp_dir = dirs::cache_dir().unwrap();
    tmp_dir.join("csep")
}

pub fn count_lines_in_text(text: &str) -> usize {
    text.lines().count()
}

/// Chunk a file into smaller pieces and get embeddings for each chunk
/// using TextSplitter and the provided embeddings client
pub async fn chunk_file_with_embeddings<'a>(
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
    let cache_file_name = format!("{:x}.cache", hash_of_file);
    let file_path = get_cache_path().join(cache_file_name);

    if file_path.exists() {
        match bincode::deserialize(&fs::read(&file_path)?) {
            Ok(chunks) => {
                return Ok((file.to_string(), chunks));
            }
            Err(err) => {
                warn!("Error deserializing cache file {}: {}", file, err);
                // Delete the file, if we cant read from it, its probably corrupt
                match fs::remove_file(&file_path) {
                    Ok(_) => (),
                    Err(err) => warn!("Error removing cache file {}: {}", file, err),
                }
            }
        };
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

    fs::create_dir_all(get_cache_path())?;
    fs::write(file_path, bincode::serialize(&chunks)?)?;

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

