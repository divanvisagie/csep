use rayon::prelude::*;
use crate::{
    chunker::{get_chunks_and_embeddings_or_load_from_cache, Chunk},
    clients::{EmbeddingsClient, EmbeddingsClientImpl},
    files::get_all_files_in_directory,
    utils::cosine_similarity,
};
use anyhow::Result;

pub struct PrintableChunk {
    file: String,
    chunk: String,
    line: usize,
    similarity: f32,
}

impl PrintableChunk {
    pub fn print(&self) {
        println!("file: {}", self.file);
        println!("chunk: {}", self.chunk);
        println!("similarity: {}\n", self.similarity);
    }


    // print in vimgrep compatible format
    pub fn print_vimgrep(&self) {
        println!("{}:{}:0:", self.file, self.line);
        // for lines in chunk
        for chunk in self.chunk.lines() {
            println!("  | {}", chunk);
        }
    }
}

pub async fn run(
    embeddings_client: &EmbeddingsClientImpl,
    search_phrase: &str,
    floor: &f32,
    no_query: &bool,
    vimgrep: &bool,
    should_print: &bool,
) -> Result<()> {
    let search_phrase_embeddings = embeddings_client
        .get_embeddings(&[search_phrase])
        .await?;
    let search_phrase_embeddings = &search_phrase_embeddings[0];

    let search_chunk = Chunk {
        line: 0,
        text: search_phrase.to_string(),
        embeddings: search_phrase_embeddings.to_owned()
    };
    

    // Now lets work with the files in the current directory
    let current_dir = std::env::current_dir()?.clone();
    let current_directory = match current_dir.to_str() {
        Some(dir) => dir,
        None => panic!("Could not get current directory"),
    };
    let files = get_all_files_in_directory(current_directory);

    let mut printable_chunk = Vec::new();

    let chunk_futures: Vec<_> = files.par_iter().map(|file| {
         get_chunks_and_embeddings_or_load_from_cache(file.as_str(), embeddings_client)
    }).collect();

    let chunk_results = futures::future::join_all(chunk_futures).await;
    
    for chunk_result in chunk_results {
        let chunks = match chunk_result {
            Ok(chunks) => chunks,
            Err(err) => {
                eprintln!("Error chunking file: {}", err);
                continue;
            }
        };

        let printable_chunks = chunks.1.par_iter().filter(|chunk| {
            let similarity = cosine_similarity(&search_chunk.embeddings, &chunk.embeddings);
            similarity > *floor
        });

        if printable_chunks.clone().count() == 0 {
            continue;
        }

        let mut printable_chunks = printable_chunks
            .map(|chunk| PrintableChunk {
                line: chunk.line,
                file: chunks.0.clone(),
                chunk: chunk.text.clone(),
                similarity: cosine_similarity(&search_chunk.embeddings, &chunk.embeddings),
            })
            .collect::<Vec<PrintableChunk>>();

        // Sort by similarity descending
        printable_chunks.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());

        printable_chunk.push(printable_chunks);
    }

    if !no_query && !vimgrep {
        println!("Results for search phrase: {}\n", search_phrase);
    }

    if *should_print {
        let mut printable_chunk = printable_chunk
            .par_iter()
            .flatten()
            .collect::<Vec<&PrintableChunk>>();
        printable_chunk.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());
        for p in printable_chunk {
            if *vimgrep {
                p.print_vimgrep();
                continue;
            }
            p.print();
        }
    }

    Ok(())
}
