use std::fs;
use std::path::PathBuf;
use chrono::{Duration, Utc};
use tracing::info;

use crate::error::WorkSplitError;
use crate::models::Config;
use crate::core::status::StatusManager;
use crate::models::JobStatus;
use crate::commands::cleanup::run_auto_cleanup;

/// Result of archive operation
#[derive(Debug)]
pub struct ArchiveResult {
    pub archived_count: usize,
    pub archived_jobs: Vec<String>,
}

/// Archive completed jobs older than X days to jobs/archive/
pub fn archive_jobs(
    project_root: &PathBuf,
    days: Option<u32>,
    dry_run: bool,
) -> Result<ArchiveResult, WorkSplitError> {
    let config = Config::load_from_dir(project_root)?;
    
    let jobs_dir = project_root.join("jobs");
    let archive_dir = jobs_dir.join("archive");
    
    let status_manager = StatusManager::new(&jobs_dir)?;
    
    let threshold_days = days.unwrap_or(config.archive.days);
    let cutoff = Utc::now() - Duration::days(threshold_days as i64);
    
    let mut archived_jobs = Vec::new();
    
    // Find all Pass jobs older than cutoff
    for entry in status_manager.all_entries() {
        if entry.status == JobStatus::Pass && entry.updated_at < cutoff {
            let job_file = jobs_dir.join(format!("{}.md", entry.id));
            
            if !job_file.exists() {
                continue;
            }
            
            let days_old = (Utc::now() - entry.updated_at).num_days();
            
            if dry_run {
                println!("Would archive: {} ({} days old)", entry.id, days_old);
            } else {
                // Create archive directory if needed
                if !archive_dir.exists() {
                    fs::create_dir_all(&archive_dir)?;
                }
                
                // Move file to archive
                let archive_file = archive_dir.join(format!("{}.md", entry.id));
                fs::rename(&job_file, &archive_file)?;
                
                println!("Archived: {} ({} days old)", entry.id, days_old);
            }
            
            archived_jobs.push(entry.id.clone());
        }
    }
    
    Ok(ArchiveResult {
        archived_count: archived_jobs.len(),
        archived_jobs,
    })
}

/// Run auto-archive after worksplit run completes
/// Also triggers auto-cleanup if enabled
pub fn run_auto_archive(project_root: &PathBuf) -> Result<(), WorkSplitError> {
    let config = Config::load_from_dir(project_root)?;
    
    if !config.archive.enabled {
        return Ok(());
    }
    
    let result = archive_jobs(project_root, None, false)?;
    
    if result.archived_count > 0 {
        info!("Archived {} job(s) to jobs/archive/", result.archived_count);
    }
    
    // Trigger auto-cleanup after archive
    run_auto_cleanup(project_root)?;
    
    Ok(())
}
