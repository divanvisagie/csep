use anyhow::Result;
use crate::{
    chunker::{chunk_file_with_embeddings, Chunk},
    clients::{EmbeddingsClient, EmbeddingsClientImpl},
    files::get_all_files_in_directory,
    utils::cosine_similarity,
};

struct Document {
    path: String,
    pub chunks: Vec<Chunk>,
}

pub struct PrintableChunk {
    pub chunk: String,
    similarity: f32,
}

pub struct PrintableFile {
    file: String,
    chunks: Vec<PrintableChunk>,
}

impl PrintableFile {
    fn print(&self) {
        println!("file: {}", self.file);
        for chunk in &self.chunks {
            println!("chunk: {}", chunk.chunk);
            println!("similarity: {}\n", chunk.similarity);
        }
    }
}

pub fn run(
    embeddings_client: &EmbeddingsClientImpl,
    search_phrase: &String,
    floor: &f32,
    no_query: &bool,
) -> Result<()> {
    let search_chunk = Chunk {
        text: search_phrase.clone(),
        embeddings: embeddings_client.get_embeddings(&search_phrase)?,
    };

    let current_dir = std::env::current_dir()?.clone();
    let current_directory = match current_dir.to_str() {
        Some(dir) => dir,
        None => panic!("Could not get current directory"),
    };
    let files = get_all_files_in_directory(current_directory);
    let documents = files
        .iter()
        .filter_map(|file| {
            let chunks = match chunk_file_with_embeddings(file, &embeddings_client) {
                Ok(chunks) => chunks,
                Err(err) => {
                    eprintln!("Error chunking file {}: {}", file, err);
                    return None;
                }
            };

            Some(Document {
                path: file.to_string(),
                chunks,
            })
        })
        .collect::<Vec<Document>>();

    if !no_query {
        println!("Results for search phrase: {}\n", search_phrase);
    }

    for document in documents {
        let chunks = document.chunks.iter().filter(|chunk| {
            let similarity = cosine_similarity(&search_chunk.embeddings, &chunk.embeddings);
            similarity > *floor
        });
        if chunks.clone().count() == 0 {
            continue;
        }

        let printable_chunks = chunks
            .map(|chunk| PrintableChunk {
                chunk: chunk.text.clone(),
                similarity: cosine_similarity(&search_chunk.embeddings, &chunk.embeddings),
            })
            .collect::<Vec<PrintableChunk>>();

        let printable_file = PrintableFile {
            file: document.path.clone(),
            chunks: printable_chunks,
        };

        printable_file.print();
    }
    Ok(())
}
