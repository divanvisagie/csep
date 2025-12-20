use std::env;
use std::path::PathBuf;

/// Get the cache directory path in a Linux-style location
/// Uses ~/.cache/csep on all platforms for consistency
pub fn cache_dir() -> PathBuf {
    let home =
        env::var("HOME").unwrap_or_else(|_| env::var("USERPROFILE").unwrap_or(".".to_string()));
    PathBuf::from(home).join(".cache").join("csep")
}

/// Get the data directory path in a Linux-style location
/// Uses ~/.local/share/csep on all platforms for consistency
pub fn data_dir() -> PathBuf {
    let home =
        env::var("HOME").unwrap_or_else(|_| env::var("USERPROFILE").unwrap_or(".".to_string()));
    PathBuf::from(home)
        .join(".local")
        .join("share")
        .join("csep")
}

/// Get the embeddings cache path for a specific model
pub fn embeddings_cache_dir(model_name: &str) -> PathBuf {
    cache_dir().join("embeddings").join(model_name)
}

/// Get the default embeddings cache path (for backward compatibility)
#[allow(dead_code)]
pub fn default_embeddings_cache_dir() -> PathBuf {
    embeddings_cache_dir("all-minilm-l6-v2")
}

/// Get the models cache path
pub fn models_cache_dir() -> PathBuf {
    data_dir().join("models")
}
