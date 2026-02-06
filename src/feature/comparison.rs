use crate::{
    chunker::Chunk,
    clients::EmbeddingsClient,
    utils::cosine_similarity,
};
use anyhow::Result;

pub async fn run(first: String, second: String, client: &dyn EmbeddingsClient) -> Result<()> {

    let first_embeddings = client.get_embeddings(&[first.as_str()]).await?;
    let second_embeddings = client.get_embeddings(&[second.as_str()]).await?;

    let first_chunk = Chunk {
        line: 0,
        text: first.clone(),
        embeddings: first_embeddings[0].to_owned(),
    };
    let second_chunk = Chunk {
        line: 0,
        text: second.clone(),
        embeddings: second_embeddings[0].to_owned(),
    };

    let similarity = cosine_similarity(&first_chunk.embeddings, &second_chunk.embeddings);
    println!("first: {}", first);
    println!("second: {}", second);
    println!("similarity: {}", similarity);
    Ok(())
}
