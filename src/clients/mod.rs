use anyhow::Result;
use async_trait::async_trait;

pub mod fastembed;
#[cfg(any(feature = "cuda", feature = "metal"))]
pub mod qwen3;

/// Model information for listing available embedding models
#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub name: &'static str,
    pub description: &'static str,
    pub category: &'static str,
    pub dimensions: usize,
    #[allow(dead_code)]
    pub parameters: &'static str,
}

/// Get information about all available embedding models
pub fn get_available_models() -> Vec<ModelInfo> {
    #[allow(unused_mut)]
    let mut models = vec![
        ModelInfo {
            name: "all-minilm-l6-v2",
            description: "Small, fast model for general use",
            category: "fast",
            dimensions: 384,
            parameters: "22M",
        },
        ModelInfo {
            name: "all-minilm-l12-v2",
            description: "Larger MiniLM variant with better accuracy",
            category: "balanced",
            dimensions: 384,
            parameters: "33M",
        },
        ModelInfo {
            name: "bge-small-en-v1.5",
            description: "Modern balanced model (recommended)",
            category: "balanced",
            dimensions: 384,
            parameters: "110M",
        },
        ModelInfo {
            name: "bge-base-en-v1.5",
            description: "High-quality embeddings with more dimensions",
            category: "accurate",
            dimensions: 768,
            parameters: "110M",
        },
        ModelInfo {
            name: "bge-large-en-v1.5",
            description: "Large model for maximum accuracy",
            category: "accurate",
            dimensions: 1024,
            parameters: "335M",
        },
        ModelInfo {
            name: "nomic-embed-text-v1.5",
            description: "State-of-the-art embeddings",
            category: "accurate",
            dimensions: 768,
            parameters: "137M",
        },
        ModelInfo {
            name: "multilingual-e5-small",
            description: "Multilingual support (small)",
            category: "multilingual",
            dimensions: 384,
            parameters: "110M",
        },
        ModelInfo {
            name: "multilingual-e5-base",
            description: "Multilingual support (balanced)",
            category: "multilingual",
            dimensions: 768,
            parameters: "110M",
        },
        ModelInfo {
            name: "mxbai-embed-large-v1",
            description: "Large model from MixedBread AI",
            category: "accurate",
            dimensions: 1024,
            parameters: "335M",
        },
    ];

    #[cfg(any(feature = "cuda", feature = "metal"))]
    models.push(ModelInfo {
        name: "qwen3-0.6b",
        description: "Qwen3 embedding model [GPU]",
        category: "gpu",
        dimensions: 1024,
        parameters: "600M",
    });

    models
}


#[async_trait]
pub trait EmbeddingsClient: Send + Sync {
    async fn get_embeddings(&self, text: &[&str]) -> Result<Vec<Vec<f32>>>;
    fn model_name(&self) -> &str;
}

/// Get the embedding dimensions for a given model name
pub fn get_embedding_dim(model_name: &str) -> usize {
    get_available_models()
        .iter()
        .find(|m| m.name == model_name)
        .map(|m| m.dimensions)
        .unwrap_or(384)
}

/// Create an embeddings client based on available features and GPU preference.
///
/// When built with `cuda` or `metal` features and `no_gpu` is false,
/// tries to create a Qwen3 GPU client first. Falls back to ONNX if
/// no GPU is detected or if the user opted out with `--no-gpu`.
#[allow(unused_variables)]
pub fn create_client(model: Option<&str>, no_gpu: bool) -> Box<dyn EmbeddingsClient> {
    #[cfg(any(feature = "cuda", feature = "metal"))]
    if !no_gpu {
        if let Some(client) = qwen3::Qwen3EmbeddingsClient::try_new(model) {
            return Box::new(client);
        }
        eprintln!("Warning: GPU feature enabled but no GPU detected, falling back to ONNX backend");
    }

    Box::new(fastembed::FastEmbeddingsClient::new(model))
}
