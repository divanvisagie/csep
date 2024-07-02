use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use anyhow::Result;
use async_trait::async_trait;

use super::EmbeddingsClient;

pub struct FastEmbeddingsClient {}

impl FastEmbeddingsClient {
    pub fn new() -> Self {
        FastEmbeddingsClient {}
    }
}

#[async_trait]
impl EmbeddingsClient for FastEmbeddingsClient {
    async fn get_embeddings(&self, text: &[&str]) -> Result<Vec<Vec<f32>>> {

        // With custom InitOptions
        let model = TextEmbedding::try_new(InitOptions {
            model_name: EmbeddingModel::AllMiniLML6V2,
            show_download_progress: true,
            ..Default::default()
        })?;

        // get documents from text param
        let documents = text.iter().map(|&t| t.to_string()).collect::<Vec<String>>();

        // Generate embeddings with the default batch size, 256
        let embeddings = model.embed(documents, None)?;

        Ok(embeddings)
    }
}
