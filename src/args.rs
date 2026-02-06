use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "csep (Cosine Similarity Embeddings Print) searches files by semantic meaning, similar to how grep searches by regex. Use it when you want content-based matches even if the exact words don't appear.\n\nEmbeddings can take time to compute, so csep caches them per directory. The cache is built automatically on first run in a directory, or you can prepare it with `csep cache --build`."
)]
pub struct Args {
    /// User search query
    #[arg(index = 1)]
    pub query: Option<String>,

    /// If provided will compare the query to this string and then return
    /// the cosine similarity score
    #[arg(index = 2)]
    pub comparison: Option<String>,

    /// Similarity floor, any result below this floating point will be
    /// filtered out from the results
    #[arg(short = 'f', long)]
    pub floor: Option<f32>,

    /// If set will not print out the query with the results
    #[arg(short, long)]
    pub no_query: bool,

    /// List the available embedding models
    #[arg(short, long)]
    pub list_models: bool,

    /// Show verbose output including current model information
    #[arg(long)]
    pub verbose: bool,

    /// Print in vimgrep compatible mode
    #[arg(short, long)]
    pub vimgrep: bool,

    /// Include or exclude files using glob patterns (repeatable)
    #[arg(short = 'g', long = "glob")]
    pub glob: Vec<String>,

    /// Force CPU/ONNX backend even when GPU is available
    #[arg(long, global = true)]
    pub no_gpu: bool,

    /// Set the embedding model to use
    ///
    /// Available models:
    ///   - all-minilm-l6-v2 (default): Fast, general-purpose
    ///   - bge-small-en-v1.5 (recommended): Modern, balanced
    ///   - See --list-models for full list
    #[arg(short = 'M', long)]
    pub model: Option<String>,

    #[command(subcommand)]
    pub subcmd: Option<SubCommands>,
}

#[derive(Parser, Debug)]
pub enum SubCommands {
    /// Options for managing the embeddings cache
    Cache(CacheSubcommand),
    /// Options for managing csep configuration
    Config(ConfigSubcommand),
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CacheSubcommand {
    /// Clear the current session context.
    #[arg(short, long)]
    pub clear: bool,

    /// Build an embeddings cache for the current directory
    /// This subcommand will default to building the cache if
    /// no other option is provided
    #[arg(short, long)]
    pub build: bool,

    /// Remove orphaned entries for files that no longer exist
    #[arg(long)]
    pub cleanup: bool,

    /// Show cache statistics (file count, chunk count)
    #[arg(short, long)]
    pub stats: bool,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct ConfigSubcommand {
    /// Set the default model for all future runs
    #[arg(short, long)]
    pub model: Option<String>,

    /// Show the current configuration
    #[arg(short, long)]
    pub show: bool,

    /// Reset configuration to defaults
    #[arg(short, long)]
    pub reset: bool,
}
