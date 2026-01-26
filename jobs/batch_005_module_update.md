---
mode: edit
context_files:
  - src/core/mod.rs
target_files:
  - src/core/mod.rs
output_dir: src/core/
output_file: mod.rs
---

# Update Core Module to Export DependencyGraph

## Goal

Register the new `dependency.rs` module in `src/core/mod.rs` and export `DependencyGraph`.

## Current mod.rs Structure

The file exports various core modules and types. We need to add the dependency module.

## Required Edit

Add the dependency module declaration and export.

### Edit 1: Add module declaration

After the existing `mod` declarations (look for patterns like `mod jobs;`, `mod status;`, etc.), add:

```rust
mod dependency;
```

### Edit 2: Add to public exports

In the `pub use` statements, add:

```rust
pub use dependency::DependencyGraph;
```

## Expected Result

After the edit, `DependencyGraph` can be used via:
```rust
use crate::core::DependencyGraph;
```
