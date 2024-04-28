use clap::{command, Parser};

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Args {
    /// User search query
    #[arg(index = 1)]
    pub query: Option<String>,

    /// Similarity floor, any result below this floating point will be
    /// filtered out from the results
    #[arg(short = 'f', long)]
    pub floor: Option<f32>,
}
