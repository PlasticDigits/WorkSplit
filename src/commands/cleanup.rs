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

/// Get the modification time of a file
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
    // Load config
    let config = Config::load_from_dir(project_root)?;

    // Create archive directory path
    let archive_dir = project_root.join("jobs").join("archive");

    // Check if archive directory exists
    if !archive_dir.exists() {
        return Ok(CleanupResult {
            deleted_count: 0,
            deleted_jobs: Vec::new(),
        });
    }

    // Determine threshold days
    let threshold_days = days.unwrap_or(config.cleanup.days);

    // Calculate cutoff time
    let cutoff = Utc::now() - Duration::days(threshold_days as i64);

    // Read archive directory
    let entries = fs::read_dir(&archive_dir)?;

    let mut deleted_count = 0;
    let mut deleted_jobs = Vec::new();

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        // Skip if not a file
        if !path.is_file() {
            continue;
        }

        // Skip if not .md extension
        if path.extension().map_or(false, |ext| ext != "md") {
            continue;
        }

        // Get modified time
        let modified = match get_file_modified_time(&path) {
            Ok(m) => m,
            Err(_) => continue,
        };

        // Check if file is older than cutoff
        if modified < cutoff {
            // Get file name and job_id (trim .md suffix)
            let file_name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            let job_id = file_name.trim_end_matches(".md").to_string();

            // Calculate days old
            let days_old = (Utc::now() - modified).num_days();

            if dry_run {
                println!("Would delete: {} ({} days old)", file_name, days_old);
            } else {
                fs::remove_file(&path)?;
                println!("Deleted: {} ({} days old)", file_name, days_old);
                deleted_count += 1;
                deleted_jobs.push(job_id);
            }
        }
    }

    Ok(CleanupResult {
        deleted_count,
        deleted_jobs,
    })
}

/// Run automatic cleanup based on config
pub fn run_auto_cleanup(project_root: &PathBuf) -> Result<(), WorkSplitError> {
    // Load config
    let config = Config::load_from_dir(project_root)?;

    // Check if cleanup is enabled
    if !config.cleanup.enabled {
        return Ok(());
    }

    // Run cleanup
    let result = cleanup_archived_jobs(project_root, None, false)?;

    // Log if jobs were deleted
    if result.deleted_count > 0 {
        info!("Deleted {} archived jobs", result.deleted_count);
    }

    Ok(())
}