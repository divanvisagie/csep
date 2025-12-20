# Migration Guide: csep 0.1.x → 0.2.0

## Breaking Change: Cache Location Change

**Version 0.2.0** introduces a major change to where csep stores its cache files.

### What Changed

**Before (v0.1.x):**
- macOS: `~/Library/Caches/csep/embeddings/`
- Linux: `~/.cache/csep/embeddings/`
- Windows: `%LOCALAPPDATA%\csep\embeddings\`

**After (v0.2.0):**
- **All platforms**: `~/.cache/csep/embeddings/`

### Why This Change

1. **Consistency**: All platforms now use the same Linux-style cache location
2. **Predictability**: Follows XDG base directory specification like Neovim and other CLI tools
3. **Simplicity**: Removed external `dirs` dependency, using simple environment variables

### Impact

- **Existing caches will not be automatically migrated**
- On first run, csep v0.2.0 will create a new cache in the new location
- Old cache files in the previous location will remain but won't be used
- **Model-specific caching**: Cache is now organized by model name (`~/.cache/csep/embeddings/{model_name}/`)

### Migration Steps

#### Option 1: Let csep rebuild cache automatically
1. Install csep v0.2.0
2. Run your normal csep commands
3. Cache will be rebuilt in the new location automatically

#### Option 2: Manual migration (if you want to preserve existing embeddings)
```bash
# On macOS (if coming from v0.1.x)
mkdir -p ~/.cache/csep/embeddings
cp -r ~/Library/Caches/csep/embeddings/* ~/.cache/csep/embeddings/

# On Linux (no action needed, path is the same)
# The new version will continue using the existing cache

# Clean up old cache (optional)
rm -rf ~/Library/Caches/csep  # macOS only
```

### Verification

Check your new cache location:
```bash
echo "New cache location: $HOME/.cache/csep/embeddings"
ls -la ~/.cache/csep/embeddings/
```

### Additional Changes

- Removed `dirs` dependency
- Added new `paths` module for consistent cross-platform path handling
- Models cache also standardized to `~/.local/share/csep/models/`

### Support

If you encounter any issues with the migration, please:
1. Clear the cache: `csep cache --clear`
2. Rebuild: `csep cache --build`
3. Report issues if problems persist

## Changelog

### v0.2.0 (2025-12-20)
- ✅ **BREAKING**: Standardized cache location to `~/.cache/csep/embeddings/{model_name}/` on all platforms
- ✅ **NEW**: Model-specific caching - each embedding model has its own cache directory
- ✅ Removed `dirs` dependency
- ✅ Added cross-platform path handling module
- ✅ Improved consistency with Linux/XDG standards
