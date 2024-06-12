use clap::{command, Parser};

#[derive(Parser, Debug)]
#[command(author, version, about)]
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


    // Print in vimgrep compatible mode
    #[arg(short, long)]
    pub vimgrep: bool,

    /// Set the model
    #[arg(short = 'M', long)]
    pub model: Option<String>,
}
