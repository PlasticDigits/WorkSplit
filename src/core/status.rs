use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::StatusError;
use crate::models::{JobStatus, JobStatusEntry};

/// Thread-safe wrapper for StatusManager
pub type SharedStatusManager = Arc<RwLock<StatusManager>>;

/// Status file manager
pub struct StatusManager {
    /// Path to the status file
    status_file: PathBuf,
    /// In-memory cache of status entries
    entries: HashMap<String, JobStatusEntry>,
}

impl StatusManager {
    /// Create a new status manager and load existing status
    pub fn new(jobs_dir: &Path) -> Result<Self, StatusError> {
        let status_file = jobs_dir.join("_jobstatus.json");
        let mut manager = Self {
            status_file,
            entries: HashMap::new(),
        };
        manager.load()?;
        Ok(manager)
    }

    /// Create a new thread-safe shared status manager
    pub fn new_shared(jobs_dir: &Path) -> Result<SharedStatusManager, StatusError> {
        let manager = Self::new(jobs_dir)?;
        Ok(Arc::new(RwLock::new(manager)))
    }

    /// Load status from file
    fn load(&mut self) -> Result<(), StatusError> {
        if !self.status_file.exists() {
            debug!("Status file does not exist, starting fresh");
            return Ok(());
        }

        let content = fs::read_to_string(&self.status_file)
            .map_err(|e| StatusError::ReadError(self.status_file.clone(), e))?;

        if content.trim().is_empty() {
            return Ok(());
        }

        let entries: Vec<JobStatusEntry> = serde_json::from_str(&content)
            .map_err(|e| StatusError::ParseError(self.status_file.clone(), e.to_string()))?;

        self.entries = entries.into_iter().map(|e| (e.id.clone(), e)).collect();
        info!("Loaded {} job status entries", self.entries.len());

        Ok(())
    }

    /// Save status to file atomically (write to temp, then rename)
    pub fn save(&self) -> Result<(), StatusError> {
        let entries: Vec<&JobStatusEntry> = self.entries.values().collect();
        let mut sorted_entries: Vec<_> = entries.into_iter().collect();
        sorted_entries.sort_by(|a, b| a.id.cmp(&b.id));

        let json = serde_json::to_string_pretty(&sorted_entries)
            .map_err(|e| StatusError::ParseError(self.status_file.clone(), e.to_string()))?;

        // Write to temporary file first
        let temp_file = self.status_file.with_extension("json.tmp");
        fs::write(&temp_file, &json)
            .map_err(|e| StatusError::WriteError(temp_file.clone(), e))?;

        // Rename atomically
        fs::rename(&temp_file, &self.status_file)
            .map_err(|e| StatusError::WriteError(self.status_file.clone(), e))?;

        debug!("Saved {} job status entries", sorted_entries.len());
        Ok(())
    }

    /// Sync status with discovered job files
    /// - Adds new jobs with "created" status
    /// - Removes entries for deleted job files (with warning)
    pub fn sync_with_jobs(&mut self, discovered_jobs: &[String]) -> Result<(), StatusError> {
        let discovered_set: std::collections::HashSet<&String> = discovered_jobs.iter().collect();

        // Add new jobs
        for job_id in discovered_jobs {
            if !self.entries.contains_key(job_id) {
                info!("Discovered new job: {}", job_id);
                self.entries.insert(job_id.clone(), JobStatusEntry::new(job_id.clone()));
            }
        }

        // Remove deleted jobs (with warning)
        let existing_ids: Vec<String> = self.entries.keys().cloned().collect();
        let to_remove: Vec<String> = existing_ids
            .into_iter()
            .filter(|id| !discovered_set.contains(id))
            .collect();

        for job_id in to_remove {
            warn!("Job file deleted, removing from status: {}", job_id);
            self.entries.remove(&job_id);
        }

        self.save()
    }

    /// Get a job's status
    pub fn get(&self, job_id: &str) -> Option<&JobStatusEntry> {
        self.entries.get(job_id)
    }

    /// Get a mutable reference to a job's status
    pub fn get_mut(&mut self, job_id: &str) -> Option<&mut JobStatusEntry> {
        self.entries.get_mut(job_id)
    }

    /// Update a job's status
    pub fn update_status(&mut self, job_id: &str, status: JobStatus) -> Result<(), StatusError> {
        let entry = self.entries.get_mut(job_id)
            .ok_or_else(|| StatusError::JobNotFound(job_id.to_string()))?;
        entry.update_status(status);
        self.save()
    }

    /// Update multiple job statuses in a single atomic write
    pub fn update_statuses_batch(&mut self, updates: &[(String, JobStatus)]) -> Result<(), StatusError> {
        for (job_id, status) in updates {
            if let Some(entry) = self.entries.get_mut(job_id) {
                entry.update_status(*status);
            }
        }
        self.save()
    }

    /// Set a job as failed with an error message
    pub fn set_failed(&mut self, job_id: &str, error: String) -> Result<(), StatusError> {
        let entry = self.entries.get_mut(job_id)
            .ok_or_else(|| StatusError::JobNotFound(job_id.to_string()))?;
        entry.set_failed(error);
        self.save()
    }

