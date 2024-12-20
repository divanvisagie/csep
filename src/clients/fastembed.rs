use std::path::PathBuf;

use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};

use rayon::prelude::*;
use anyhow::Result;
use async_trait::async_trait;

use super::EmbeddingsClient;

pub struct FastEmbeddingsClient {
    model: TextEmbedding
}

pub fn get_cache_path() -> PathBuf {
    let tmp_dir = dirs::data_dir().unwrap();
    tmp_dir.join("csep").join("models")
}

impl FastEmbeddingsClient {
    pub fn new() -> Self {
        
        let init_options = InitOptions::new(EmbeddingModel::AllMiniLML6V2).with_show_download_progress(true).with_cache_dir(get_cache_path());
        let model = TextEmbedding::try_new(init_options);
        let model = model.unwrap();
        
        FastEmbeddingsClient {
            model
        }
    }
}

#[async_trait]
impl EmbeddingsClient for FastEmbeddingsClient {
    async fn get_embeddings(&self, text: &[&str]) -> Result<Vec<Vec<f32>>> {
        let documents = text.par_iter().map(|&t| t.to_string()).collect::<Vec<String>>();

        // Default batch size, 256 which is used if we pass None
        let embeddings = self.model.embed(documents, None)?;

        Ok(embeddings)
    }
}
