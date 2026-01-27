---
mode: edit
context_files:
  - src/main.rs
  - src/commands/mod.rs
target_files:
  - src/main.rs
  - src/commands/mod.rs
output_dir: src/
output_file: main.rs
---

# Plan3 Task 3: Wire up Reset Command

Add the `reset` command to the CLI and register the module.

## Requirements

1. Update `src/commands/mod.rs`: Add `pub mod reset;`.
2. Update `src/main.rs`: Add `Reset` variant to `Commands` enum.
3. Update `src/main.rs`: Handle `Commands::Reset` in `main`.

## Edit Instructions

FILE: src/commands/mod.rs
FIND:
pub mod run;
REPLACE:
pub mod run;
pub mod reset;
END

FILE: src/main.rs
FIND:
    /// Run pending jobs
    Run {
REPLACE:
    /// Reset job status
    Reset {
        /// Job ID to reset (or "all" for all failed jobs)
        job: String,

        /// Reset all jobs matching status (e.g., "fail", "partial")
        #[arg(long)]
        status: Option<String>,
    },

    /// Run pending jobs
    Run {
END

FILE: src/main.rs
FIND:
        Commands::Run {
            job,
            resume,
            reset,
            model,
            url,
            timeout,
            no_stream,
        } => {
REPLACE:
        Commands::Reset { job, status } => {
            crate::commands::reset::reset_jobs(&project_root, &job, status.as_deref())?;
        }

        Commands::Run {
            job,
            resume,
            reset,
            model,
            url,
            timeout,
            no_stream,
        } => {
END
