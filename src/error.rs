use std::path::PathBuf;
use thiserror::Error;

use crate::models::{ConfigError, JobValidationError};

use serde::Serialize;

/// Actionable suggestion for fixing an error
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EditSuggestion {
    /// The suggestion text
    pub message: String,
    /// Priority (1 = most likely to help)
    pub priority: u8,
    /// Category of suggestion
    pub category: SuggestionCategory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SuggestionCategory {
    Whitespace,
    CaseSensitivity,
    ContextSize,
    ModeChoice,
    LineReference,
}

/// Main error type for WorkSplit
#[derive(Error, Debug)]
pub enum WorkSplitError {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("Job validation error: {0}")]
    JobValidation(#[from] JobValidationError),

    #[error("Job parsing error: {0}")]
    JobParsing(#[from] JobParseError),

    #[error("Status file error: {0}")]
    Status(#[from] StatusError),

    #[error("Ollama error: {0}")]
    Ollama(#[from] OllamaError),

    #[error("Build failed for {command}:\n{output}")]
    BuildFailed {
        command: String,
        output: String,
    },

    #[error("File too large: {path} has {lines} lines (max: {limit})\n\nManager action required:\n{suggestion}")]
    FileTooLarge {
        path: PathBuf,
        lines: usize,
        limit: usize,
        suggestion: String,
    },

    #[error("Cyclic dependency detected in job files. Check depends_on for cycles.")]
    CyclicDependency,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Jobs folder not found at {0}")]
    JobsFolderNotFound(PathBuf),

    #[error("System prompt not found: {0}")]
    SystemPromptNotFound(PathBuf),

    #[error("Context file not found: {0}")]
    ContextFileNotFound(PathBuf),

    #[error("Context file too large: {path} has {lines} lines (max: {max})")]
    ContextFileTooLarge {
        path: PathBuf,
        lines: usize,
        max: usize,
    },

    #[error("Output exceeded line limit: {lines} lines (max: {max})")]
    OutputTooLarge { lines: usize, max: usize },

    #[error("Token budget exceeded: estimated {estimated} tokens (max: {max})")]
    TokenBudgetExceeded { estimated: usize, max: usize },

    #[error("Edit failed: {message}")]
    EditFailedWithSuggestions {
        message: String,
        suggestions: Vec<EditSuggestion>,
        fuzzy_matches: Vec<(String, usize, u8)>, // (file, line, similarity%)
    },

    #[error("Edit failed: {0}")]
    EditFailed(String),

    #[error("Cannot write to protected path: {0}")]
    ProtectedPathViolation(PathBuf),

    #[error("Invalid job name: {0}")]
    InvalidJobName(String),

    #[error("Job already exists: {0}")]
    JobAlreadyExists(String),
}

/// Errors related to job file parsing
#[derive(Error, Debug)]
pub enum JobParseError {
    #[error("Failed to read job file {0}: {1}")]
    ReadError(PathBuf, std::io::Error),

    #[error("Failed to parse frontmatter in {0}: {1}")]
    FrontmatterError(PathBuf, String),

    #[error("Missing required field in {0}: {1}")]
    MissingField(PathBuf, String),

    #[error("Invalid YAML in {0}: {1}")]
    YamlError(PathBuf, String),
}

/// Errors related to status file operations
#[derive(Error, Debug)]
pub enum StatusError {
    #[error("Failed to read status file {0}: {1}")]
    ReadError(PathBuf, std::io::Error),

    #[error("Failed to write status file {0}: {1}")]
    WriteError(PathBuf, std::io::Error),

    #[error("Failed to parse status file {0}: {1}")]
    ParseError(PathBuf, String),

    #[error("Job not found in status file: {0}")]
    JobNotFound(String),
}

/// Errors related to Ollama API
#[derive(Error, Debug)]
pub enum OllamaError {
    #[error("Connection refused: {0}")]
    ConnectionRefused(String),

    #[error("Request timeout after {0} seconds")]
    Timeout(u64),

    #[error("HTTP error: {status} - {message}")]
    HttpError { status: u16, message: String },

    #[error("Failed to parse response: {0}")]
    ParseError(String),

    #[error("Request failed: {0}")]
    RequestFailed(String),

    #[error("Stream error: {0}")]
    StreamError(String),

    #[error("SYSTEM PROMPT ERROR: Model stuck in thinking loop for {duration_secs}s ({thinking_tokens} thinking tokens, 0 output). Adjust system prompt to prevent over-analysis.")]
    ThinkingTimeout {
        duration_secs: u64,
        thinking_tokens: usize,
    },
}

impl From<reqwest::Error> for OllamaError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            OllamaError::Timeout(0)
        } else if err.is_connect() {
            OllamaError::ConnectionRefused(err.to_string())
        } else if let Some(status) = err.status() {
            OllamaError::HttpError {
                status: status.as_u16(),
                message: err.to_string(),
            }
        } else {
            OllamaError::RequestFailed(err.to_string())
        }
    }
}

impl WorkSplitError {
    /// Generate suggestions from edit failure context
    pub fn edit_failed_with_context(
        message: String,
        file_path: &str,
        find_text: &str,
        edit_count: usize,
        has_fuzzy_match: bool,
        fuzzy_matches: Vec<(String, usize, u8)>,
    ) -> Self {
        let mut suggestions = Vec::new();

        // Check for whitespace hints
        if message.contains("whitespace") || has_fuzzy_match {
            suggestions.push(EditSuggestion {
                message: format!(
                    "Check whitespace: {} may use different indentation (spaces vs tabs, indent level)",
                    file_path
                ),
                priority: 1,
                category: SuggestionCategory::Whitespace,
            });
        }

        // Check for many edits
        if edit_count > 10 {
            suggestions.push(EditSuggestion {
                message: format!(
                    "Consider replace mode: this job has {} edits, replace mode is safer for 10+ edits",
                    edit_count
                ),
                priority: 2,
                category: SuggestionCategory::ModeChoice,
            });
        }

        // Check for small FIND context
        let find_lines = find_text.lines().count();
        if find_lines < 3 {
            suggestions.push(EditSuggestion {
                message: "Include more context: FIND text is too short, add surrounding lines for uniqueness".to_string(),
                priority: 1,
                category: SuggestionCategory::ContextSize,
            });
        }

        // Add line references from fuzzy matches
        for (file, line, similarity) in &fuzzy_matches {
            if *similarity >= 70 {
                suggestions.push(EditSuggestion {
                    message: format!(
                        "Possible match at {}:{} ({}% similar)",
                        file, line, similarity
                    ),
                    priority: 3,
                    category: SuggestionCategory::LineReference,
                });
            }
        }

        // Sort by priority
        suggestions.sort_by_key(|s| s.priority);

        WorkSplitError::EditFailedWithSuggestions {
            message,
            suggestions,
            fuzzy_matches,
        }
    }

    /// Format error with suggestions for display
    pub fn display_with_suggestions(&self) -> String {
        match self {
            WorkSplitError::EditFailedWithSuggestions { message, suggestions, .. } => {
                let mut output = format!("Error: Edit failed: {}\n", message);
                if !suggestions.is_empty() {
                    output.push_str("\nSuggestions:\n");
                    for (i, s) in suggestions.iter().enumerate() {
                        output.push_str(&format!("  {}. {}\n", i + 1, s.message));
                    }
                }
                output
            }
            other => format!("Error: {}", other),
        }
    }
}

pub type Result<T> = std::result::Result<T, WorkSplitError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edit_suggestion_priority() {
        let suggestion = EditSuggestion {
            message: "Test suggestion".to_string(),
            priority: 1,
            category: SuggestionCategory::Whitespace,
        };
        assert_eq!(suggestion.priority, 1);
        assert_eq!(suggestion.category, SuggestionCategory::Whitespace);
    }

    #[test]
    fn test_edit_failed_with_context_whitespace() {
        let error = WorkSplitError::edit_failed_with_context(
            "FIND text not found".to_string(),
            "src/main.rs",
            "fn main()",
            5,
            true,
            vec![("src/main.rs".to_string(), 45, 85)],
        );

        match error {
            WorkSplitError::EditFailedWithSuggestions {
                message,
                suggestions,
                fuzzy_matches,
            } => {
                assert_eq!(message, "FIND text not found");
                assert!(!suggestions.is_empty());
                assert_eq!(fuzzy_matches.len(), 1);
                assert_eq!(fuzzy_matches[0].0, "src/main.rs");
                assert_eq!(fuzzy_matches[0].1, 45);
                assert_eq!(fuzzy_matches[0].2, 85);
            }
            _ => panic!("Expected EditFailedWithSuggestions"),
        }
    }

    #[test]
    fn test_edit_failed_with_context_many_edits() {
        let error = WorkSplitError::edit_failed_with_context(
            "FIND text not found".to_string(),
            "src/main.rs",
            "fn main()",
            15,
            false,
            vec![],
        );

        match error {
            WorkSplitError::EditFailedWithSuggestions {
                message,
                suggestions,
                fuzzy_matches,
            } => {
                assert_eq!(message, "FIND text not found");
                assert!(!suggestions.is_empty());
                assert_eq!(fuzzy_matches.len(), 0);
                // Should have suggestion about replace mode
                assert!(suggestions.iter().any(|s| s.message.contains("replace mode")));
            }
            _ => panic!("Expected EditFailedWithSuggestions"),
        }
    }

    #[test]
    fn test_display_with_suggestions() {
        let error = WorkSplitError::edit_failed_with_context(
            "FIND text not found".to_string(),
            "src/main.rs",
            "fn main()",
            5,
            true,
            vec![("src/main.rs".to_string(), 45, 85)],
        );

        let display = error.display_with_suggestions();
        assert!(display.contains("Error: Edit failed: FIND text not found"));
        assert!(display.contains("Suggestions:"));
        assert!(display.contains("Check whitespace"));
        assert!(display.contains("Possible match"));
    }

    #[test]
    fn test_suggestion_category_variants() {
        assert_eq!(SuggestionCategory::Whitespace, SuggestionCategory::Whitespace);
        assert_eq!(SuggestionCategory::CaseSensitivity, SuggestionCategory::CaseSensitivity);
        assert_eq!(SuggestionCategory::ContextSize, SuggestionCategory::ContextSize);
        assert_eq!(SuggestionCategory::ModeChoice, SuggestionCategory::ModeChoice);
        assert_eq!(SuggestionCategory::LineReference, SuggestionCategory::LineReference);
    }

    #[test]
    fn test_suggestion_serialization() {
        let suggestion = EditSuggestion {
            message: "Test suggestion".to_string(),
            priority: 2,
            category: SuggestionCategory::ModeChoice,
        };
        let json = serde_json::to_string(&suggestion).unwrap();
        assert!(json.contains("\"message\""));
        assert!(json.contains("\"priority\""));
        assert!(json.contains("\"category\""));
    }

    #[test]
    fn test_suggestion_category_serialization() {
        let category = SuggestionCategory::Whitespace;
        let json = serde_json::to_string(&category).unwrap();
        assert!(json.contains("\"whitespace\""));
    }
}