# csep
[![crates.io](https://img.shields.io/crates/v/csep.svg)](https://crates.io/crates/csep)

Cosine Similarity Embeddings Print

**Version 0.2.0** - Cache location standardized to `~/.cache/csep/embeddings/` on all platforms. See [MIGRATION_GUIDE.md](MIGRATION_GUIDE.md) for details.

```
╭─── ╭────┬───────╮
│    ╰──╮ ├─  ╭───╯  
╰───────╯ ╰── ╵
```
Like Grep (Global Regular Expression Print) takes a regular expression and
prints all the lines that have a match in it, Csep (Cosine Similarity Embeddings
Print) takes an input phrase and prints all the chunks that are similar to it.

The goal of this project is to give users command line access to semantic search
in the same way that grep is used for regular expressions. This not only gives
you a command line semantic search tool on any unix like system,
but also allows you to use it in scripts and pipelines. If you combine it with a
command line llm tool like
[chat-gipity](https://github.com/divanvisagie/chat-gipity) or [Ollama](https://ollama.com/) you could
even potentially perform [RAG](https://www.wikiwand.com/en/Prompt_engineering#Retrieval-augmented_generation) in a simple unix shell script.

## How it works

csep searches files by semantic meaning, so results can match the intent of a query even when the exact words are not present. Embeddings can take time to compute, so csep caches them per directory. The cache is built automatically on first run in a directory, or you can prepare it ahead of time:

```sh
csep cache --build
```

**Cache Location**: `~/.cache/csep/embeddings/{model_name}/` (see [Storage Locations](#storage-locations) for details)

## Usage

Default output mirrors rg: file heading followed by line:match entries, with exact query tokens highlighted.

```sh
# Basic search
csep "main entry point"

# Filter to markdown files
csep -g '*.md' "main entry point"

# vimgrep-compatible output
csep --vimgrep "main entry point"
```

### Example output

Input:
```sh
csep "main entry point"
```

Output:
```
src/main.rs
22:#[tokio::main]
23:async fn main() {
```

Input:
```sh
csep -g '*.md' "main entry point"
```

Output:
```
README.md
12:Print) takes an input phrase and prints all the chunks that are similar to it.
```

## Installation

You can then install csep from this source using:
```sh
cargo install --path .
```

Or you can pull whatever the latest published version is from crates.io with
```sh
cargo install csep
```

### Storage Locations

CSEP uses two storage locations:

- **Embeddings Cache**: `~/.cache/csep/embeddings/` - Stores computed file embeddings
- **Models Cache**: `~/.local/share/csep/models/` - Stores downloaded embedding models (fastembed only)

See [STORAGE_LOCATIONS.md](STORAGE_LOCATIONS.md) for detailed information about cache management and migration.

### Ollama client option
If you want to use the ollama client option, you will need to install ollama and pull the default all-minilm model, or any model you wish to use with the model switch, since ollama currently doesnt suppor pulling the models for embeddings automatically like it does with llms.

```sh
ollama pull all-minilm
```

Per embedding, fastembed is actually much slower, but due to the overhead of making requests to ollama, for large directories, the embeddings cache builds much faster when using fastembed.
