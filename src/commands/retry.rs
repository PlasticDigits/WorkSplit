use std::path::PathBuf;
use crate::error::WorkSplitError;
use crate::commands::reset::reset_jobs;
use crate::commands::run::run_jobs;

/// Retry a failed job by resetting it to created status and running it again.
pub async fn retry_job(project_root: &PathBuf, job_id: &str) -> Result<(), WorkSplitError> {
    // Reset the job to created status
    reset_jobs(project_root, job_id, None)?;
    
    // Run the job immediately
    let options = crate::commands::run::RunOptions {
        job_id: Some(job_id.to_string()),
        dry_run: false,
        resume: false,
        reset: None,
        model: None,
        url: None,
        timeout: None,
        job_timeout: None,
        no_stream: false,
        stop_on_fail: false,
        batch: false,
        max_concurrent: 0,
        rerun: false, // Not needed since reset clears the ran flag
    };
    
    run_jobs(project_root, options).await?;
    
    Ok(())
}