use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use self::ollama::OllamaEmbeddingsClient;

pub mod ollama;

#[derive(Debug, Serialize, Deserialize)]
struct EmbeddingsRequest {
    input: String,
    model: String,
}

pub enum EmbeddingsClientImpl {
    Ollama(OllamaEmbeddingsClient),
}

#[async_trait]
impl EmbeddingsClient for EmbeddingsClientImpl {
    async fn get_embeddings(&self, text: &[&str]) -> Result<Vec<Vec<f32>>> {
        match self {
            EmbeddingsClientImpl::Ollama(client) => client.get_embeddings(text).await,
        }
    }
}

#[async_trait]
pub trait EmbeddingsClient {
    async fn get_embeddings(&self, text: &[&str]) -> Result<Vec<Vec<f32>>>;
}

