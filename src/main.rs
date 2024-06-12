use args::Args;
use clap::Parser;
use clients::ollama::OLLAMA_MODELS;
use clients::{ollama::OllamaEmbeddingsClient, EmbeddingsClientImpl};
use utils::{cosine_similarity, get_stdin};

mod args;
mod chunker;
mod clients;
mod feature;
mod files;
mod utils;

const DEFAULT_FLOOR: f32 = 0.2;

fn main() {
    let args = Args::parse();
    let mut search_phrase = args.query.unwrap_or("".to_string());

    if let Some(comparison) = args.comparison {
        match feature::comparison::run(search_phrase, comparison, args.model) {
            Ok(_) => return,
            Err(err) => eprintln!("Error while doing comparison: {}", err),
        }
        return;
    }

    if args.list_models {
        println!("Available models:");
        for model in OLLAMA_MODELS.iter() {
            println!("  - {}", model);
        }
        return;
    }

    let stdin_text = get_stdin();
    if !stdin_text.is_empty() {
        search_phrase = stdin_text;
    }

    let floor = args.floor.unwrap_or(DEFAULT_FLOOR);
    let embeddings_client = EmbeddingsClientImpl::Ollama(OllamaEmbeddingsClient::new(args.model));
    match feature::default::run(&embeddings_client, &search_phrase, &floor, &args.no_query, &args.vimgrep) {
        Ok(_) => return,
        Err(err) => eprintln!("Error while running: {}", err),
    }
}
