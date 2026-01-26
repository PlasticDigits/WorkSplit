---
context_files:
  - src/core/mod.rs
output_dir: src/core/
output_file: mod.rs
---

# Update Core Module to Export parser_edit

## Overview
Update the core module to include and export the new `parser_edit` module which contains edit instruction parsing.

## Requirements

Add the following to `src/core/mod.rs`:

1. Add module declaration:
```rust
pub mod parser_edit;
```

2. Add re-export:
```rust
pub use parser_edit::*;
```

## Expected Output

```rust
pub mod config;
pub mod jobs;
pub mod ollama;
pub mod parser;
pub mod parser_edit;
pub mod runner;
pub mod status;

pub use config::*;
pub use jobs::*;
pub use ollama::*;
pub use parser::*;
pub use parser_edit::*;
pub use runner::*;
pub use status::*;
```

This makes all the edit parsing types and functions available through `crate::core::*`.
