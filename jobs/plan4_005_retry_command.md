---
context_files:
  - src/commands/reset.rs
  - src/commands/run.rs
output_dir: src/commands/
output_file: retry.rs
---

# Add retry command (reset + run)

Create a new command file: `src/commands/retry.rs`.

## Requirements
- Provide a public async function:
  - `pub async fn retry_job(project_root: &PathBuf, job_id: &str) -> Result<(), WorkSplitError>`
- Behavior:
  - Reset the job to `created` (same behavior as `reset_jobs(project_root, job_id, None)`).
  - Immediately run the job by calling `run_jobs` with `RunOptions`:
    - `job_id: Some(job_id.to_string())`
    - `dry_run: false`, `resume: false`, `reset: None`
    - Keep other fields at default.
- Do not swallow errors from reset or run.

## Constraints
- No changes to CLI parsing here (handled in `src/main.rs`).
- Do not add new dependencies.
