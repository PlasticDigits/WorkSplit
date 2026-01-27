use std::path::PathBuf;
use crate::error::WorkSplitError;
use crate::core::status::StatusManager;

/// Cancel running jobs by marking them as failed.
/// Note: This marks jobs as cancelled but cannot actually kill the Ollama process.
/// The running Ollama request will complete but its output will be discarded.
pub fn cancel_jobs(
    project_root: &PathBuf,
    job_id: &str,
) -> Result<(), WorkSplitError> {
    let mut status_manager = StatusManager::new(&project_root.join("jobs"))?;
    
    // Find jobs in running states (PendingWork or PendingVerification)
    let running_jobs: Vec<String> = status_manager
        .get_stuck_jobs()
        .iter()
        .map(|e| e.id.clone())
        .collect();

    if running_jobs.is_empty() {
        println!("No running or stuck jobs found.");
        return Ok(());
    }

    if job_id == "all" {
        for id in &running_jobs {
            status_manager.set_failed(id, "Cancelled by user".to_string())?;
            println!("Cancelled: {}", id);
        }
        println!("\nCancelled {} job(s).", running_jobs.len());
    } else {
        if !running_jobs.contains(&job_id.to_string()) {
            // Check if the job exists at all
            if status_manager.get(job_id).is_some() {
                println!("Job '{}' is not running.", job_id);
            } else {
                println!("Job '{}' not found.", job_id);
            }
            return Ok(());
        }
        status_manager.set_failed(job_id, "Cancelled by user".to_string())?;
        println!("Cancelled: {}", job_id);
    }

    Ok(())
}
