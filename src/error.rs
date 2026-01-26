use std::path::PathBuf;
use thiserror::Error;

use crate::models::{ConfigError, JobValidationError};

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

pub type Result<T> = std::result::Result<T, WorkSplitError>;