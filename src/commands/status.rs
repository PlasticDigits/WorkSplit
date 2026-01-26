use std::path::PathBuf;

use crate::core::{JobsManager, StatusManager};
use crate::error::WorkSplitError;
use crate::models::{JobStatus, LimitsConfig};

/// Show job status
pub fn show_status(project_root: &PathBuf, verbose: bool) -> Result<(), WorkSplitError> {
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

    let summary = status_manager.get_summary();

    println!("=== WorkSplit Status ===\n");
    println!("{}", summary);
    println!();

    if verbose {
        let entries = status_manager.all_entries();
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

                print!("  {} [{}]", entry.id, status_str);
                
                if let Some(ref error) = entry.error {
                    print!(" - {}", error);
                }
                
                println!();
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
