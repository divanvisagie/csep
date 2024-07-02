use crate::{
    chunker::{chunk_file_with_embeddings, Chunk},
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
        println!("{}:{}:0:{}", self.file, self.line, self.similarity);
    }
}

pub async fn run(
    embeddings_client: &EmbeddingsClientImpl,
    search_phrase: &String,
    floor: &f32,
    no_query: &bool,
    vimgrep: &bool,
    should_print: &bool,
) -> Result<()> {
    let text = search_phrase.clone();
    let search_phrase_embeddings = embeddings_client
        .get_embeddings(search_phrase.clone())
        .await?;
    let search_chunk = Chunk {
        line: 0,
        text: text.clone(),
        embeddings: search_phrase_embeddings,
    };

    let current_dir = std::env::current_dir()?.clone();
    let current_directory = match current_dir.to_str() {
        Some(dir) => dir,
        None => panic!("Could not get current directory"),
    };
    let files = get_all_files_in_directory(current_directory);

    let mut printable_chunk = Vec::new();

    for file in files {
        let chunks = chunk_file_with_embeddings(file.as_str(), &embeddings_client).await?;
        let printable_chunks = chunks.iter().filter(|chunk| {
            let similarity = cosine_similarity(&search_chunk.embeddings, &chunk.embeddings);
            similarity > *floor
        });

        if printable_chunks.clone().count() == 0 {
            continue;
        }

        let mut printable_chunks = printable_chunks
            .map(|chunk| PrintableChunk {
                line: chunk.line,
                file: file.to_string(),
                chunk: chunk.text.clone(),
                similarity: cosine_similarity(&search_chunk.embeddings, &chunk.embeddings),
            })
            .collect::<Vec<PrintableChunk>>();

        // sort by similarity descending
        printable_chunks.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());

        printable_chunk.push(printable_chunks);
    }

    // let printable_chunk = files
    //     .iter()
    //     .filter_map(|file| {
    //         let chunks = match chunk_file_with_embeddings(file, &embeddings_client).await? {
    //             Ok(chunks) => chunks,
    //             Err(err) => {
    //                 eprintln!("Error chunking file {}: {}", file, err);
    //                 return None;
    //             }
    //         };
    //
    //         let printable_chunks = chunks.iter().filter(|chunk| {
    //             let similarity = cosine_similarity(&search_chunk.embeddings, &chunk.embeddings);
    //             similarity > *floor
    //         });
    //
    //         if printable_chunks.clone().count() == 0 {
    //             return None;
    //         }
    //
    //         let mut printable_chunks = printable_chunks
    //             .map(|chunk| PrintableChunk {
    //                 line: chunk.line,
    //                 file: file.to_string(),
    //                 chunk: chunk.text.clone(),
    //                 similarity: cosine_similarity(&search_chunk.embeddings, &chunk.embeddings),
    //             })
    //             .collect::<Vec<PrintableChunk>>();
    //
    //         // sort by similarity descending
    //         printable_chunks.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap());
    //
    //         Some(printable_chunks)
    //     })
    //     .collect::<Vec<Vec<PrintableChunk>>>();

    if !no_query && !vimgrep {
        println!("Results for search phrase: {}\n", text);
    }

    if *should_print {
        let mut printable_chunk = printable_chunk
            .iter()
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
