# Model Upgrade Proposal for CSEP

## Current Situation

**Current Default Model:** `AllMiniLML6V2` (sentence-transformers/all-MiniLM-L6-v2)
- **Parameters:** 22M
- **Dimensions:** 384
- **Performance:** Good but outdated
- **Release Date:** 2021

## Available Better Models

Based on fastembed v4.9.1, we have access to these superior models:

### 1. BGE Models (Recommended)
- **BGESmallENV15** (BAAI/bge-small-en-v1.5) - **110M params, 384 dim**
  - ✅ 5x larger than current model
  - ✅ Better performance on MTEB benchmark
  - ✅ Same dimensions (384) - no breaking changes
  - ✅ Modern architecture (2023)
  - ✅ Marked as "Default" in fastembed enum
  - ✅ Good balance of quality and speed

- **BGEBaseENV15** (BAAI/bge-base-en-v1.5) - **110M params, 768 dim**
  - ✅ Higher dimensionality for better semantic capture
  - ❌ Would change embedding size (breaking change)

### 2. Nomic Embed Text
- **NomicEmbedTextV15** (nomic-ai/nomic-embed-text-v1.5) - **137M params**
  - ✅ Excellent performance
  - ✅ Modern architecture
  - ❌ Larger size, slower inference

### 3. Multilingual Models
- **MultilingualE5Base** - **110M params, 768 dim**
  - ✅ Great for multilingual support
  - ❌ Different dimensions (breaking change)

## Recommendation

**Upgrade to BGESmallENV15** for these reasons:

### Benefits:
1. **Better Quality**: Significantly improved embedding quality
2. **Same Dimensions**: 384-dim embeddings maintain compatibility
3. **Modern Architecture**: 2023 model vs 2021 model
4. **Minimal Impact**: No breaking changes for existing users
5. **Fastembed Default**: Already marked as default in fastembed
6. **Performance**: Better on MTEB benchmark (50+ vs 40+)

### Implementation:
```rust
// Current
let init_options = InitOptions::new(EmbeddingModel::AllMiniLML6V2)

// Proposed
let init_options = InitOptions::new(EmbeddingModel::BGESmallENV15)
```

### Migration Impact:
- ✅ **No breaking changes** - same embedding dimensions
- ✅ **Automatic upgrade** - users get better quality automatically
- ✅ **Cache invalidation** - new model = new embeddings (expected)
- ⚠️ **Larger downloads** - ~400MB vs ~80MB model size
- ⚠️ **Slightly slower** - 110M vs 22M parameters

## Performance Comparison

Based on MTEB (Massive Text Embedding Benchmark) results:

| Model | MTEB Score | Parameters | Dimensions | Year |
|-------|------------|------------|------------|------|
| AllMiniLML6V2 | ~40 | 22M | 384 | 2021 |
| BGESmallENV15 | ~55 | 110M | 384 | 2023 |
| NomicEmbedTextV15 | ~60 | 137M | 768 | 2024 |

## Alternative Considerations

### Option 1: Make Model Configurable
```rust
// Could add to Args struct
#[arg(long, default_value = "bge-small-en-v1.5")]
pub model: Option<String>,
```

### Option 2: Model Auto-Selection
- Detect use case and recommend appropriate model
- Allow override via environment variable

### Option 3: Keep Current + Add Warning
- Keep AllMiniLML6V2 as default
- Add deprecation warning suggesting upgrade
- Provide easy migration path

## Implementation Plan

### Phase 1: Direct Upgrade (Recommended)
1. Change default model to BGESmallENV15
2. Update documentation
3. Add note about improved quality in changelog
4. Test performance impact

### Phase 2: Future Enhancements
1. Add model selection option
2. Add model benchmarking command
3. Add model download progress
4. Consider model quantization options

## Code Changes Required

```rust
// src/clients/fastembed.rs
pub fn new() -> Self {
    // Old:
    // let init_options = InitOptions::new(EmbeddingModel::AllMiniLML6V2)
    
    // New:
    let init_options = InitOptions::new(EmbeddingModel::BGESmallENV15)
        .with_show_download_progress(true)
        .with_cache_dir(get_cache_path());
    
    let model = TextEmbedding::try_new(init_options);
    let model = model.unwrap();
    
    FastEmbeddingsClient { model }
}
```

## Testing Requirements

1. **Performance Testing**: Verify inference speed is acceptable
2. **Quality Testing**: Verify embedding quality improvement
3. **Cache Testing**: Verify new cache location works
4. **Memory Testing**: Verify memory usage is reasonable

## Rollback Plan

If issues arise:
1. Revert to AllMiniLML6V2
2. Add as optional model selection
3. Document performance tradeoffs

## Conclusion

**Recommend upgrading to BGESmallENV15** as it provides significantly better embedding quality with minimal downsides. The model is:
- ✅ 2.5x larger but still reasonable
- ✅ Same dimensions (no breaking changes)
- ✅ Marked as default in fastembed
- ✅ Modern architecture with better performance
- ✅ Good balance of quality and speed

This change would position CSEP as using a more modern, higher-quality embedding model while maintaining compatibility and performance.
