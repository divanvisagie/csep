use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use self::ollama::OllamaEmbeddingsClient;
use self::fastembed::FastEmbeddingsClient;

pub mod ollama;
pub mod fastembed;

#[derive(Debug, Serialize, Deserialize)]
struct EmbeddingsRequest {
    input: String,
    model: String,
}

#[allow(dead_code)]
pub enum EmbeddingsClientImpl {
    Ollama(OllamaEmbeddingsClient),
    FastEmbed(FastEmbeddingsClient),
}

impl EmbeddingsClientImpl {
    pub fn model_name(&self) -> String {
        match self {
            EmbeddingsClientImpl::Ollama(client) => client.model_name(),
            EmbeddingsClientImpl::FastEmbed(client) => client.model_name().to_string(),
        }
    }
}

#[async_trait]
impl EmbeddingsClient for EmbeddingsClientImpl {
    async fn get_embeddings(&self, text: &[&str]) -> Result<Vec<Vec<f32>>> {
        match self {
            EmbeddingsClientImpl::Ollama(client) => client.get_embeddings(text).await,
            EmbeddingsClientImpl::FastEmbed(client) => client.get_embeddings(text).await,
        }
    }
}

#[async_trait]
pub trait EmbeddingsClient {
    async fn get_embeddings(&self, text: &[&str]) -> Result<Vec<Vec<f32>>>;
}

