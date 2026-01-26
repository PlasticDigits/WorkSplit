---
context_files:
  - src/main.rs
  - src/commands/run.rs
output_dir: src/
output_file: main.rs
output_files:
  - src/main.rs
  - src/commands/run.rs
  - src/commands/mod.rs
  - src/commands/deps.rs
sequential: true
---

# Add CLI Flags and Deps Command

Add --dry-run flag, --continue flag, and new deps subcommand to the CLI.

## Requirements

### 1. Add --dry-run flag to Run command (src/main.rs)

In the `Commands::Run` variant, add:

```rust
/// Preview what edits would be applied without applying them
#[arg(long)]
dry_run: bool,
```

Pass it to RunOptions.

### 2. Add --continue flag to Run command (src/main.rs)

In the `Commands::Run` variant, add:

```rust
/// Resume a partially completed job (retry only failed edits)
#[arg(long, value_name = "JOB_ID")]
continue_job: Option<String>,
```

Pass it to RunOptions.

### 3. Add Deps subcommand (src/main.rs)

Add a new subcommand variant:

```rust
/// Show job dependency graph
Deps {
    /// Show detailed output including file dependencies
    #[arg(short, long)]
    verbose: bool,
    
    /// Output as JSON
    #[arg(long)]
    json: bool,
},
```

Add the match arm to handle it:
```rust
Commands::Deps { verbose, json } => {
    let project_root = std::env::current_dir().unwrap();
    show_deps(&project_root, verbose, json, cli.quiet)
}
```

### 4. Update RunOptions (src/commands/run.rs)

Add new fields:

```rust
pub struct RunOptions {
    // ... existing fields ...
    
    /// Dry-run mode (preview edits without applying)
    pub dry_run: bool,
    
    /// Job ID to continue (retry failed edits only)
    pub continue_job: Option<String>,
}
```

Update the Default impl.

### 5. Handle dry-run in run_jobs (src/commands/run.rs)

Add handling for dry-run mode:

```rust
// In run_jobs function
if options.dry_run {
    if !quiet {
        info!("Running in dry-run mode (no changes will be made)");
    }
    // Call runner.dry_run_job() when implemented
    // For now, print a message
    if !quiet {
        println!("Dry-run mode: would process job(s)");
    }
    return Ok(());
}
```

### 6. Handle --continue in run_jobs (src/commands/run.rs)

Add handling for continue mode:

```rust
// Handle continue (before regular job processing)
if let Some(job_id) = options.continue_job {
    if !quiet {
        info!("Continuing partial job: {}", job_id);
    }
    // Check if job has partial state
    // Retry only the failed edits
    // For now, reset and re-run
    runner.reset_job(&job_id)?;
    let result = runner.run_single(&job_id).await?;
    // ... handle result ...
    return Ok(());
}
```

### 7. Update commands/mod.rs

Add the deps module:

```rust
pub mod deps;
pub use deps::*;
```

### 8. Create deps command (src/commands/deps.rs)

Create a new file with:

```rust
use std::path::PathBuf;
use tracing::info;

use crate::core::{load_config, JobsManager, DependencyGraph};
use crate::error::WorkSplitError;

/// Show job dependency graph
pub fn show_deps(
    project_root: &PathBuf,
    verbose: bool,
    json: bool,
    quiet: bool,
) -> Result<(), WorkSplitError> {
    let config = load_config(project_root, None, None, None, false)?;
    let jobs_manager = JobsManager::new(project_root.clone(), config.limits.clone());
    
    let discovered = jobs_manager.discover_jobs()?;
    
    // Build metadata for dependency analysis
    let jobs_metadata: Vec<(String, crate::models::JobMetadata)> = discovered
        .iter()
        .filter_map(|id| {
            jobs_manager.parse_job(id).ok()
                .map(|job| (id.clone(), job.metadata))
        })
        .collect();
    
    let graph = DependencyGraph::build(&jobs_metadata);
    let groups = graph.execution_groups(&discovered);
    
    if json {
        // Output JSON format
        let output = serde_json::json!({
            "groups": groups.iter().enumerate().map(|(i, g)| {
                serde_json::json!({
                    "group": i + 1,
                    "jobs": g,
                })
            }).collect::<Vec<_>>(),
            "total_jobs": discovered.len(),
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
        return Ok(());
    }
    
    if !quiet {
        println!("Job Dependencies:");
        println!();
        
        for (idx, group) in groups.iter().enumerate() {
            println!("  Group {}: {} job(s)", idx + 1, group.len());
            for job_id in group {
                if verbose {
                    // Show file dependencies
                    if let Ok(job) = jobs_manager.parse_job(job_id) {
                        let deps: Vec<_> = job.metadata.context_files
                            .iter()
                            .map(|p| p.display().to_string())
                            .collect();
                        if deps.is_empty() {
                            println!("    - {} (no dependencies)", job_id);
                        } else {
                            println!("    - {} (depends on: {})", job_id, deps.join(", "));
                        }
                    }
                } else {
                    println!("    - {}", job_id);
                }
            }
            println!();
        }
        
        println!("Execution Groups: {}", groups.len());
        println!("Total Jobs: {}", discovered.len());
    }
    
    Ok(())
}
```

## Constraints

- Keep existing command behavior unchanged
- New flags should have sensible defaults
- Dry-run should not modify any files
- Continue should only work on jobs with Partial status

## Formatting Notes

- Uses 4-space indentation
- Follow existing CLI patterns in main.rs
- Use clap derive macros consistently

## Output Files

This is a sequential multi-file job. Generate in order:
1. src/main.rs - Updated CLI with new flags and subcommand
2. src/commands/run.rs - Updated run_jobs with new option handling
3. src/commands/mod.rs - Add deps module
4. src/commands/deps.rs - New deps command implementation
