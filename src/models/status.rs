use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Status of a job in the processing pipeline
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    /// Job file exists but hasn't been started
    Created,
    /// Job has been sent to Ollama for test generation (TDD first step)
    PendingTest,
    /// Job has been sent to Ollama for creation
    PendingWork,
    /// Creation complete, waiting for verification
    PendingVerification,
    /// Job has been sent to Ollama for test execution
    PendingTestRun,
    /// Verification passed
    Pass,
    /// Verification failed
    Fail,
}

impl JobStatus {
    /// Check if this status indicates the job is complete
    pub fn is_complete(&self) -> bool {
        matches!(self, JobStatus::Pass | JobStatus::Fail)
    }

    /// Check if this status indicates the job is stuck (intermediate state)
    pub fn is_stuck(&self) -> bool {
        matches!(
            self,
            JobStatus::PendingTest
                | JobStatus::PendingWork
                | JobStatus::PendingVerification
                | JobStatus::PendingTestRun
        )
    }

    /// Check if this status indicates the job is ready to be processed
    pub fn is_ready(&self) -> bool {
        matches!(self, JobStatus::Created)
    }

    /// Check if this status is part of the TDD workflow
    pub fn is_tdd_phase(&self) -> bool {
        matches!(self, JobStatus::PendingTest | JobStatus::PendingTestRun)
    }

    /// Get the next status in the workflow
    pub fn next_status(&self, tdd_enabled: bool) -> Option<JobStatus> {
        match (self, tdd_enabled) {
            (JobStatus::Created, true) => Some(JobStatus::PendingTest),
            (JobStatus::Created, false) => Some(JobStatus::PendingWork),
            (JobStatus::PendingTest, true) => Some(JobStatus::PendingWork),
            (JobStatus::PendingWork, true) | (JobStatus::PendingWork, false) => {
                Some(JobStatus::PendingVerification)
            }
            (JobStatus::PendingVerification, true) => Some(JobStatus::PendingTestRun),
            (JobStatus::PendingVerification, false) => None,
            (JobStatus::PendingTestRun, true) => None,
            _ => None,
        }
    }
}

/// Entry in the job status file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStatusEntry {
    /// Job identifier
    pub id: String,
    /// Current status
    pub status: JobStatus,
    /// When the job was first discovered
    pub created_at: DateTime<Utc>,
    /// When the status was last updated
    pub updated_at: DateTime<Utc>,
    /// Error message if the job failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl JobStatusEntry {
    /// Create a new job status entry with Created status
    pub fn new(id: String) -> Self {
        let now = Utc::now();
        Self {
            id,
            status: JobStatus::Created,
            created_at: now,
            updated_at: now,
            error: None,
        }
    }

    /// Update the status and timestamp
    pub fn update_status(&mut self, status: JobStatus) {
        self.status = status;
        self.updated_at = Utc::now();
        if status != JobStatus::Fail {
            self.error = None;
        }
    }

    /// Set the status to failed with an error message
    pub fn set_failed(&mut self, error: String) {
        self.status = JobStatus::Fail;
        self.updated_at = Utc::now();
        self.error = Some(error);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_status_is_complete() {
        assert!(JobStatus::Pass.is_complete());
        assert!(JobStatus::Fail.is_complete());
        assert!(!JobStatus::Created.is_complete());
        assert!(!JobStatus::PendingTest.is_complete());
        assert!(!JobStatus::PendingWork.is_complete());
        assert!(!JobStatus::PendingVerification.is_complete());
        assert!(!JobStatus::PendingTestRun.is_complete());
    }

    #[test]
    fn test_job_status_is_stuck() {
        assert!(JobStatus::PendingTest.is_stuck());
        assert!(JobStatus::PendingWork.is_stuck());
        assert!(JobStatus::PendingVerification.is_stuck());
        assert!(JobStatus::PendingTestRun.is_stuck());
        assert!(!JobStatus::Created.is_stuck());
        assert!(!JobStatus::Pass.is_stuck());
        assert!(!JobStatus::Fail.is_stuck());
    }

    #[test]
    fn test_job_status_is_tdd_phase() {
        assert!(JobStatus::PendingTest.is_tdd_phase());
        assert!(JobStatus::PendingTestRun.is_tdd_phase());
        assert!(!JobStatus::Created.is_tdd_phase());
        assert!(!JobStatus::PendingWork.is_tdd_phase());
        assert!(!JobStatus::PendingVerification.is_tdd_phase());
        assert!(!JobStatus::Pass.is_tdd_phase());
        assert!(!JobStatus::Fail.is_tdd_phase());
    }

    #[test]
    fn test_job_status_next_status_tdd() {
        assert_eq!(JobStatus::Created.next_status(true), Some(JobStatus::PendingTest));
        assert_eq!(JobStatus::PendingTest.next_status(true), Some(JobStatus::PendingWork));
        assert_eq!(JobStatus::PendingWork.next_status(true), Some(JobStatus::PendingVerification));
        assert_eq!(JobStatus::PendingVerification.next_status(true), Some(JobStatus::PendingTestRun));
        assert_eq!(JobStatus::PendingTestRun.next_status(true), None);
        assert_eq!(JobStatus::Pass.next_status(true), None);
        assert_eq!(JobStatus::Fail.next_status(true), None);
    }

    #[test]
    fn test_job_status_next_status_standard() {
        assert_eq!(JobStatus::Created.next_status(false), Some(JobStatus::PendingWork));
        assert_eq!(JobStatus::PendingWork.next_status(false), Some(JobStatus::PendingVerification));
        assert_eq!(JobStatus::PendingVerification.next_status(false), None);
        assert_eq!(JobStatus::Pass.next_status(false), None);
        assert_eq!(JobStatus::Fail.next_status(false), None);
    }

    #[test]
    fn test_job_status_entry_new() {
        let entry = JobStatusEntry::new("test_job".to_string());
        assert_eq!(entry.id, "test_job");
        assert_eq!(entry.status, JobStatus::Created);
        assert!(entry.error.is_none());
    }

    #[test]
    fn test_job_status_entry_update() {
        let mut entry = JobStatusEntry::new("test_job".to_string());
        entry.update_status(JobStatus::PendingWork);
        assert_eq!(entry.status, JobStatus::PendingWork);
        assert!(entry.error.is_none());
    }

    #[test]
    fn test_job_status_entry_set_failed() {
        let mut entry = JobStatusEntry::new("test_job".to_string());
        entry.set_failed("Test error".to_string());
        assert_eq!(entry.status, JobStatus::Fail);
        assert_eq!(entry.error, Some("Test error".to_string()));
    }

    #[test]
    fn test_job_status_serialization() {
        let entry = JobStatusEntry::new("test_job".to_string());
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"status\":\"created\""));
        
        let parsed: JobStatusEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, "test_job");
        assert_eq!(parsed.status, JobStatus::Created);
    }

    #[test]
    fn test_job_status_serialization_new_variants() {
        // Test PendingTest serialization
        let entry = JobStatusEntry::new("test_job".to_string());
        let mut entry_with_pending_test = entry.clone();
        entry_with_pending_test.update_status(JobStatus::PendingTest);
        let json = serde_json::to_string(&entry_with_pending_test).unwrap();
        assert!(json.contains("\"status\":\"pending_test\""));
        
        // Test PendingTestRun serialization
        let mut entry_with_pending_test_run = entry.clone();
        entry_with_pending_test_run.update_status(JobStatus::PendingTestRun);
        let json = serde_json::to_string(&entry_with_pending_test_run).unwrap();
        assert!(json.contains("\"status\":\"pending_test_run\""));
    }
}