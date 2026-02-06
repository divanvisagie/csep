use args::{Args, SubCommands};
use clap::Parser;
use spinners::{Spinner, Spinners};
use tracing::error;
use utils::get_stdin;

mod args;
mod chunker;
mod clients;
mod config;
mod db;
mod feature;
mod files;
mod paths;
mod table;
mod utils;

const DEFAULT_FLOOR: f32 = 0.2;

#[tokio::main]
async fn main() {
    if cfg!(debug_assertions) {
        tracing_subscriber::fmt::init();
    }

    let args = Args::parse();

    if args.list_models {
        // Load config to get the actual current model
        let config = config::load_config().unwrap_or_default();
        let config_default_model = config.default_model;
        
        // Determine current model: CLI flag overrides config
        let current_model = if let Some(model) = args.model.as_ref() {
            model.as_str()
        } else {
            &config_default_model
        };

        // Prepare table data
        let mut table_data = vec![];
        
        // Add header
        table_data.push(vec![
            "NAME".to_string(),
            "DESCRIPTION".to_string(),
            "CATEGORY".to_string(),
            "DIM".to_string(),
        ]);
        
        // Add model rows
        let models = clients::get_available_models();
        for model in models {
            let row = vec![
                model.name.to_string(),
                model.description.to_string(),
                model.category.to_string(),
                model.dimensions.to_string(),
            ];
            table_data.push(row);
        }

        let headers = vec![
            "NAME".to_string(),
            "DESCRIPTION".to_string(),
            "CATEGORY".to_string(),
            "DIM".to_string(),
        ];
        
        let rows: Vec<Vec<String>> = table_data.into_iter().skip(1).collect();
        let table_output = table::format_table_with_headers(
            &headers,
            &rows
        );
        
        const DEFAULT_MODEL: &str = "all-minilm-l6-v2";

        // Highlight rows: green for current, blue for default
        let highlighted_output: String = table_output
            .lines()
            .map(|line| {
                if line.starts_with(current_model) {
                    format!("\x1b[32m{}\x1b[0m", line) // Green for current
                } else if line.starts_with(DEFAULT_MODEL) && current_model != DEFAULT_MODEL {
                    format!("\x1b[34m{}\x1b[0m", line) // Blue for default
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        println!("{}", highlighted_output);
        println!("\n\x1b[32m■\x1b[0m Current   \x1b[34m■\x1b[0m Default");
        return;
    }

    let floor = args.floor.unwrap_or(DEFAULT_FLOOR);

    // Load configuration to get default model
    let config = config::load_config().unwrap_or_default();
    let default_model = config.default_model;

    let model_name = args.model.as_deref().unwrap_or(&default_model);
    let embeddings_client = clients::create_client(Some(model_name), args.no_gpu);
    if args.verbose {
        println!("Using model: {} ({})", embeddings_client.model_name(),
            if cfg!(any(feature = "cuda", feature = "metal")) && !args.no_gpu {
                "GPU"
            } else {
                "CPU/ONNX"
            }
        );
    }

    if let Some(subcmd) = args.subcmd {
        match subcmd {
            SubCommands::Cache(cache_args) => {
                if cache_args.clear {
                    // Clear all model caches
                    let base_cache_path = paths::cache_dir().join("embeddings");
                    if base_cache_path.exists() {
                        match std::fs::remove_dir_all(base_cache_path) {
                            Ok(_) => println!("All model caches cleared"),
                            Err(err) => error!("Error clearing cache: {}", err),
                        }
                    } else {
                        println!("Cache is already clear");
                    }
                    return;
                }

                if cache_args.stats {
                    let cache_model_name = embeddings_client.model_name();
                    let dim = clients::get_embedding_dim(cache_model_name);
                    match db::open_or_create(cache_model_name, dim).await {
                        Ok(database) => match database.connect() {
                            Ok(conn) => match db::get_stats(&conn).await {
                                Ok((files, chunks)) => {
                                    println!("Cache statistics for model '{}':", cache_model_name);
                                    println!("  Files:  {}", files);
                                    println!("  Chunks: {}", chunks);
                                }
                                Err(err) => error!("Error getting stats: {}", err),
                            },
                            Err(err) => error!("Error connecting to database: {}", err),
                        },
                        Err(err) => error!("Error opening database: {}", err),
                    }
                    return;
                }

                if cache_args.cleanup {
                    let cache_model_name = embeddings_client.model_name();
                    let dim = clients::get_embedding_dim(cache_model_name);
                    match db::open_or_create(cache_model_name, dim).await {
                        Ok(database) => match database.connect() {
                            Ok(conn) => {
                                // Get all file paths in the database and check which still exist
                                match db::get_all_file_paths(&conn).await {
                                    Ok(paths_in_db) => {
                                        let existing: Vec<String> = paths_in_db
                                            .iter()
                                            .filter(|p| std::path::Path::new(p).exists())
                                            .cloned()
                                            .collect();
                                        match db::remove_missing_files(&conn, &existing).await {
                                            Ok(removed) => {
                                                println!(
                                                    "Removed {} orphaned entries",
                                                    removed
                                                );
                                            }
                                            Err(err) => error!("Error during cleanup: {}", err),
                                        }
                                    }
                                    Err(err) => error!("Error reading file paths: {}", err),
                                }
                            }
                            Err(err) => error!("Error connecting to database: {}", err),
                        },
                        Err(err) => error!("Error opening database: {}", err),
                    }
                    return;
                }

                let mut spinner =
                    Spinner::new(Spinners::Dots9, "Building embeddings cache...".into());

                let run_result = feature::default::run(
                    embeddings_client.as_ref(),
                    "",
                    &floor,
                    &true,
                    &args.vimgrep,
                    &false,
                    &args.glob,
                    embeddings_client.model_name(),
                )
                .await;

                match run_result {
                    Ok(_) => return,
                    Err(err) => eprintln!("Error while running: {}", err),
                }
                spinner.stop()
            }
            SubCommands::Config(config_args) => {
                if config_args.show {
                    match config::load_config() {
                        Ok(config) => {
                            println!("Current configuration:");
                            println!("  Default model: {}", config.default_model);
                            println!("  Config file: {}", config::get_config_path().display());
                        }
                        Err(err) => error!("Error loading config: {}", err),
                    }
                    return;
                }

                if config_args.reset {
                    match config::reset_config() {
                        Ok(_) => println!("Configuration reset to defaults"),
                        Err(err) => error!("Error resetting config: {}", err),
                    }
                    return;
                }

                if let Some(model) = config_args.model {
                    // Validate the model exists
                    let models = clients::get_available_models();
                    let is_valid = models.iter().any(|m| m.name == model);

                    if is_valid {
                        let mut config = config::load_config().unwrap_or_default();
                        config.default_model = model.clone();

                        match config::save_config(&config) {
                            Ok(_) => println!("Default model set to: {}", model),
                            Err(err) => error!("Error saving config: {}", err),
                        }
                    } else {
                        error!(
                            "Invalid model '{}'. Use --list-models to see available models.",
                            model
                        );
                    }
                    return;
                }

                // If no specific config option, show current config
                println!("csep config - manage configuration");
                println!("Usage:");
                println!("  csep config --show          Show current configuration");
                println!("  csep config --model <name>  Set default model");
                println!("  csep config --reset         Reset to defaults");
                return;
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
        let run_result = feature::comparison::run(search_phrase, comparison, embeddings_client.as_ref()).await;

        match run_result {
            Ok(_) => return,
            Err(err) => eprintln!("Error while doing comparison: {}", err),
        }
        return;
    }

    // Determine the model name to use for caching
    let cache_model_name = embeddings_client.model_name();

    let run_result = feature::default::run(
        embeddings_client.as_ref(),
        &search_phrase,
        &floor,
        &args.no_query,
        &args.vimgrep,
        &true,
        &args.glob,
        cache_model_name,
    )
    .await;

    match run_result {
        Ok(_) => return,
        Err(err) => eprintln!("Error while running: {}", err),
    }
}
