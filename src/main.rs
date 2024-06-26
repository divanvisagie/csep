use args::{Args, SubCommands};
use clap::Parser;
use clients::ollama::OLLAMA_MODELS;
use clients::{ollama::OllamaEmbeddingsClient, EmbeddingsClientImpl};
use spinners::{Spinner, Spinners};
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

    if args.list_models {
        println!("Available models:");
        for model in OLLAMA_MODELS.iter() {
            println!("  - {}", model);
        }
        return;
    }

    let floor = args.floor.unwrap_or(DEFAULT_FLOOR);
    let embeddings_client = EmbeddingsClientImpl::Ollama(OllamaEmbeddingsClient::new(&args.model));

    // If we are using the build subcommand
    if let Some(subcmd) = args.subcmd {
        match subcmd {
            SubCommands::Cache(cache_args) => {
                if cache_args.clear {
                    println!("Clearing cache");
                    return;
                }
                // if cache_args.build {
                // }
                let mut spinner = Spinner::new(Spinners::Dots9, "Building embeddings cache...".into());
                match feature::default::run(
                    &embeddings_client,
                    &"".to_string(),
                    &floor,
                    &true,
                    &args.vimgrep,
                    &false,
                ) {
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
        match feature::comparison::run(search_phrase, comparison, &args.model) {
            Ok(_) => return,
            Err(err) => eprintln!("Error while doing comparison: {}", err),
        }
        return;
    }
    match feature::default::run(
        &embeddings_client,
        &search_phrase,
        &floor,
        &args.no_query,
        &args.vimgrep,
        &true,
    ) {
        Ok(_) => return,
        Err(err) => eprintln!("Error while running: {}", err),
    }
}
