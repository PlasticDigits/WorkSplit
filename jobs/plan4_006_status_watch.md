---
mode: edit
context_files:
  - src/commands/status.rs
target_files:
  - src/commands/status.rs
output_dir: src/commands/
output_file: status.rs
---

# Add status watch mode

## Requirements
- Update signature to:
  - `pub fn show_status(project_root: &PathBuf, verbose: bool, watch: bool) -> Result<(), WorkSplitError>`
- When `watch == false`, preserve existing behavior exactly.
- When `watch == true`:
  - Poll status summary every 2 seconds.
  - Print the summary only when it changes.
  - Avoid repeating the full verbose listing on every tick.
  - Avoid repeating the "stuck jobs" warning unless the stuck list changes.
  - Keep output concise (no extra blank lines on every tick).

## Edit Locations (snippets for exact context)
- Function signature currently:
  ```rust
  pub fn show_status(project_root: &PathBuf, verbose: bool) -> Result<(), WorkSplitError> {
  ```
- Summary printing block:
  ```rust
  println!("=== WorkSplit Status ===\n");
  println!("{}", summary);
  println!();
  ```

## Constraints
- Do not add new dependencies.
