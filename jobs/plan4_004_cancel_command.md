---
context_files:
  - src/core/status.rs
  - src/commands/reset.rs
output_dir: src/commands/
output_file: cancel.rs
---

# Add cancel command for running jobs

Create a new command file: `src/commands/cancel.rs`.

## Requirements
- Provide a public function:
  - `pub fn cancel_jobs(project_root: &PathBuf, job_id: &str) -> Result<(), WorkSplitError>`
- Use `StatusManager` running-job registry (from `core/status.rs`) to find running jobs.
- Behavior:
  - If `job_id == "all"`: cancel all running jobs.
  - Else: cancel only the matching job if it is running.
  - If nothing is running, print a friendly message and return `Ok(())`.
- Cancel implementation:
  - Use `std::process::Command::new("kill")` with `-TERM` on unix.
  - On non-unix targets, return a `WorkSplitError::Io` with a clear message.
  - If kill succeeds, update status to `fail` with message "Cancelled by user" and clear the running entry.
  - If kill fails, return a `WorkSplitError::Io` with stderr content.
- Print concise output similar to other commands:
  - `Cancelled: <job_id> (pid <pid>)`

## Constraints
- Do not add new dependencies.
- Keep output minimal and consistent with existing commands.
