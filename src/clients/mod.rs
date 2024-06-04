use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::error;

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

// Ollama implementation
pub struct OllamaEmbeddingsClient {
    base_url: &'static str,
    model: String,
}
impl OllamaEmbeddingsClient {
    pub fn new(model: Option<String>) -> Self {
        OllamaEmbeddingsClient {
            base_url: "http://localhost:11434",
            model: model.unwrap_or_else(|| "all-minilm".to_string()),
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
