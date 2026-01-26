---
mode: edit
context_files:
  - src/commands/run.rs
target_files:
  - src/commands/run.rs
output_dir: src/commands/
output_file: run.rs
---

# Add Quiet Mode and Exit Code Handling to run.rs

This job implements:
- Task 1: Quiet mode support (suppress output when quiet flag is set)
- Task 6: Proper exit code handling (0=success, 1=failure)

## Edit 1: Add quiet field to RunOptions struct

Add the quiet field to RunOptions:

```rust
pub struct RunOptions {
    /// Specific job to run (if None, run all pending)
    pub job_id: Option<String>,
    /// Resume stuck jobs
    pub resume: bool,
    /// Reset specific job to created status
    pub reset: Option<String>,
    /// Model override
    pub model: Option<String>,
    /// URL override
    pub url: Option<String>,
    /// Timeout override
    pub timeout: Option<u64>,
    /// Disable streaming output
    pub no_stream: bool,
    /// Stop processing when any job fails
    pub stop_on_fail: bool,
    /// Enable batch mode with dependency-based parallel execution
    pub batch: bool,
    /// Maximum concurrent jobs (0 = unlimited)
    pub max_concurrent: usize,
    /// Suppress all output except errors
    pub quiet: bool,
}
```

## Edit 2: Update Default implementation

Add quiet: false to the Default impl:

```rust
impl Default for RunOptions {
    fn default() -> Self {
        Self {
            job_id: None,
            resume: false,
            reset: None,
            model: None,
            url: None,
            timeout: None,
            no_stream: false,
            stop_on_fail: false,
            batch: false,
            max_concurrent: 0,
            quiet: false,
        }
    }
}
```

## Edit 3: Make run_jobs return a result with failure count

Change the return type and add quiet-aware output. The function should:
- Suppress println! calls when options.quiet is true
- Return with appropriate exit behavior for scripting

Replace the run_jobs function with this version that respects quiet mode:

```rust
/// Run jobs
pub async fn run_jobs(project_root: &PathBuf, options: RunOptions) -> Result<(), WorkSplitError> {
    let config = load_config(
        project_root,
        options.model,
        options.url,
        options.timeout,
        options.no_stream,
    )?;

    let mut runner = Runner::new(config, project_root.clone())?;
    let quiet = options.quiet;

    // Handle reset
    if let Some(job_id) = options.reset {
        runner.reset_job(&job_id)?;
        if !quiet {
            println!("Reset job '{}' to created status", job_id);
        }
        return Ok(());
    }

    // Run specific job or all jobs
    if let Some(job_id) = options.job_id {
        if !quiet {
            info!("Running single job: {}", job_id);
        }
        let result = runner.run_single(&job_id).await?;
        
        if !quiet {
            print_job_result(&result.job_id, result.status, result.error.as_deref(), result.output_lines);
        }
        
        // Exit with error if job failed
        if result.status == JobStatus::Fail {
            if options.stop_on_fail && !quiet {
                println!("\nStopping due to failure (--stop-on-fail)");
            }
            std::process::exit(1);
        }
    } else if options.batch {
        if !quiet {
            info!("Running in batch mode");
        }
        let summary = runner.run_batch(options.resume, options.stop_on_fail, options.max_concurrent).await?;
        
        if !quiet {
            println!("\n=== Batch Run Summary ===");
            println!("Processed: {}", summary.processed);
            println!("Passed:    {}", summary.passed);
            println!("Failed:    {}", summary.failed);
            if summary.skipped > 0 {
                println!("Skipped:   {} (not processed)", summary.skipped);
            }
            
            if !summary.results.is_empty() {
                println!("\nResults:");
                for result in &summary.results {
                    print_job_result(&result.job_id, result.status, result.error.as_deref(), result.output_lines);
                }
            }
        }
        
        if summary.failed > 0 {
            if options.stop_on_fail && !quiet {
                println!("\nStopping due to failure (--stop-on-fail)");
            }
            std::process::exit(1);
        }
    } else {
        if !quiet {
            info!("Running all pending jobs");
        }
        let summary = runner.run_all(options.resume, options.stop_on_fail).await?;
        
        if !quiet {
            println!("\n=== Run Summary ===");
            println!("Processed: {}", summary.processed);
            println!("Passed:    {}", summary.passed);
            println!("Failed:    {}", summary.failed);
            if summary.skipped > 0 {
                println!("Skipped:   {} (not processed)", summary.skipped);
            }
            
            if !summary.results.is_empty() {
                println!("\nResults:");
                for result in &summary.results {
                    print_job_result(&result.job_id, result.status, result.error.as_deref(), result.output_lines);
                }
            }
        }
        
        // Exit with error if any job failed
        if summary.failed > 0 {
            if options.stop_on_fail && !quiet {
                println!("\nStopping due to failure (--stop-on-fail)");
            }
            std::process::exit(1);
        }
    }

    Ok(())
}
```

## Constraints

- Use 4-space indentation
- Keep existing imports (tracing::info is used)
- The exit(1) call handles failure exit code; main.rs handles exit(2) for errors
- Suppress all println! and info! calls when quiet is true
