use anyhow::Result;
use async_trait::async_trait;
use candle_core::{DType, Device};
use fastembed::Qwen3TextEmbedding;

use super::EmbeddingsClient;

pub struct Qwen3EmbeddingsClient {
    model: Qwen3TextEmbedding,
    model_name: String,
}

impl Qwen3EmbeddingsClient {
    /// Try to create a Qwen3 client with GPU acceleration.
    /// Returns None if no GPU is available.
    pub fn try_new(_model_name: Option<&str>) -> Option<Self> {
        let device = {
            #[cfg(feature = "cuda")]
            {
                Device::new_cuda(0).ok()
            }
            #[cfg(feature = "metal")]
            {
                Device::new_metal(0).ok()
            }
            #[cfg(not(any(feature = "cuda", feature = "metal")))]
            {
                None::<Device>
            }
        }?;

        let model = Qwen3TextEmbedding::from_hf(
            "Qwen/Qwen3-Embedding-0.6B",
            &device,
            DType::F32,
            512,
        )
        .ok()?;

        Some(Qwen3EmbeddingsClient {
            model,
            model_name: "qwen3-0.6b".to_string(),
        })
    }
}

#[async_trait]
impl EmbeddingsClient for Qwen3EmbeddingsClient {
    async fn get_embeddings(&self, text: &[&str]) -> Result<Vec<Vec<f32>>> {
        // Qwen3 uses instruction prefixes for best results:
        // - short texts (search queries) get "query: " prefix
        // - longer texts (document chunks) get "passage: " prefix
        let prefixed: Vec<String> = text
            .iter()
            .map(|t| {
                if t.split_whitespace().count() <= 20 {
                    format!("query: {}", t)
                } else {
                    format!("passage: {}", t)
                }
            })
            .collect();
        let refs: Vec<&str> = prefixed.iter().map(|s| s.as_str()).collect();
        let embeddings = self.model.embed(&refs)?;
        Ok(embeddings)
    }

    fn model_name(&self) -> &str {
        &self.model_name
    }
}
