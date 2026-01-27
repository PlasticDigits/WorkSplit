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