    /// Get all jobs with a specific status
    pub fn get_by_status(&self, status: JobStatus) -> Vec<&JobStatusEntry> {
        self.entries
            .values()
            .filter(|e| e.status == status)
            .collect()
    }

    /// Get all jobs that are stuck (pending_work or pending_verification)
    pub fn get_stuck_jobs(&self) -> Vec<&JobStatusEntry> {
        self.entries
            .values()
            .filter(|e| e.status.is_stuck())
            .collect()
    }

    /// Get all jobs that are ready to be processed (created status)
    pub fn get_ready_jobs(&self) -> Vec<&JobStatusEntry> {
        self.entries
            .values()
            .filter(|e| e.status.is_ready())
            .collect()
    }

    /// Get summary counts
    pub fn get_summary(&self) -> StatusSummary {
        let mut summary = StatusSummary::default();
        for entry in self.entries.values() {
            match entry.status {
                JobStatus::Created => summary.created += 1,
                JobStatus::PendingTest => summary.pending_test += 1,
                JobStatus::PendingWork => summary.pending_work += 1,
                JobStatus::PendingVerification => summary.pending_verification += 1,
                JobStatus::PendingTestRun => summary.pending_test_run += 1,
                JobStatus::Pass => summary.passed += 1,
                JobStatus::Fail => summary.failed += 1,
            }
        }
        summary.total = self.entries.len();
        summary
    }

    /// Reset a job to created status
    pub fn reset_job(&mut self, job_id: &str) -> Result<(), StatusError> {
        let entry = self.entries.get_mut(job_id)
            .ok_or_else(|| StatusError::JobNotFound(job_id.to_string()))?;
        entry.update_status(JobStatus::Created);
        entry.error = None;
        self.save()
    }

    /// Get all entries
    pub fn all_entries(&self) -> Vec<&JobStatusEntry> {
        self.entries.values().collect()
    }
}

/// Summary of job statuses
#[derive(Debug, Default)]
pub struct StatusSummary {
    pub total: usize,
    pub created: usize,
    pub pending_test: usize,
    pub pending_work: usize,
    pub pending_verification: usize,
    pub pending_test_run: usize,
    pub passed: usize,
    pub failed: usize,
}

impl std::fmt::Display for StatusSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let pending = self.pending_test + self.pending_work + self.pending_verification + self.pending_test_run;
        write!(
            f,
            "Total: {} | Created: {} | Pending: {} | Passed: {} | Failed: {}",
            self.total,
            self.created,
            pending,
            self.passed,
            self.failed
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_manager() -> (TempDir, StatusManager) {
        let temp_dir = TempDir::new().unwrap();
        let manager = StatusManager::new(temp_dir.path()).unwrap();
        (temp_dir, manager)
    }

    #[test]
    fn test_new_manager() {
        let (_temp_dir, manager) = create_test_manager();
        assert!(manager.entries.is_empty());
    }

    #[test]
    fn test_sync_with_jobs() {
        let (_temp_dir, mut manager) = create_test_manager();
        
        manager.sync_with_jobs(&["job1".to_string(), "job2".to_string()]).unwrap();
        
        assert_eq!(manager.entries.len(), 2);
        assert!(manager.get("job1").is_some());
        assert!(manager.get("job2").is_some());
        assert_eq!(manager.get("job1").unwrap().status, JobStatus::Created);
    }

    #[test]
    fn test_update_status() {
        let (_temp_dir, mut manager) = create_test_manager();
        
        manager.sync_with_jobs(&["job1".to_string()]).unwrap();
        manager.update_status("job1", JobStatus::PendingWork).unwrap();
        
        assert_eq!(manager.get("job1").unwrap().status, JobStatus::PendingWork);
    }

    #[test]
    fn test_set_failed() {
        let (_temp_dir, mut manager) = create_test_manager();
        
        manager.sync_with_jobs(&["job1".to_string()]).unwrap();
        manager.set_failed("job1", "Test error".to_string()).unwrap();
        
        let entry = manager.get("job1").unwrap();
        assert_eq!(entry.status, JobStatus::Fail);
        assert_eq!(entry.error, Some("Test error".to_string()));
    }

    #[test]
    fn test_get_summary() {
        let (_temp_dir, mut manager) = create_test_manager();
        
        manager.sync_with_jobs(&["job1".to_string(), "job2".to_string(), "job3".to_string()]).unwrap();
        manager.update_status("job2", JobStatus::Pass).unwrap();
        manager.set_failed("job3", "Error".to_string()).unwrap();
        
        let summary = manager.get_summary();
        assert_eq!(summary.total, 3);
        assert_eq!(summary.created, 1);
        assert_eq!(summary.passed, 1);
        assert_eq!(summary.failed, 1);
    }

    #[test]
    fn test_persistence() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create and save
        {
            let mut manager = StatusManager::new(temp_dir.path()).unwrap();
            manager.sync_with_jobs(&["job1".to_string()]).unwrap();
            manager.update_status("job1", JobStatus::Pass).unwrap();
        }
        
        // Load again
        {
            let manager = StatusManager::new(temp_dir.path()).unwrap();
            assert_eq!(manager.get("job1").unwrap().status, JobStatus::Pass);
        }
    }
}
