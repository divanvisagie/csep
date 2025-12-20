# CSEP Storage Locations (v0.2.0+)

## Overview

CSEP uses two distinct storage locations for different purposes:

## 1. Embeddings Cache

**Location:** `~/.cache/csep/embeddings/{model_name}/`

**Purpose:** Stores computed embeddings for files to avoid recomputation

**Format:**
- Binary files with `.cache` extension
- Named by SHA-256 hash of file contents
- Contains `Vec<Chunk>` structures with line numbers, text, and embeddings
- **Model-specific**: Each model has its own subdirectory

**Examples:**
- `~/.cache/csep/embeddings/all-minilm-l6-v2/` (current default)
- `~/.cache/csep/embeddings/bge-small-en-v1.5/` (future support)

**Content:**
```rust
pub struct Chunk {
    pub line: usize,      // Line number in source file
    pub text: String,     // The actual text chunk
    pub embeddings: Vec<f32>, // Computed embeddings for this chunk
}
```

**Lifetime:**
- Created automatically when files are processed
- Invalidated when file contents change (new SHA-256 hash)
- Can be manually cleared with `csep cache --clear`
- Rebuilt with `csep cache --build`

## 2. Embedding Models Cache

**Location:** `~/.local/share/csep/models/`

**Purpose:** Stores downloaded embedding models (fastembed only)

**Format:**
- Model files downloaded by the fastembed library
- Specific format depends on the embedding model used
- Currently supports: AllMiniLML6V2 (default), other fastembed models

**Content:**
- Downloaded model weights and configuration files
- Managed by the fastembed library's `InitOptions`

**Lifetime:**
- Downloaded automatically on first use
- Persists between csep runs
- Can be manually deleted if needed

## Comparison with Previous Versions

### v0.1.x (Before)

| Type | macOS Location | Linux Location | Windows Location |
|------|----------------|----------------|------------------|
| Cache | `~/Library/Caches/csep/embeddings/` | `~/.cache/csep/embeddings/` | `%LOCALAPPDATA%\csep\embeddings\` |
| Models | `~/Library/Application Support/csep/models/` | `~/.local/share/csep/models/` | `%APPDATA%\csep\models\` |

### v0.2.0+ (After)

| Type | All Platforms Location |
|------|-------------------------|
| Cache | `~/.cache/csep/embeddings/{model_name}/` |
| Models | `~/.local/share/csep/models/` |

## Model-Specific Caching (v0.2.0+)

### Overview

CSEP v0.2.0 introduces **model-specific caching**, where each embedding model has its own cache directory. This prevents cache invalidation when switching between different models.

### Structure

```
~/.cache/csep/embeddings/
├── all-minilm-l6-v2/
│   ├── 00287b4f6aaf6967c00a34c658bfa833c27773bc80721ce4d80dbc0c7bf8be0c.cache
│   ├── 1059139bf2a38193ee55730f08cf894b04bd8d9f8c76dc89cbb1f1d7af02da3d.cache
│   └── ... (other cache files)
└── bge-small-en-v1.5/  # Future models
    └── ... (model-specific cache files)
```

### Benefits

1. **No Cache Invalidation**: Switching models doesn't clear other model caches
2. **Multi-Model Support**: Foundation for future `--model` flag
3. **Better Organization**: Clear separation of model data
4. **Easy Management**: Simple to clear specific model caches
5. **Debugging**: Easier to identify which model generated which cache

### Cache Management

```bash
# Clear all model caches
csep cache --clear

# Future: Clear specific model cache
# csep cache --clear --model all-minilm-l6-v2

# Future: List cached models
# csep cache --list-models
```

## XDG Base Directory Specification

CSEP v0.2.0+ follows the [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/basedir-spec-latest.html):

- **`~/.cache/`**: User-specific non-essential data files (cache)
- **`~/.local/share/`**: User-specific data files (persistent data)

This provides:
- **Consistency**: Same paths on all Unix-like systems
- **Predictability**: Follows established standards
- **Compatibility**: Works well with tools like Neovim, ripgrep, etc.

## Environment Variables

CSEP respects standard environment variables:

- **`$HOME`**: Primary home directory (default: current user's home)
- **`$USERPROFILE`**: Windows fallback for home directory

## Migration Notes

### For Users Upgrading from v0.1.x

**macOS Users:**
```bash
# Manual migration (optional - cache will rebuild automatically)
mkdir -p ~/.cache/csep/embeddings
cp -r ~/Library/Caches/csep/embeddings/* ~/.cache/csep/embeddings/

mkdir -p ~/.local/share/csep/models  
cp -r ~/Library/Application\ Support/csep/models/* ~/.local/share/csep/models/
```

**Linux Users:**
- No path changes needed
- Cache will be rebuilt automatically on first run

**Windows Users:**
- Cache moves from `%LOCALAPPDATA%\csep\` to `~/.cache/csep/`
- Models move from `%APPDATA%\csep\` to `~/.local/share/csep/`

## Cleanup Commands

```bash
# Clear embeddings cache
csep cache --clear

# Rebuild embeddings cache
csep cache --build

# Manual cleanup (if needed)
rm -rf ~/.cache/csep/embeddings/*
rm -rf ~/.local/share/csep/models/*
```

## Filesystem Permissions

Ensure proper permissions for cache directories:
```bash
mkdir -p ~/.cache/csep/embeddings
mkdir -p ~/.local/share/csep/models
chmod 755 ~/.cache/csep
chmod 755 ~/.local/share/csep
chmod 755 ~/.cache/csep/embeddings
chmod 755 ~/.local/share/csep/models
```

## Troubleshooting

**Issue:** "Permission denied" when writing cache
**Solution:** Ensure directories exist and have proper permissions
```bash
mkdir -p ~/.cache/csep/embeddings
chmod 755 ~/.cache/csep/embeddings
```

**Issue:** Cache not found after upgrade
**Solution:** Let csep rebuild cache automatically or run `csep cache --build`

**Issue:** Models not downloading
**Solution:** Check network connection and permissions for `~/.local/share/csep/models/`
