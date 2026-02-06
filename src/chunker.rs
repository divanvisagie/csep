use anyhow::Result;
use libsql::Connection;
use sha2::{Digest, Sha256};
use text_splitter::{ChunkConfig, TextSplitter};
use tiktoken_rs::cl100k_base;
use tracing::warn;

use crate::{clients::EmbeddingsClient, db, files::read_file_with_fallback};

pub struct Chunk {
    pub line: usize,
    pub text: String,
    pub embeddings: Vec<f32>,
}

pub fn count_lines_in_text(text: &str) -> usize {
    text.lines().count()
}

/// Chunk a file into smaller pieces and get embeddings for each chunk
/// using TextSplitter and the provided embeddings client.
/// Uses libSQL database for caching instead of flat files.
pub async fn get_chunks_and_embeddings_or_load_from_cache(
    file: &str,
    embeddings_client: &dyn EmbeddingsClient,
    conn: &Connection,
) -> Result<(String, Vec<Chunk>)> {
    let file_text = match read_file_with_fallback(file) {
        Ok(text) => text,
        Err(_err) => {
            warn!("Error reading file {}", file);
            return Ok((file.to_string(), Vec::new()));
        }
    };

    let hash_of_file = Sha256::digest(file_text.as_bytes());
    let content_hash = format!("{:x}", hash_of_file);

    if let Some(chunks) = db::get_cached_chunks(conn, &content_hash).await? {
        return Ok((file.to_string(), chunks));
    }

    let tokenizer = cl100k_base()?;
    let max_tokens = 100;
    let splitter = TextSplitter::new(ChunkConfig::new(max_tokens).with_sizer(tokenizer));

    let str_chunks: Vec<&str> = splitter.chunks(&file_text).collect();
    let embeddings_batch = embeddings_client.get_embeddings(&str_chunks[..]).await?;

    let mut lc = 1;
    #[allow(clippy::unused_enumerate_index)]
    let chunks: Vec<Chunk> = str_chunks
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

    db::store_file_chunks(conn, file, &content_hash, &chunks).await?;

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
