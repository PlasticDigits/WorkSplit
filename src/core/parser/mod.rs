//! Parser module for extracting code from LLM responses and assembling prompts.

mod edit;
mod extract;
mod prompts;

pub use edit::*;
pub use extract::*;
pub use prompts::*;

use std::path::PathBuf;
use crate::models::JobStatus;

/// An extracted file from LLM response
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedFile {
    /// File path (None if using default output path from job metadata)
    pub path: Option<PathBuf>,
    /// File content
    pub content: String,
}

impl ExtractedFile {
    /// Create a new extracted file with a specific path
    pub fn with_path(path: PathBuf, content: String) -> Self {
        Self {
            path: Some(path),
            content,
        }
    }

    /// Create a new extracted file using default path
    pub fn default_path(content: String) -> Self {
        Self {
            path: None,
            content,
        }
    }
}

/// Verification result with severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationResult {
    Pass,
    PassWithWarnings,
    FailSoft,
    FailHard,
}

impl VerificationResult {
    /// Returns true if the job should be marked as passed
    pub fn is_pass(&self) -> bool {
        matches!(self, VerificationResult::Pass | VerificationResult::PassWithWarnings)
    }
    
    /// Returns true if the job failed critically
    pub fn is_hard_fail(&self) -> bool {
        matches!(self, VerificationResult::FailHard)
    }

    /// Convert verification result to job status
    pub fn to_job_status(&self) -> JobStatus {
        match self {
            VerificationResult::Pass | VerificationResult::PassWithWarnings => JobStatus::Pass,
            VerificationResult::FailSoft | VerificationResult::FailHard => JobStatus::Fail,
        }
    }
}

/// Instruction for replace_pattern mode (AFTER/INSERT)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplacePatternInstruction {
    /// Text to find (AFTER pattern)
    pub after_pattern: String,
    /// Text to insert after the pattern
    pub insert_text: String,
    /// Optional scope restriction (e.g., "#[cfg(test)]")
    pub scope: Option<String>,
}

/// Parsed replace pattern instructions
#[derive(Debug, Clone)]
pub struct ParsedReplacePatterns {
    pub instructions: Vec<ReplacePatternInstruction>,
    pub scope: Option<String>,
}

/// Struct literal match for update_fixtures mode
#[derive(Debug, Clone)]
pub struct StructLiteralMatch {
    /// Start position of the struct literal
    pub start: usize,
    /// End position (after closing brace)
    pub end: usize,
    /// The last field before the closing brace
    pub last_field_end: usize,
    /// Line number (1-indexed)
    pub line_number: usize,
}
