use embeddings::OllamaEmbeddingsClient;

use crate::embeddings::EmbeddingsClient;
mod embeddings;
fn main() {
    let oec = OllamaEmbeddingsClient::new();
    let embeddings = oec.get_embeddings("this is some text".to_string()).unwrap();
    println!("{:?}", embeddings);
}
