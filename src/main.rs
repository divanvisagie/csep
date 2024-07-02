use args::{Args, SubCommands};
use clap::Parser;
use clients::ollama::OLLAMA_MODELS;
use clients::{
    fastembed::FastEmbeddingsClient, ollama::OllamaEmbeddingsClient, EmbeddingsClientImpl,
};
use spinners::{Spinner, Spinners};
use utils::{cosine_similarity, get_stdin};

use crate::chunker::get_cache_path;

mod args;
mod chunker;
mod clients;
mod feature;
mod files;
mod utils;

const DEFAULT_FLOOR: f32 = 0.2;

#[tokio::main]
async fn main() {
    if cfg!(debug_assertions) {
        tracing_subscriber::fmt::init();
    }

    let args = Args::parse();

    if args.list_models {
        println!("Available models:");
        for model in OLLAMA_MODELS.iter() {
            println!("  - {}", model);
        }
        return;
    }

    let floor = args.floor.unwrap_or(DEFAULT_FLOOR);

    let embeddings_client =
        EmbeddingsClientImpl::FastEmbed(FastEmbeddingsClient::new());

    if let Some(subcmd) = args.subcmd {
        match subcmd {
            SubCommands::Cache(cache_args) => {
                if cache_args.clear {
                    let path = get_cache_path();
                    if path.exists() {
                        match std::fs::remove_dir_all(path) {
                            Ok(_) => println!("Cache cleared"),
                            Err(err) => eprintln!("Error clearing cache: {}", err),
                        }
                    } else {
                        println!("Cache is already clear");
                    }
                    return;
                }
                let mut spinner =
                    Spinner::new(Spinners::Dots9, "Building embeddings cache...".into());

                let run_result = feature::default::run(
                    &embeddings_client,
                    &"".to_string(),
                    &floor,
                    &true,
                    &args.vimgrep,
                    &false,
                )
                .await;

                match run_result {
                    Ok(_) => return,
                    Err(err) => eprintln!("Error while running: {}", err),
                }
                spinner.stop()
            }
        }
        return;
    }

    let mut search_phrase = args.query.unwrap_or("".to_string());
    let stdin_text = get_stdin();
    if !stdin_text.is_empty() {
        search_phrase = stdin_text;
    }

    if let Some(comparison) = args.comparison {
        let run_result = feature::comparison::run(search_phrase, comparison, &args.model).await;

        match run_result {
            Ok(_) => return,
            Err(err) => eprintln!("Error while doing comparison: {}", err),
        }
        return;
    }

    let run_result = feature::default::run(
        &embeddings_client,
        &search_phrase,
        &floor,
        &args.no_query,
        &args.vimgrep,
        &true,
    )
    .await;

    match run_result {
        Ok(_) => return,
        Err(err) => eprintln!("Error while running: {}", err),
    }
}
