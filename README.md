# csep
[![crates.io](https://img.shields.io/crates/v/csep.svg)](https://crates.io/crates/csep)

Cosine Similarity Embeddings Print

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
You will need to install ollama and pull the all-minilm model in order for csep
to have something to get embeddings from.

```sh
ollama pull all-minilm
```

You can then install csep from this source using:
```sh
cargo install --path .
```

Or you can pull whatever the latest published version is from crates.io with
```sh
cargo install csep
```
