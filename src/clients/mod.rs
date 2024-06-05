use anyhow::Result;
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

impl EmbeddingsClient for EmbeddingsClientImpl {
    fn get_embeddings(&self, text: &String) -> Result<Vec<f32>> {
        match self {
            EmbeddingsClientImpl::Ollama(client) => client.get_embeddings(text),
        }
    }
}

pub trait EmbeddingsClient {
    fn get_embeddings(&self, text: &String) -> Result<Vec<f32>>;
}

