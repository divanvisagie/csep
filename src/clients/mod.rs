use anyhow::Result;
use async_trait::async_trait;

pub mod fastembed;

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
    vec![
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
    ]
}


#[async_trait]
pub trait EmbeddingsClient {
    async fn get_embeddings(&self, text: &[&str]) -> Result<Vec<Vec<f32>>>;
}
