use tracing::error;
use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::EmbeddingsClient;

pub struct OllamaEmbeddingsClient {
    base_url: &'static str,
    model: String,
}
impl OllamaEmbeddingsClient {
    pub fn new(model: &Option<String>) -> Self {
        let model = model.clone();
        OllamaEmbeddingsClient {
            base_url: "http://localhost:11434",
            model: model.unwrap_or("all-minilm".to_string()),
        }
    }
}

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
}

#[derive(Deserialize)]
struct OllamaResponse {
    embedding: Vec<f32>,
}

/// Benchmark leaderboard: https://huggingface.co/spaces/mteb/leaderboard
pub const OLLAMA_MODELS: [&str; 3] = ["all-minilm", "mxbai-embed-large", "nomic-embed-text"];

impl EmbeddingsClient for OllamaEmbeddingsClient {
    fn get_embeddings(&self, text: &String) -> Result<Vec<f32>> {
        let url = format!("{}/api/embeddings", self.base_url);
        let client = reqwest::blocking::Client::new();

        let request_body = serde_json::to_string(&OllamaRequest {
            model: self.model.to_string(),
            prompt: text.to_string(),
        });

        let response = client.post(&url).body(request_body.unwrap()).send();

        let ollama_response = response?.text()?;

        let response_object: OllamaResponse = match serde_json::from_str(&ollama_response) {
            Ok(object) => object,
            Err(e) => {
                error!("Error in respone object: {}", e);
                return Err(anyhow::anyhow!("Error in response object"));
            }
        };

        Ok(response_object.embedding)
    }
}
