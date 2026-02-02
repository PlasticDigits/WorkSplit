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
    /// Verification partially passed (some edits succeeded, some failed)
    Partial,
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
                | JobStatus::Partial
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

    /// Check if this status is a partial completion
    pub fn is_partial(&self) -> bool {
        matches!(self, JobStatus::Partial)
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

/// State for partially completed edit jobs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialEditState {
    /// Edits that were successfully applied
    pub successful_edits: Vec<SuccessfulEdit>,
    /// Edits that failed to apply
    pub failed_edits: Vec<FailedEdit>,
}

impl PartialEditState {
    /// Create a new empty partial edit state
    pub fn new() -> Self {
        Self {
            successful_edits: Vec::new(),
            failed_edits: Vec::new(),
        }
    }

    /// Add a successful edit record
    pub fn add_successful_edit(&mut self, file_path: impl Into<String>, find_preview: impl Into<String>) {
        self.successful_edits.push(SuccessfulEdit {
            file_path: file_path.into(),
            find_preview: find_preview.into(),
        });
    }

    /// Add a failed edit record
    pub fn add_failed_edit(&mut self, file_path: impl Into<String>, find_preview: impl Into<String>) {
        self.failed_edits.push(FailedEdit {
            file_path: file_path.into(),
            find_preview: find_preview.into(),
            reason: String::new(),
            suggested_line: None,
        });
    }

    /// Check if there were any failures
    pub fn has_failures(&self) -> bool {
        !self.failed_edits.is_empty()
    }
}

impl Default for PartialEditState {
    fn default() -> Self {
        Self::new()
    }
}

/// Successful edit record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessfulEdit {
    pub file_path: String,
    pub find_preview: String,
}

/// Failed edit record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedEdit {
    pub file_path: String,
    pub find_preview: String,
    pub reason: String,
    pub suggested_line: Option<usize>,
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
    /// State for partially completed edit jobs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partial_state: Option<PartialEditState>,
    /// Whether this job has been run (regardless of pass/fail outcome)
    /// Jobs with ran=true are skipped by default on subsequent runs
    #[serde(default)]
    pub ran: bool,
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
            partial_state: None,
            ran: false,
        }
    }

    /// Mark this job as having been run
    pub fn mark_ran(&mut self) {
        self.ran = true;
        self.updated_at = Utc::now();
    }

    /// Reset the ran flag (for re-running jobs)
    pub fn clear_ran(&mut self) {
        self.ran = false;
        self.updated_at = Utc::now();
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

    /// Set status to Partial with partial edit state
    pub fn set_partial(&mut self, state: PartialEditState) {
        self.status = JobStatus::Partial;
        self.updated_at = Utc::now();
        self.partial_state = Some(state);
    }

    /// Get partial state if any
    pub fn get_partial_state(&self) -> Option<&PartialEditState> {
        self.partial_state.as_ref()
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
        assert!(!JobStatus::Partial.is_complete());
    }

    #[test]
    fn test_job_status_is_stuck() {
        assert!(JobStatus::PendingTest.is_stuck());
        assert!(JobStatus::PendingWork.is_stuck());
        assert!(JobStatus::PendingVerification.is_stuck());
        assert!(JobStatus::PendingTestRun.is_stuck());
        assert!(JobStatus::Partial.is_stuck());
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
        assert!(!JobStatus::Partial.is_tdd_phase());
    }

    #[test]
    fn test_job_status_is_partial() {
        assert!(JobStatus::Partial.is_partial());
        assert!(!JobStatus::Created.is_partial());
        assert!(!JobStatus::Pass.is_partial());
        assert!(!JobStatus::Fail.is_partial());
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
        assert!(entry.partial_state.is_none());
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
    fn test_job_status_entry_set_partial() {
        let mut entry = JobStatusEntry::new("test_job".to_string());
        let state = PartialEditState {
            successful_edits: vec![SuccessfulEdit {
                file_path: "src/main.rs".to_string(),
                find_preview: "fn main()".to_string(),
            }],
            failed_edits: vec![],
        };
        entry.set_partial(state);
        assert_eq!(entry.status, JobStatus::Partial);
        assert!(entry.partial_state.is_some());
    }

    #[test]
    fn test_job_status_entry_get_partial_state() {
        let mut entry = JobStatusEntry::new("test_job".to_string());
        assert!(entry.get_partial_state().is_none());

        let state = PartialEditState {
            successful_edits: vec![],
            failed_edits: vec![],
        };
        entry.set_partial(state);
        assert!(entry.get_partial_state().is_some());
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
        
        // Test Partial serialization
        let mut entry_with_partial = entry.clone();
        entry_with_partial.set_partial(PartialEditState {
            successful_edits: vec![],
            failed_edits: vec![],
        });
        let json = serde_json::to_string(&entry_with_partial).unwrap();
        assert!(json.contains("\"status\":\"partial\""));
    }

    #[test]
    fn test_partial_edit_state_serialization() {
        let state = PartialEditState {
            successful_edits: vec![SuccessfulEdit {
                file_path: "src/main.rs".to_string(),
                find_preview: "fn main()".to_string(),
            }],
            failed_edits: vec![FailedEdit {
                file_path: "src/lib.rs".to_string(),
                find_preview: "pub fn".to_string(),
                reason: "Pattern not found".to_string(),
                suggested_line: Some(10),
            }],
        };
        let json = serde_json::to_string(&state).unwrap();
        assert!(json.contains("\"successful_edits\""));
        assert!(json.contains("\"failed_edits\""));
        
        let parsed: PartialEditState = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.successful_edits.len(), 1);
        assert_eq!(parsed.failed_edits.len(), 1);
    }

    #[test]
    fn test_successful_edit_serialization() {
        let edit = SuccessfulEdit {
            file_path: "src/main.rs".to_string(),
            find_preview: "fn main()".to_string(),
        };
        let json = serde_json::to_string(&edit).unwrap();
        assert!(json.contains("\"file_path\""));
        assert!(json.contains("\"find_preview\""));
        
        let parsed: SuccessfulEdit = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.file_path, "src/main.rs");
        assert_eq!(parsed.find_preview, "fn main()");
    }

    #[test]
    fn test_failed_edit_serialization() {
        let edit = FailedEdit {
            file_path: "src/lib.rs".to_string(),
            find_preview: "pub fn".to_string(),
            reason: "Pattern not found".to_string(),
            suggested_line: Some(10),
        };
        let json = serde_json::to_string(&edit).unwrap();
        assert!(json.contains("\"file_path\""));
        assert!(json.contains("\"find_preview\""));
        assert!(json.contains("\"reason\""));
        assert!(json.contains("\"suggested_line\""));
        
        let parsed: FailedEdit = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.file_path, "src/lib.rs");
        assert_eq!(parsed.find_preview, "pub fn");
        assert_eq!(parsed.reason, "Pattern not found");
        assert_eq!(parsed.suggested_line, Some(10));
    }
}