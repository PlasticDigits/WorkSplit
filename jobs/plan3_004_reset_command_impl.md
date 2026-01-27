---
mode: replace
output_dir: src/commands/
output_file: reset.rs
---

# Plan3 Task 3: Implement Reset Command

Create the implementation for the `reset` command.

## Requirements

1. Implement `reset_jobs` function in `src/commands/reset.rs`.
2. Support resetting a specific job by ID or "all" failed/partial jobs.
3. Use `StatusManager` to perform the reset.

~~~worksplit:src/commands/reset.rs
use std::path::PathBuf;
use crate::error::WorkSplitError;
use crate::core::status::StatusManager;
use crate::models::status::JobStatus;

pub fn reset_jobs(
    project_root: &PathBuf,
    job_id: &str,
    status_filter: Option<&str>,
) -> Result<(), WorkSplitError> {
    let mut status_manager = StatusManager::new(&project_root.join("jobs"))?;
    
    if job_id == "all" {
        let filter = status_filter.unwrap_or("fail");
        let to_reset: Vec<String> = status_manager
            .all_entries()
            .iter()
            .filter(|e| match filter {
                "fail" => e.status == JobStatus::Fail,
                "partial" => e.status == JobStatus::Partial,
                _ => false,
            })
            .map(|e| e.id.clone())
            .collect();
        
        for id in &to_reset {
            status_manager.reset_job(id)?;
            println!("Reset: {}", id);
        }
        println!("\nReset {} job(s). Run 'worksplit run' to re-execute.", to_reset.len());
    } else {
        status_manager.reset_job(job_id)?;
        println!("Reset: {}", job_id);
    }
    
    Ok(())
}
~~~worksplit
