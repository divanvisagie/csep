use crate::{
    chunker::get_chunks_and_embeddings_or_load_from_cache,
    clients::{self, EmbeddingsClient},
    db,
    files::get_all_files_in_directory,
};
use anyhow::Result;
use std::collections::HashSet;
use std::path::Path;

pub struct PrintableChunk {
    file: String,
    line: usize,
    #[allow(dead_code)]
    chunk: String,
    display_line: String,
    similarity: f32,
}

impl PrintableChunk {
    // print in vimgrep compatible format
    pub fn print_vimgrep(&self) {
        println!("{}:{}:0:{}", self.file, self.line, self.display_line);
    }

    pub fn print_file_heading(&self, colorize: bool) {
        if colorize {
            let reset = "\x1b[0m";
            let file_color = "\x1b[31m";
            println!("{}{}{}", file_color, self.file, reset);
        } else {
            println!("{}", self.file);
        }
    }

    pub fn print_match_line(&self, colorize: bool, token_set: &HashSet<String>) {
        if colorize {
            let reset = "\x1b[0m";
            let line_color = "\x1b[32m";
            let match_color = "\x1b[31m";
            let highlighted = highlight_line(&self.display_line, token_set, match_color, reset);
            println!("{}{}{}:{}", line_color, self.line, reset, highlighted);
        } else {
            println!("{}:{}", self.line, self.display_line);
        }
    }
}

fn tokenize_query(query: &str) -> Vec<String> {
    query
        .split_whitespace()
        .map(|token| token.to_lowercase())
        .filter(|token| !token.is_empty())
        .collect()
}

fn is_word_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

fn highlight_line(
    line: &str,
    token_set: &HashSet<String>,
    match_color: &str,
    reset: &str,
) -> String {
    if token_set.is_empty() {
        return line.to_string();
    }

    let mut output = String::with_capacity(line.len());
    let mut iter = line.char_indices().peekable();
    while let Some((idx, ch)) = iter.next() {
        if is_word_char(ch) {
            let start = idx;
            let mut end = idx + ch.len_utf8();
            while let Some(&(next_idx, next_ch)) = iter.peek() {
                if is_word_char(next_ch) {
                    iter.next();
                    end = next_idx + next_ch.len_utf8();
                } else {
                    break;
                }
            }
            let word = &line[start..end];
            if token_set.contains(&word.to_lowercase()) {
                output.push_str(match_color);
                output.push_str(word);
                output.push_str(reset);
            } else {
                output.push_str(word);
            }
        } else {
            output.push(ch);
        }
    }
    output
}

fn select_display_line_from_text(
    query_tokens: &[String],
    text: &str,
    chunk_line: usize,
) -> (usize, String) {
    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        return (chunk_line, String::new());
    }

    let mut best_idx = 0usize;
    let mut best_score = 0usize;
    for (idx, line) in lines.iter().enumerate() {
        let lower = line.to_lowercase();
        let score = query_tokens
            .iter()
            .filter(|token| lower.contains(token.as_str()))
            .count();
        if score > best_score {
            best_score = score;
            best_idx = idx;
        }
    }

    let line_count = lines.len();
    let start_line = chunk_line.saturating_sub(line_count);
    let line_number = start_line + best_idx;
    (line_number, lines[best_idx].to_string())
}

#[allow(clippy::too_many_arguments)]
pub async fn run(
    embeddings_client: &dyn EmbeddingsClient,
    search_phrase: &str,
    floor: &f32,
    _no_query: &bool,
    vimgrep: &bool,
    should_print: &bool,
    globs: &[String],
    model_name: &str,
) -> Result<()> {
    let current_dir = std::env::current_dir()?;
    let current_directory = match current_dir.to_str() {
        Some(dir) => dir,
        None => panic!("Could not get current directory"),
    };
    let files = get_all_files_in_directory(current_directory, globs)?;

    // Phase A: Sync all files to the database
    let dim = clients::get_embedding_dim(model_name);
    let database = db::open_or_create(model_name, dim).await?;
    let conn = database.connect()?;

    for file in &files {
        if let Err(err) =
            get_chunks_and_embeddings_or_load_from_cache(file.as_str(), embeddings_client, &conn)
                .await
        {
            eprintln!("Error chunking file {}: {}", file, err);
        }
    }

    // Phase B: Search using vector index
    if *should_print && !search_phrase.is_empty() {
        let query_tokens = tokenize_query(search_phrase);
        let token_set: HashSet<String> = query_tokens.iter().cloned().collect();

        let search_phrase_embeddings =
            embeddings_client.get_embeddings(&[search_phrase]).await?;
        let search_phrase_embeddings = &search_phrase_embeddings[0];

        // Use a generous top_k to account for directory filtering
        let top_k = 100;
        let dir_prefix = format!("{}/", current_directory);
        let results =
            db::vector_search(&conn, search_phrase_embeddings, top_k, &dir_prefix).await?;

        let mut printable_chunks: Vec<PrintableChunk> = results
            .into_iter()
            .filter_map(|result| {
                let similarity = 1.0 - result.distance;
                if similarity <= *floor {
                    return None;
                }
                let (line_number, display_line) =
                    select_display_line_from_text(&query_tokens, &result.text, result.line);
                let relative_path = Path::new(&result.file_path)
                    .strip_prefix(current_directory)
                    .unwrap_or(Path::new(&result.file_path))
                    .to_string_lossy()
                    .to_string();
                Some(PrintableChunk {
                    line: line_number,
                    file: relative_path,
                    chunk: result.text,
                    display_line,
                    similarity,
                })
            })
            .collect();

        printable_chunks.sort_by(|a, b| {
            a.file
                .cmp(&b.file)
                .then_with(|| a.line.cmp(&b.line))
                .then_with(|| b.similarity.partial_cmp(&a.similarity).unwrap())
        });

        let colorize = !vimgrep && atty::is(atty::Stream::Stdout);
        let mut last_file: Option<&str> = None;
        for p in &printable_chunks {
            if *vimgrep {
                p.print_vimgrep();
                continue;
            }
            if last_file != Some(p.file.as_str()) {
                if last_file.is_some() {
                    println!();
                }
                p.print_file_heading(colorize);
                last_file = Some(p.file.as_str());
            }
            p.print_match_line(colorize, &token_set);
        }
    }

    Ok(())
}
