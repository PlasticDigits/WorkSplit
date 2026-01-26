---
context_files:
  - src/commands/status.rs
  - src/core/status.rs
output_dir: src/commands/
output_file: status.rs
---

# Rewrite status.rs with Summary, JSON, Quiet, and Concise Failures

Rewrite the status command to support:
- Task 1: Quiet mode (minimal output)
- Task 2: Summary mode (single line: "5 passed, 1 failed, 2 pending")
- Task 4: Concise failure messages (one line per failure)
- Task 7: JSON output mode

## New Function Signature

```rust
pub fn show_status(
    project_root: &PathBuf,
    verbose: bool,
    summary: bool,
    json: bool,
    quiet: bool,
) -> Result<(), WorkSplitError>
```

## Output Modes Priority

1. If `quiet`: output nothing, just return Ok/Err
2. If `json`: output JSON object
3. If `summary`: output single summary line
4. Otherwise: normal output (with verbose details if set)

## JSON Output Format

```json
{"passed":5,"failed":1,"pending":2,"created":0,"failures":["job_001","job_002"]}
```

Use serde_json to serialize. The `failures` array lists job IDs that failed.

## Summary Output Format

Single line, no decoration:
```
5 passed, 1 failed, 2 pending
```

Where pending = pending_work + pending_verification + pending_test + pending_test_run

## Concise Failure Messages (in verbose mode)

Instead of printing verbose context, show one line per failure:
```
FAIL job_001: Missing implementation of delete_user method
```

Truncate error messages to first 80 chars if longer.

## Complete Implementation

```rust
use std::path::PathBuf;
use serde::Serialize;

use crate::core::{JobsManager, StatusManager};
use crate::error::WorkSplitError;
use crate::models::{JobStatus, LimitsConfig};

/// JSON output structure for status command
#[derive(Serialize)]
struct StatusJson {
    passed: usize,
    failed: usize,
    pending: usize,
    created: usize,
    failures: Vec<String>,
}

/// Show job status
pub fn show_status(
    project_root: &PathBuf,
    verbose: bool,
    summary: bool,
    json: bool,
    quiet: bool,
) -> Result<(), WorkSplitError> {
    let jobs_manager = JobsManager::new(project_root.clone(), LimitsConfig::default());
    
    if !jobs_manager.jobs_folder_exists() {
        return Err(WorkSplitError::JobsFolderNotFound(
            project_root.join("jobs"),
        ));
    }

    // Discover jobs and sync
    let discovered = jobs_manager.discover_jobs()?;
    let mut status_manager = StatusManager::new(jobs_manager.jobs_dir())?;
    status_manager.sync_with_jobs(&discovered)?;

    let status_summary = status_manager.get_summary();
    
    // Calculate pending total
    let pending = status_summary.pending_work 
        + status_summary.pending_verification 
        + status_summary.pending_test 
        + status_summary.pending_test_run;

    // Collect failed job IDs
    let entries = status_manager.all_entries();
    let failed_jobs: Vec<String> = entries
        .iter()
        .filter(|e| e.status == JobStatus::Fail)
        .map(|e| e.id.clone())
        .collect();

    // Quiet mode: no output
    if quiet {
        return Ok(());
    }

    // JSON mode
    if json {
        let json_output = StatusJson {
            passed: status_summary.passed,
            failed: status_summary.failed,
            pending,
            created: status_summary.created,
            failures: failed_jobs,
        };
        println!("{}", serde_json::to_string(&json_output).unwrap());
        return Ok(());
    }

    // Summary mode: single line
    if summary {
        println!("{} passed, {} failed, {} pending", 
            status_summary.passed, 
            status_summary.failed, 
            pending);
        return Ok(());
    }

    // Normal mode
    println!("=== WorkSplit Status ===\n");
    println!("Total: {} | Created: {} | Pending: {} | Passed: {} | Failed: {}",
        status_summary.total,
        status_summary.created,
        pending,
        status_summary.passed,
        status_summary.failed);
    println!();

    if verbose {
        let mut sorted: Vec<_> = entries.into_iter().collect();
        sorted.sort_by(|a, b| a.id.cmp(&b.id));

        if sorted.is_empty() {
            println!("No jobs found.");
        } else {
            println!("Jobs:");
            for entry in sorted {
                let status_str = match entry.status {
                    JobStatus::Created => "CREATED",
                    JobStatus::PendingTest => "PENDING TEST",
                    JobStatus::PendingWork => "PENDING WORK",
                    JobStatus::PendingVerification => "PENDING VERIFY",
                    JobStatus::PendingTestRun => "PENDING TEST RUN",
                    JobStatus::Pass => "PASS",
                    JobStatus::Fail => "FAIL",
                };

                if entry.status == JobStatus::Fail {
                    // Concise failure message
                    let error_msg = entry.error.as_deref().unwrap_or("Unknown error");
                    let truncated = if error_msg.len() > 80 {
                        format!("{}...", &error_msg[..77])
                    } else {
                        error_msg.to_string()
                    };
                    println!("  {} [{}]: {}", entry.id, status_str, truncated);
                } else {
                    println!("  {} [{}]", entry.id, status_str);
                }
            }
        }
    }

    // Warn about stuck jobs
    let stuck = status_manager.get_stuck_jobs();
    if !stuck.is_empty() {
        println!("\nWarning: {} stuck job(s) found:", stuck.len());
        for entry in stuck {
            println!("  {} [{}]", entry.id, match entry.status {
                JobStatus::PendingTest => "PENDING TEST",
                JobStatus::PendingWork => "PENDING WORK",
                JobStatus::PendingVerification => "PENDING VERIFY",
                JobStatus::PendingTestRun => "PENDING TEST RUN",
                _ => "STUCK",
            });
        }
        println!("\nUse 'worksplit run --resume' to retry stuck jobs");
        println!("Or 'worksplit run --reset <job_id>' to reset a specific job");
    }

    Ok(())
}
```

## Constraints

- Use 4-space indentation
- Import serde::Serialize for JSON struct
- Keep existing JobsManager and StatusManager usage
- Error message truncation at 80 chars with "..." suffix
