use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use chrono::{DateTime, Duration, Utc};
use tracing::info;

use crate::error::WorkSplitError;
use crate::models::Config;

/// Result of cleanup operation
#[derive(Debug)]
pub struct CleanupResult {
    pub deleted_count: usize,
    pub deleted_jobs: Vec<String>,
}

/// Get file modification time as DateTime<Utc>
fn get_file_modified_time(path: &Path) -> Result<DateTime<Utc>, std::io::Error> {
    let metadata = fs::metadata(path)?;
    let modified = metadata.modified()?;
    let duration = modified.duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    Ok(DateTime::from_timestamp(duration.as_secs() as i64, 0)
        .unwrap_or_else(Utc::now))
}

/// Clean up archived jobs older than X days
pub fn cleanup_archived_jobs(
    project_root: &PathBuf,
    days: Option<u32>,
    dry_run: bool,
) -> Result<CleanupResult, WorkSplitError> {
    let config = Config::load_from_dir(project_root)?;
    
    let archive_dir = project_root.join("jobs").join("archive");
    
    // Return early if archive directory doesn't exist
    if !archive_dir.exists() {
        return Ok(CleanupResult {
            deleted_count: 0,
            deleted_jobs: Vec::new(),
        });
    }
    
    let threshold_days = days.unwrap_or(config.cleanup.days);
    let cutoff = Utc::now() - Duration::days(threshold_days as i64);
    
    let mut deleted_jobs = Vec::new();
    
    // Read all .md files in archive directory
    let entries = fs::read_dir(&archive_dir)?;
    
    for entry in entries {
        let entry = entry?;
        
        let path = entry.path();
        
        // Only process .md files
        if path.extension().map(|e| e != "md").unwrap_or(true) {
            continue;
        }
        
        // Get file modification time
        let modified = match get_file_modified_time(&path) {
            Ok(t) => t,
            Err(_) => continue,
        };
        
        if modified < cutoff {
            let file_name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");
            let job_id = file_name.trim_end_matches(".md");
            let days_old = (Utc::now() - modified).num_days();
            
            if dry_run {
                println!("Would delete: {} ({} days old)", file_name, days_old);
            } else {
                fs::remove_file(&path)?;
                
                println!("Deleted: {} ({} days old)", file_name, days_old);
            }
            
            deleted_jobs.push(job_id.to_string());
        }
    }
    
    Ok(CleanupResult {
        deleted_count: deleted_jobs.len(),
        deleted_jobs,
    })
}

/// Run auto-cleanup after archive completes
pub fn run_auto_cleanup(project_root: &PathBuf) -> Result<(), WorkSplitError> {
    let config = Config::load_from_dir(project_root)?;
    
    if !config.cleanup.enabled {
        return Ok(());
    }
    
    let result = cleanup_archived_jobs(project_root, None, false)?;
    
    if result.deleted_count > 0 {
        info!("Cleaned up {} archived job(s)", result.deleted_count);
    }
    
    Ok(())
}
