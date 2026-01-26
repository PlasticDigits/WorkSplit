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

# Add CLI Options for Batch Mode

## Goal

Add command-line flags to enable batch/parallel execution mode.

## New CLI Flags

Add to the `Run` command:
- `--batch` - Enable batch mode with dependency analysis
- `--max-concurrent N` - Limit parallel jobs (default: 0 = unlimited)

## Changes to src/main.rs

### Edit 1: Add batch flag to Run command

In the `Commands::Run` variant (around lines 37-69), add after `stop_on_fail`:

```rust
        /// Enable batch mode with parallel execution
        #[arg(long)]
        batch: bool,

        /// Maximum concurrent jobs in batch mode (0 = unlimited)
        #[arg(long, default_value = "0")]
        max_concurrent: usize,
```

### Edit 2: Update run command invocation

In the match arm for `Commands::Run` (around lines 126-148), update to include new fields:

```rust
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
            let project_root = std::env::current_dir().unwrap();
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
            run_jobs(&project_root, options).await
        }
```

## Changes to src/commands/run.rs

### Edit 3: Add fields to RunOptions struct

In `RunOptions` struct (around lines 8-26), add after `stop_on_fail`:

```rust
    /// Enable batch mode with dependency-based parallel execution
    pub batch: bool,
    /// Maximum concurrent jobs (0 = unlimited)
    pub max_concurrent: usize,
```

### Edit 4: Update Default impl

In the `Default` impl (around lines 28-40), add:

```rust
            batch: false,
            max_concurrent: 0,
```

### Edit 5: Update run_jobs function

In `run_jobs` function (around lines 74-76), add batch mode handling:

```rust
    } else if options.batch {
        info!("Running in batch mode");
        let summary = runner.run_batch(options.resume, options.stop_on_fail, options.max_concurrent).await?;
        // ... print summary (same as current run_all)
```

The full updated else branch should be:

```rust
    } else if options.batch {
        info!("Running in batch mode");
        let summary = runner.run_batch(options.resume, options.stop_on_fail, options.max_concurrent).await?;
        
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
        
        if options.stop_on_fail && summary.failed > 0 {
            println!("\nStopping due to failure (--stop-on-fail)");
            std::process::exit(1);
        }
    } else {
        // existing run_all code
    }
```

## Usage Examples

```bash
# Run all jobs in batch mode
worksplit run --batch

# Limit to 2 concurrent jobs
worksplit run --batch --max-concurrent 2

# Batch mode with resume and stop on fail
worksplit run --batch --resume --stop-on-fail
```
