---
mode: edit
context_files:
  - src/main.rs
  - src/commands/run.rs
target_files:
  - src/main.rs
  - src/commands/run.rs
output_dir: src/
output_file: main.rs
---

# Plan3 Task 2: Dry Run Mode

Implement the `--dry-run` flag for the `run` command.

## Requirements

1. Update `src/main.rs`: Add `dry_run: bool` to `Run` command arguments.
2. Update `src/commands/run.rs`: Implement the preview logic when `dry_run` is true.

## Edit Instructions

FILE: src/main.rs
FIND:
        /// Specific job ID to run
        #[arg(short, long)]
        job: Option<String>,

        /// Resume stuck jobs (pending_work or pending_verification)
REPLACE:
        /// Specific job ID to run
        #[arg(short, long)]
        job: Option<String>,

        /// Preview what would run without executing jobs
        #[arg(long)]
        dry_run: bool,

        /// Resume stuck jobs (pending_work or pending_verification)
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
            stop_on_fail,
            batch,
            max_concurrent,
        } => {
            let options = RunOptions {
                job_id: job,
                resume,
                reset,
                model,
                url,
                timeout,
                no_stream,
                stop_on_fail,
                batch,
                max_concurrent,
            };
REPLACE:
        Commands::Run {
            job,
            dry_run,
            resume,
            reset,
            model,
            url,
            timeout,
            no_stream,
            stop_on_fail,
            batch,
            max_concurrent,
        } => {
            let options = RunOptions {
                job_id: job,
                dry_run,
                resume,
                reset,
                model,
                url,
                timeout,
                no_stream,
                stop_on_fail,
                batch,
                max_concurrent,
            };
END

FILE: src/commands/run.rs
FIND:
pub struct RunOptions {
    /// Specific job to run (if None, run all pending)
    pub job_id: Option<String>,
    /// Preview what would run without executing
    pub dry_run: bool,
    /// Resume stuck jobs
    pub resume: bool,
    /// Reset specific job to created status
    pub reset: Option<String>,
REPLACE:
pub struct RunOptions {
    /// Specific job to run (if None, run all pending)
    pub job_id: Option<String>,
    /// Preview what would run without executing jobs
    pub dry_run: bool,
    /// Resume stuck jobs
    pub resume: bool,
    /// Reset specific job to created status
    pub reset: Option<String>,
END

FILE: src/commands/run.rs
FIND:
    if let Some(job_id) = options.job_id {
        info!("Running single job: {}", job_id);
REPLACE:
    if let Some(job_id) = options.job_id {
        info!("Running single job: {}", job_id);
        // Keep behavior, but update dry-run output text to mention jobs
END
