---
mode: edit
context_files:
  - src/main.rs
target_files:
  - src/main.rs
output_dir: src/
output_file: main.rs
---

# Add CLI Flags and Exit Code Handling to main.rs

This job implements multiple tasks by editing main.rs:
- Task 1: Add global `--quiet`/`-q` flag
- Task 2: Add `--summary`/`-s` flag to Status subcommand
- Task 6: Standardize exit codes (0=success, 1=failure, 2=error)
- Task 7: Add `--json` flag to Status subcommand

## Edit 1: Add quiet flag to global Cli struct

After the existing `verbose` field in the Cli struct, add a quiet flag:

```rust
    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Suppress all output except errors (overrides verbose)
    #[arg(short, long, global = true)]
    quiet: bool,
```

## Edit 2: Add summary and json flags to Status subcommand

The Status subcommand currently has only `verbose`. Add `summary` and `json` flags:

```rust
    /// Show job status
    Status {
        /// Show detailed status for each job
        #[arg(short, long)]
        verbose: bool,

        /// Show only summary line (e.g., "5 passed, 1 failed, 2 pending")
        #[arg(short, long)]
        summary: bool,

        /// Output status as JSON
        #[arg(long)]
        json: bool,
    },
```

## Edit 3: Update the Status command handler

Pass the new flags to show_status. Change:

```rust
        Commands::Status { verbose } => {
            let project_root = std::env::current_dir().unwrap();
            show_status(&project_root, verbose)
        }
```

To:

```rust
        Commands::Status { verbose, summary, json } => {
            let project_root = std::env::current_dir().unwrap();
            show_status(&project_root, verbose, summary, json, cli.quiet)
        }
```

## Edit 4: Update logging setup to respect quiet mode

Change the logging level setup to suppress output when quiet:

```rust
    // Set up logging
    let level = if cli.quiet {
        Level::ERROR  // Only show errors in quiet mode
    } else if cli.verbose {
        Level::DEBUG
    } else {
        Level::INFO
    };
```

## Edit 5: Standardize exit codes in error handling

At the end of main(), improve exit code handling. The current code uses exit(1) for all failures. Keep it but add a comment documenting the exit codes:

```rust
    // Exit codes: 0 = success, 1 = failure (job failed), 2 = error (config/connection)
    if let Err(e) = result {
        if !cli.quiet {
            eprintln!("Error: {}", e);
        }
        // Use exit code 2 for infrastructure errors
        std::process::exit(2);
    }
```

## Edit 6: Pass quiet flag to run_jobs

Update the Run command handler to pass quiet flag:

```rust
        Commands::Run { ... } => {
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
                quiet: cli.quiet,  // Add this field
            };
            run_jobs(&project_root, options).await
        }
```

## Constraints

- Preserve all existing functionality
- Use 4-space indentation (matches existing code)
- Keep clap derive macros format
- Global flags are already using `global = true`
