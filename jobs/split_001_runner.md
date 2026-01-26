---
mode: split
target_file: src/core/runner.rs
output_dir: src/core/runner/
output_file: mod.rs
output_files:
  - src/core/runner/mod.rs
  - src/core/runner/edit.rs
  - src/core/runner/sequential.rs
  - src/core/runner/verify.rs
---
# Split runner.rs into Directory Module

Split runner.rs into a directory structure using **standalone helper functions** (not impl blocks in submodules).

## File Structure

```
src/core/runner/
  mod.rs        # Runner struct, public API, orchestration
  edit.rs       # process_edit_mode() function
  sequential.rs # process_sequential_mode() function  
  verify.rs     # run_verification(), run_retry() functions
```

## Extraction Plan

### mod.rs (Main Module)
Keep in mod.rs:
- `mod edit; mod sequential; mod verify;` declarations
- `Runner` struct definition (private fields, no changes needed)
- `JobResult` and `RunSummary` structs
- `impl Runner` block with:
  - `new()`, `run_all()`, `run_single()`, `run_job()`
  - `load_context_files_with_implicit()`
  - `is_protected_path()`, `safe_write()`
  - `get_summary()`, `reset_job()`, `status_manager()`, `jobs_manager()`

The `run_job()` method calls into submodule functions:
```rust
if job.metadata.is_edit_mode() {
    let files = edit::process_edit_mode(&self.ollama, ...)?;
} else if job.metadata.is_sequential() {
    let files = sequential::process_sequential_mode(&self.ollama, ...)?;
}
// Then call verify::run_verification(...)
```

### edit.rs (Edit Mode Helper)
Extract as standalone function:
```rust
pub(crate) fn process_edit_mode(
    ollama: &OllamaClient,
    project_root: &Path,
    config: &Config,
    job: &Job,
    context_files: &[(PathBuf, String)],
    edit_prompt: &str,
) -> Result<(Vec<(PathBuf, String)>, Vec<PathBuf>, usize), WorkSplitError>
```
- Takes all needed data as parameters
- Returns (generated_files, full_output_paths, total_lines)

### sequential.rs (Sequential Mode Helper)
Extract as standalone function:
```rust
pub(crate) fn process_sequential_mode(
    ollama: &OllamaClient,
    project_root: &Path,
    config: &Config,
    job: &Job,
    context_files: &[(PathBuf, String)],
    create_prompt: &str,
) -> Result<(Vec<(PathBuf, String)>, Vec<PathBuf>, usize), WorkSplitError>
```

### verify.rs (Verification Helpers)
Extract as standalone functions:
```rust
pub(crate) fn run_verification(
    ollama: &OllamaClient,
    verify_prompt: &str,
    context_files: &[(PathBuf, String)],
    generated_files: &[(PathBuf, String)],
    instructions: &str,
) -> Result<(VerificationResult, Option<String>), WorkSplitError>

pub(crate) fn run_retry(
    ollama: &OllamaClient,
    create_prompt: &str,
    context_files: &[(PathBuf, String)],
    generated_files: &[(PathBuf, String)],
    instructions: &str,
    error_msg: &str,
) -> Result<Vec<(PathBuf, String)>, WorkSplitError>
```

## Import Pattern for Submodules

Each submodule imports from crate, NOT super:
```rust
use std::fs;
use std::path::{Path, PathBuf};
use crate::core::{OllamaClient, extract_code, extract_code_files, ...};
use crate::error::WorkSplitError;
use crate::models::{Config, Job};
```

## Key Points

1. Runner struct fields stay PRIVATE - no pub(crate) needed
2. Submodules have standalone functions, not impl blocks
3. mod.rs calls submodule functions, passing needed data
4. Each function returns what mod.rs needs to continue
