use crate::{
    chunker::Chunk,
    clients::{ollama::OllamaEmbeddingsClient, EmbeddingsClient},
    cosine_similarity,
};
use anyhow::Result;

pub fn run(first: String, second: String, model: &Option<String>) -> Result<()> {
    let model_clone = model.clone();
    let oec = OllamaEmbeddingsClient::new(&model_clone);
    let first_chunk = Chunk {
        line: 0,
        text: first.clone(),
        embeddings: oec.get_embeddings(&first)?,
    };
    let second_chunk = Chunk {
        line: 0,
        text: second.clone(),
        embeddings: oec.get_embeddings(&second)?,
    };

    let similarity = cosine_similarity(&first_chunk.embeddings, &second_chunk.embeddings);
    println!("first: {}", first);
    println!("second: {}", second);
    println!("similarity: {}", similarity);
    Ok(())
}
