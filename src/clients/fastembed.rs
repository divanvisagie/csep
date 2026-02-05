use std::path::PathBuf;

use fastembed::{EmbeddingModel, TextEmbedding, TextInitOptions};

use anyhow::Result;
use async_trait::async_trait;
use rayon::prelude::*;
use std::sync::Mutex;

use super::EmbeddingsClient;
use crate::paths;

pub struct FastEmbeddingsClient {
    model: Mutex<TextEmbedding>,
    model_name: String,
}

pub fn get_cache_path() -> PathBuf {
    paths::models_cache_dir()
}

impl FastEmbeddingsClient {
    pub fn new(model_name: Option<&str>) -> Self {
        // Default to all-minilm-l6-v2 if no model specified
        let model_name = model_name.unwrap_or("all-minilm-l6-v2");

        // Map model names to EmbeddingModel enum
        let embedding_model = match model_name {
            "all-minilm-l6-v2" => EmbeddingModel::AllMiniLML6V2,
            "all-minilm-l12-v2" => EmbeddingModel::AllMiniLML12V2,
            "bge-small-en-v1.5" => EmbeddingModel::BGESmallENV15,
            "bge-base-en-v1.5" => EmbeddingModel::BGEBaseENV15,
            "bge-large-en-v1.5" => EmbeddingModel::BGELargeENV15,
            "nomic-embed-text-v1.5" => EmbeddingModel::NomicEmbedTextV15,
            "multilingual-e5-small" => EmbeddingModel::MultilingualE5Small,
            "multilingual-e5-base" => EmbeddingModel::MultilingualE5Base,
            "mxbai-embed-large-v1" => EmbeddingModel::MxbaiEmbedLargeV1,
            _ => {
                eprintln!(
                    "Warning: Unknown model '{}', defaulting to all-minilm-l6-v2",
                    model_name
                );
                EmbeddingModel::AllMiniLML6V2
            }
        };

        let init_options = TextInitOptions::new(embedding_model)
            .with_show_download_progress(true)
            .with_cache_dir(get_cache_path());
        let model = TextEmbedding::try_new(init_options).unwrap();

        FastEmbeddingsClient {
            model: Mutex::new(model),
            model_name: model_name.to_string(),
        }
    }
}

#[async_trait]
impl EmbeddingsClient for FastEmbeddingsClient {
    async fn get_embeddings(&self, text: &[&str]) -> Result<Vec<Vec<f32>>> {
        let documents = text
            .par_iter()
            .map(|&t| t.to_string())
            .collect::<Vec<String>>();

        // Default batch size, 256 which is used if we pass None
        let mut model = self.model.lock().map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        let embeddings = model.embed(documents, None)?;

        Ok(embeddings)
    }

    fn model_name(&self) -> &str {
        &self.model_name
    }
}
