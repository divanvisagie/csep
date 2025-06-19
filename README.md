# csep
[![crates.io](https://img.shields.io/crates/v/csep.svg)](https://crates.io/crates/csep)

Cosine Similarity Embeddings Print

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

## Installation

You can then install csep from this source using:
```sh
cargo install --path .
```

Or you can pull whatever the latest published version is from crates.io with
```sh
cargo install csep
```

### Ollama client option
If you want to use the ollama client option, you will need to install ollama and pull the default all-minilm model, or any model you wish to use with the model switch, since ollama currently doesnt suppor pulling the models for embeddings automatically like it does with llms.

```sh
ollama pull all-minilm
```

Per embedding, fastembed is actually much slower, but due to the overhead of making requests to ollama, for large directories, the embeddings cache builds much faster when using fastembed.

## Embeddings cache

Csep stores chunk embeddings in a SQLite database located under your system
cache directory (for example `~/.cache/csep`). Each embedding model has its own
database file so caches do not clash when you switch models. Performing a
search automatically prunes the cache by removing records for files that no
longer exist or that have changed since they were cached.

The cache can be explicitly managed with the `csep cache` subcommand:

```sh
# build or refresh the cache for the current directory
csep cache --build

# remove any stale entries without deleting the database
csep cache --prune

# delete the cache completely
csep cache --clear
```

