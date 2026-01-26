use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Job template type for new-job command
#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum JobTemplate {
    /// Standard file replacement (default)
    Replace,
    /// Surgical edits to existing files
    Edit,
    /// Split large file into modules
    Split,
    /// Sequential multi-file generation
    Sequential,
    /// Test-driven development workflow
    Tdd,
}

/// Output mode: "replace" (default) generates full files, "edit" applies surgical changes,
/// "split" breaks a large file into smaller modules
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum OutputMode {
    #[default]
    Replace,
    Edit,
    Split,
    /// Batch text replacements using AFTER/INSERT pattern
    ReplacePattern,
    /// Update struct literals in test fixtures
    UpdateFixtures,
}

/// Metadata parsed from job file YAML frontmatter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobMetadata {
    /// Context files to include (max 2, each < 1000 LOC)
    #[serde(default)]
    pub context_files: Vec<PathBuf>,
    /// Output directory relative to project root
    pub output_dir: PathBuf,
    /// Output filename (used when output_files is not specified)
    pub output_file: String,
    /// Optional test file for TDD workflow (generated before implementation)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub test_file: Option<String>,
    /// Optional list of output files for sequential multi-file mode
    /// When specified with sequential: true, each file gets its own LLM call
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_files: Option<Vec<PathBuf>>,
    /// Enable sequential mode: one Ollama call per file with context accumulation
    /// Previously modified files in this job become automatic context for subsequent files
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sequential: Option<bool>,
    /// Output mode: "replace" (default) generates full files, "edit" applies surgical changes
    #[serde(default)]
    pub mode: OutputMode,
    /// Target files for edit mode (files to apply edits to)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_files: Option<Vec<PathBuf>>,
    /// Target file for split mode (the large file to split into modules)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_file: Option<PathBuf>,
    /// Whether to run verification phase (defaults to true)
    /// Set to false for simple/trusted jobs to skip verification and save an Ollama call
    #[serde(default = "default_verify")]
    pub verify: bool,
    /// Struct name for update_fixtures mode
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub struct_name: Option<String>,
    /// New field to add for update_fixtures mode (e.g., "verify: true")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub new_field: Option<String>,
}

fn default_verify() -> bool {
    true
}

impl JobMetadata {
    /// Validate the metadata against configuration limits
    pub fn validate(&self, max_context_files: usize) -> Result<(), JobValidationError> {
        if self.context_files.len() > max_context_files {
            return Err(JobValidationError::TooManyContextFiles {
                count: self.context_files.len(),
                max: max_context_files,
            });
        }
        if self.output_file.is_empty() {
            return Err(JobValidationError::EmptyOutputFile);
        }
        if let Some(test_file) = &self.test_file {
            if test_file.is_empty() {
                return Err(JobValidationError::EmptyTestFile);
            }
        }
        // Validate sequential mode configuration
        if let Some(ref files) = self.output_files {
            if files.is_empty() {
                return Err(JobValidationError::EmptyOutputFiles);
            }
            for file in files {
                if file.as_os_str().is_empty() {
                    return Err(JobValidationError::EmptyOutputFilePath);
                }
            }
        }
        // Validate edit mode configuration
        if self.mode == OutputMode::Edit {
            if let Some(ref files) = self.target_files {
                if files.is_empty() {
                    return Err(JobValidationError::EmptyTargetFiles);
                }
                for file in files {
                    if file.as_os_str().is_empty() {
                        return Err(JobValidationError::EmptyTargetFilePath);
                    }
                }
            }
            // Edit mode is incompatible with sequential mode
            if self.is_sequential() {
                return Err(JobValidationError::EditModeWithSequential);
            }
        }
        // Validate split mode configuration
        if self.mode == OutputMode::Split {
            // Split mode requires target_file
            if self.target_file.is_none() {
                return Err(JobValidationError::SplitMissingTargetFile);
            }
            if let Some(ref target) = self.target_file {
                if target.as_os_str().is_empty() {
                    return Err(JobValidationError::SplitEmptyTargetFile);
                }
            }
            // Split mode requires output_files
            if self.output_files.is_none() {
                return Err(JobValidationError::SplitMissingOutputFiles);
            }
            // Split mode is incompatible with sequential mode
            if self.is_sequential() {
                return Err(JobValidationError::SplitModeWithSequential);
            }
        }
        // Validate replace_pattern mode configuration
        if self.mode == OutputMode::ReplacePattern {
            if self.target_files.is_none() {
                return Err(JobValidationError::ReplacePatternMissingTargetFiles);
            }
        }
        // Validate update_fixtures mode configuration
        if self.mode == OutputMode::UpdateFixtures {
            if self.target_files.is_none() {
                return Err(JobValidationError::UpdateFixturesMissingTargetFiles);
            }
            if self.struct_name.is_none() {
                return Err(JobValidationError::UpdateFixturesMissingStructName);
            }
            if self.new_field.is_none() {
                return Err(JobValidationError::UpdateFixturesMissingNewField);
            }
        }
        Ok(())
    }

    /// Get the full output path
    pub fn output_path(&self) -> PathBuf {
        self.output_dir.join(&self.output_file)
    }

    /// Check if TDD workflow is enabled
    pub fn is_tdd_enabled(&self) -> bool {
        self.test_file.is_some()
    }

    /// Get the full test file path if TDD is enabled
    pub fn test_path(&self) -> Option<PathBuf> {
        self.test_file
            .as_ref()
            .map(|test_file| self.output_dir.join(test_file))
    }

    /// Check if sequential multi-file mode is enabled
    pub fn is_sequential(&self) -> bool {
        self.sequential.unwrap_or(false) && self.output_files.is_some()
    }

    /// Get the list of output files for sequential mode
    /// Returns the explicit output_files list, or a single-item list with the default output_path
    pub fn get_output_files(&self) -> Vec<PathBuf> {
        if let Some(ref files) = self.output_files {
            files.clone()
        } else {
            vec![self.output_path()]
        }
    }

    /// Check if this job uses edit mode
    pub fn is_edit_mode(&self) -> bool {
        self.mode == OutputMode::Edit
    }

    /// Check if this job uses split mode
    pub fn is_split_mode(&self) -> bool {
        self.mode == OutputMode::Split
    }

    /// Get target files for edit mode
    /// Returns target_files if set, otherwise returns output_path as single-item vec
    pub fn get_target_files(&self) -> Vec<PathBuf> {
        if let Some(ref files) = self.target_files {
            files.clone()
        } else {
            vec![self.output_path()]
        }
    }

    /// Check if verification should be run for this job
    pub fn should_verify(&self) -> bool {
        self.verify
    }

    /// Check if this job uses replace_pattern mode
    pub fn is_replace_pattern_mode(&self) -> bool {
        self.mode == OutputMode::ReplacePattern
    }

    /// Check if this job uses update_fixtures mode
    pub fn is_update_fixtures_mode(&self) -> bool {
        self.mode == OutputMode::UpdateFixtures
    }

    /// Get struct_name for update_fixtures mode
    pub fn get_struct_name(&self) -> Option<&String> {
        self.struct_name.as_ref()
    }

    /// Get new_field for update_fixtures mode
    pub fn get_new_field(&self) -> Option<&String> {
        self.new_field.as_ref()
    }
}

/// A parsed job with metadata and instructions
#[derive(Debug, Clone)]
pub struct Job {
    /// Job identifier (filename without .md extension)
    pub id: String,
    /// Metadata from frontmatter
    pub metadata: JobMetadata,
    /// Instructions (markdown body after frontmatter)
    pub instructions: String,
    /// Path to the job file
    pub file_path: PathBuf,
}

impl Job {
    /// Create a new job from parsed components
    pub fn new(id: String, metadata: JobMetadata, instructions: String, file_path: PathBuf) -> Self {
        Self {
            id,
            metadata,
            instructions,
            file_path,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum JobValidationError {
    #[error("Too many context files: {count} (max: {max})")]
    TooManyContextFiles { count: usize, max: usize },
    #[error("Output file cannot be empty")]
    EmptyOutputFile,
    #[error("Context file not found: {0}")]
    ContextFileNotFound(PathBuf),
    #[error("Context file too large: {path} has {lines} lines (max: {max})")]
    ContextFileTooLarge { path: PathBuf, lines: usize, max: usize },
    #[error("Test file name cannot be empty")]
    EmptyTestFile,
    #[error("output_files list cannot be empty")]
    EmptyOutputFiles,
    #[error("output_files contains an empty path")]
    EmptyOutputFilePath,
    #[error("target_files list cannot be empty in edit mode")]
    EmptyTargetFiles,
    #[error("target_files contains an empty path")]
    EmptyTargetFilePath,
    #[error("edit mode cannot be combined with sequential mode")]
    EditModeWithSequential,
    #[error("split mode requires target_file")]
    SplitMissingTargetFile,
    #[error("split mode target_file cannot be empty")]
    SplitEmptyTargetFile,
    #[error("split mode requires output_files")]
    SplitMissingOutputFiles,
    #[error("split mode cannot be combined with sequential mode")]
    SplitModeWithSequential,
    #[error("replace_pattern mode requires target_files")]
    ReplacePatternMissingTargetFiles,
    #[error("update_fixtures mode requires target_files")]
    UpdateFixturesMissingTargetFiles,
    #[error("update_fixtures mode requires struct_name")]
    UpdateFixturesMissingStructName,
    #[error("update_fixtures mode requires new_field")]
    UpdateFixturesMissingNewField,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_metadata_validate() {
        let metadata = JobMetadata {
            context_files: vec![PathBuf::from("a.rs"), PathBuf::from("b.rs")],
            output_dir: PathBuf::from("src/"),
            output_file: "output.rs".to_string(),
            test_file: None,
            output_files: None,
            sequential: None,
            mode: OutputMode::Replace,
            target_files: None,
            target_file: None,
            verify: true,
            struct_name: None,
            new_field: None,
        };
        assert!(metadata.validate(2).is_ok());
        assert!(metadata.validate(1).is_err());
    }

    #[test]
    fn test_job_metadata_empty_output_file() {
        let metadata = JobMetadata {
            context_files: vec![],
            output_dir: PathBuf::from("src/"),
            output_file: "".to_string(),
            test_file: None,
            output_files: None,
            sequential: None,
            mode: OutputMode::Replace,
            target_files: None,
            target_file: None,
            verify: true,
            struct_name: None,
            new_field: None,
        };
        assert!(matches!(
            metadata.validate(2),
            Err(JobValidationError::EmptyOutputFile)
        ));
    }

    #[test]
    fn test_output_path() {
        let metadata = JobMetadata {
            context_files: vec![],
            output_dir: PathBuf::from("src/services"),
            output_file: "user_service.rs".to_string(),
            test_file: None,
            output_files: None,
            sequential: None,
            mode: OutputMode::Replace,
            target_files: None,
            target_file: None,
            verify: true,
            struct_name: None,
            new_field: None,
        };
        assert_eq!(
            metadata.output_path(),
            PathBuf::from("src/services/user_service.rs")
        );
    }

    #[test]
    fn test_job_metadata_tdd_enabled() {
        let metadata_with_test = JobMetadata {
            context_files: vec![],
            output_dir: PathBuf::from("src/"),
            output_file: "service.rs".to_string(),
            test_file: Some("service_test.rs".to_string()),
            output_files: None,
            sequential: None,
            mode: OutputMode::Replace,
            target_files: None,
            target_file: None,
            verify: true,
            struct_name: None,
            new_field: None,
        };
        assert!(metadata_with_test.is_tdd_enabled());

        let metadata_without_test = JobMetadata {
            context_files: vec![],
            output_dir: PathBuf::from("src/"),
            output_file: "service.rs".to_string(),
            test_file: None,
            output_files: None,
            sequential: None,
            mode: OutputMode::Replace,
            target_files: None,
            target_file: None,
            verify: true,
            struct_name: None,
            new_field: None,
        };
        assert!(!metadata_without_test.is_tdd_enabled());
    }

    #[test]
    fn test_job_metadata_test_path() {
        let metadata_with_test = JobMetadata {
            context_files: vec![],
            output_dir: PathBuf::from("src/services"),
            output_file: "user_service.rs".to_string(),
            test_file: Some("user_service_test.rs".to_string()),
            output_files: None,
            sequential: None,
            mode: OutputMode::Replace,
            target_files: None,
            target_file: None,
            verify: true,
            struct_name: None,
            new_field: None,
        };
        assert_eq!(
            metadata_with_test.test_path(),
            Some(PathBuf::from("src/services/user_service_test.rs"))
        );

        let metadata_without_test = JobMetadata {
            context_files: vec![],
            output_dir: PathBuf::from("src/services"),
            output_file: "user_service.rs".to_string(),
            test_file: None,
            output_files: None,
            sequential: None,
            mode: OutputMode::Replace,
            target_files: None,
            target_file: None,
            verify: true,
            struct_name: None,
            new_field: None,
        };
        assert_eq!(metadata_without_test.test_path(), None);
    }

    #[test]
    fn test_job_metadata_validate_empty_test_file() {
        let metadata = JobMetadata {
            context_files: vec![],
            output_dir: PathBuf::from("src/"),
            output_file: "service.rs".to_string(),
            test_file: Some("".to_string()),
            output_files: None,
            sequential: None,
            mode: OutputMode::Replace,
            target_files: None,
            target_file: None,
            verify: true,
            struct_name: None,
            new_field: None,
        };
        assert!(matches!(
            metadata.validate(2),
            Err(JobValidationError::EmptyTestFile)
        ));
    }

    #[test]
    fn test_job_metadata_default_no_test_file() {
        let metadata: JobMetadata = serde_yaml::from_str(
            r#"
context_files: []
output_dir: src/
output_file: service.rs
"#,
        )
        .unwrap();
        assert_eq!(metadata.test_file, None);
        assert!(!metadata.is_tdd_enabled());
        assert_eq!(metadata.test_path(), None);
    }

    #[test]
    fn test_job_metadata_sequential_mode_enabled() {
        let metadata: JobMetadata = serde_yaml::from_str(
            r#"
context_files: []
output_dir: src/
output_file: default.rs
output_files:
  - src/main.rs
  - src/lib.rs
  - src/models.rs
sequential: true
"#,
        )
        .unwrap();
        assert!(metadata.is_sequential());
        assert_eq!(metadata.output_files.as_ref().unwrap().len(), 3);
        let output_files = metadata.get_output_files();
        assert_eq!(output_files.len(), 3);
        assert_eq!(output_files[0], PathBuf::from("src/main.rs"));
        assert_eq!(output_files[1], PathBuf::from("src/lib.rs"));
        assert_eq!(output_files[2], PathBuf::from("src/models.rs"));
    }

    #[test]
    fn test_job_metadata_sequential_mode_disabled_without_flag() {
        let metadata: JobMetadata = serde_yaml::from_str(
            r#"
context_files: []
output_dir: src/
output_file: default.rs
output_files:
  - src/main.rs
  - src/lib.rs
"#,
        )
        .unwrap();
        // output_files present but sequential not set
        assert!(!metadata.is_sequential());
        // get_output_files still returns the list
        let output_files = metadata.get_output_files();
        assert_eq!(output_files.len(), 2);
    }

    #[test]
    fn test_job_metadata_sequential_mode_without_output_files() {
        let metadata: JobMetadata = serde_yaml::from_str(
            r#"
context_files: []
output_dir: src/
output_file: default.rs
sequential: true
"#,
        )
        .unwrap();
        // sequential: true but no output_files
        assert!(!metadata.is_sequential());
        // Falls back to single file
        let output_files = metadata.get_output_files();
        assert_eq!(output_files.len(), 1);
        assert_eq!(output_files[0], PathBuf::from("src/default.rs"));
    }

    #[test]
    fn test_job_metadata_get_output_files_fallback() {
        let metadata = JobMetadata {
            context_files: vec![],
            output_dir: PathBuf::from("src/services"),
            output_file: "user_service.rs".to_string(),
            test_file: None,
            output_files: None,
            sequential: None,
            mode: OutputMode::Replace,
            target_files: None,
            target_file: None,
            verify: true,
            struct_name: None,
            new_field: None,
        };
        let output_files = metadata.get_output_files();
        assert_eq!(output_files.len(), 1);
        assert_eq!(output_files[0], PathBuf::from("src/services/user_service.rs"));
    }

    #[test]
    fn test_job_metadata_validate_empty_output_files() {
        let metadata = JobMetadata {
            context_files: vec![],
            output_dir: PathBuf::from("src/"),
            output_file: "service.rs".to_string(),
            test_file: None,
            output_files: Some(vec![]),
            sequential: Some(true),
            mode: OutputMode::Replace,
            target_files: None,
            target_file: None,
            verify: true,
            struct_name: None,
            new_field: None,
        };
        assert!(matches!(
            metadata.validate(2),
            Err(JobValidationError::EmptyOutputFiles)
        ));
    }

    #[test]
    fn test_job_metadata_validate_empty_path_in_output_files() {
        let metadata = JobMetadata {
            context_files: vec![],
            output_dir: PathBuf::from("src/"),
            output_file: "service.rs".to_string(),
            test_file: None,
            output_files: Some(vec![PathBuf::from("src/main.rs"), PathBuf::from("")]),
            sequential: Some(true),
            mode: OutputMode::Replace,
            target_files: None,
            target_file: None,
            verify: true,
            struct_name: None,
            new_field: None,
        };
        assert!(matches!(
            metadata.validate(2),
            Err(JobValidationError::EmptyOutputFilePath)
        ));
    }

    #[test]
    fn test_job_metadata_is_edit_mode() {
        let metadata_replace = JobMetadata {
            context_files: vec![],
            output_dir: PathBuf::from("src/"),
            output_file: "service.rs".to_string(),
            test_file: None,
            output_files: None,
            sequential: None,
            mode: OutputMode::Replace,
            target_files: None,
            target_file: None,
            verify: true,
            struct_name: None,
            new_field: None,
        };
        assert!(!metadata_replace.is_edit_mode());

        let metadata_edit = JobMetadata {
            context_files: vec![],
            output_dir: PathBuf::from("src/"),
            output_file: "service.rs".to_string(),
            test_file: None,
            output_files: None,
            sequential: None,
            mode: OutputMode::Edit,
            target_files: None,
            target_file: None,
            verify: true,
            struct_name: None,
            new_field: None,
        };
        assert!(metadata_edit.is_edit_mode());
    }

    #[test]
    fn test_job_metadata_get_target_files() {
        let metadata_with_targets = JobMetadata {
            context_files: vec![],
            output_dir: PathBuf::from("src/"),
            output_file: "service.rs".to_string(),
            test_file: None,
            output_files: None,
            sequential: None,
            mode: OutputMode::Edit,
            target_files: Some(vec![
                PathBuf::from("src/main.rs"),
                PathBuf::from("src/lib.rs"),
            ]),
            target_file: None,
            verify: true,
            struct_name: None,
            new_field: None,
        };
        let target_files = metadata_with_targets.get_target_files();
        assert_eq!(target_files.len(), 2);
        assert_eq!(target_files[0], PathBuf::from("src/main.rs"));
        assert_eq!(target_files[1], PathBuf::from("src/lib.rs"));

        let metadata_without_targets = JobMetadata {
            context_files: vec![],
            output_dir: PathBuf::from("src/services"),
            output_file: "user_service.rs".to_string(),
            test_file: None,
            output_files: None,
            sequential: None,
            mode: OutputMode::Edit,
            target_files: None,
            target_file: None,
            verify: true,
            struct_name: None,
            new_field: None,
        };
        let target_files = metadata_without_targets.get_target_files();
        assert_eq!(target_files.len(), 1);
        assert_eq!(target_files[0], PathBuf::from("src/services/user_service.rs"));
    }

    #[test]
    fn test_job_metadata_validate_empty_target_files() {
        let metadata = JobMetadata {
            context_files: vec![],
            output_dir: PathBuf::from("src/"),
            output_file: "service.rs".to_string(),
            test_file: None,
            output_files: None,
            sequential: None,
            mode: OutputMode::Edit,
            target_files: Some(vec![]),
            target_file: None,
            verify: true,
            struct_name: None,
            new_field: None,
        };
        assert!(matches!(
            metadata.validate(2),
            Err(JobValidationError::EmptyTargetFiles)
        ));
    }

    #[test]
    fn test_job_metadata_validate_empty_path_in_target_files() {
        let metadata = JobMetadata {
            context_files: vec![],
            output_dir: PathBuf::from("src/"),
            output_file: "service.rs".to_string(),
            test_file: None,
            output_files: None,
            sequential: None,
            mode: OutputMode::Edit,
            target_files: Some(vec![PathBuf::from("src/main.rs"), PathBuf::from("")]),
            target_file: None,
            verify: true,
            struct_name: None,
            new_field: None,
        };
        assert!(matches!(
            metadata.validate(2),
            Err(JobValidationError::EmptyTargetFilePath)
        ));
    }

    #[test]
    fn test_job_metadata_validate_edit_mode_with_sequential() {
        let metadata = JobMetadata {
            context_files: vec![],
            output_dir: PathBuf::from("src/"),
            output_file: "service.rs".to_string(),
            test_file: None,
            output_files: Some(vec![PathBuf::from("src/main.rs")]),
            sequential: Some(true),
            mode: OutputMode::Edit,
            target_files: Some(vec![PathBuf::from("src/main.rs")]),
            target_file: None,
            verify: true,
            struct_name: None,
            new_field: None,
        };
        assert!(matches!(
            metadata.validate(2),
            Err(JobValidationError::EditModeWithSequential)
        ));
    }

    #[test]
    fn test_job_metadata_edit_mode_serialization() {
        let metadata: JobMetadata = serde_yaml::from_str(
            r#"
mode: edit
target_files:
  - src/main.rs
  - src/lib.rs
output_dir: src/
output_file: main.rs
"#,
        )
        .unwrap();
        assert_eq!(metadata.mode, OutputMode::Edit);
        assert_eq!(metadata.target_files.as_ref().unwrap().len(), 2);
        assert_eq!(metadata.target_files.as_ref().unwrap()[0], PathBuf::from("src/main.rs"));
        assert_eq!(metadata.target_files.as_ref().unwrap()[1], PathBuf::from("src/lib.rs"));
    }

    #[test]
    fn test_job_metadata_default_mode_is_replace() {
        let metadata: JobMetadata = serde_yaml::from_str(
            r#"
output_dir: src/
output_file: service.rs
"#,
        )
        .unwrap();
        assert_eq!(metadata.mode, OutputMode::Replace);
    }

    #[test]
    fn test_job_metadata_replace_mode_serialization() {
        let metadata: JobMetadata = serde_yaml::from_str(
            r#"
mode: replace
output_dir: src/
output_file: service.rs
"#,
        )
        .unwrap();
        assert_eq!(metadata.mode, OutputMode::Replace);
    }

    #[test]
    fn test_job_metadata_is_split_mode() {
        let metadata_split = JobMetadata {
            context_files: vec![],
            output_dir: PathBuf::from("src/"),
            output_file: "runner.rs".to_string(),
            test_file: None,
            output_files: Some(vec![PathBuf::from("src/runner.rs"), PathBuf::from("src/runner_edit.rs")]),
            sequential: None,
            mode: OutputMode::Split,
            target_files: None,
            target_file: Some(PathBuf::from("src/runner.rs")),
            verify: true,
            struct_name: None,
            new_field: None,
        };
        assert!(metadata_split.is_split_mode());
        assert!(!metadata_split.is_edit_mode());
    }

    #[test]
    fn test_job_metadata_split_mode_validation() {
        // Valid split mode
        let valid_metadata = JobMetadata {
            context_files: vec![],
            output_dir: PathBuf::from("src/"),
            output_file: "runner.rs".to_string(),
            test_file: None,
            output_files: Some(vec![PathBuf::from("src/runner.rs"), PathBuf::from("src/runner_edit.rs")]),
            sequential: None,
            mode: OutputMode::Split,
            target_files: None,
            target_file: Some(PathBuf::from("src/core/runner.rs")),
            verify: true,
            struct_name: None,
            new_field: None,
        };
        assert!(valid_metadata.validate(2).is_ok());
    }

    #[test]
    fn test_job_metadata_split_mode_missing_target_file() {
        let metadata = JobMetadata {
            context_files: vec![],
            output_dir: PathBuf::from("src/"),
            output_file: "runner.rs".to_string(),
            test_file: None,
            output_files: Some(vec![PathBuf::from("src/runner.rs")]),
            sequential: None,
            mode: OutputMode::Split,
            target_files: None,
            target_file: None, // Missing!
            verify: true,
            struct_name: None,
            new_field: None,
        };
        assert!(matches!(
            metadata.validate(2),
            Err(JobValidationError::SplitMissingTargetFile)
        ));
    }

    #[test]
    fn test_job_metadata_split_mode_missing_output_files() {
        let metadata = JobMetadata {
            context_files: vec![],
            output_dir: PathBuf::from("src/"),
            output_file: "runner.rs".to_string(),
            test_file: None,
            output_files: None, // Missing!
            sequential: None,
            mode: OutputMode::Split,
            target_files: None,
            target_file: Some(PathBuf::from("src/core/runner.rs")),
            verify: true,
            struct_name: None,
            new_field: None,
        };
        assert!(matches!(
            metadata.validate(2),
            Err(JobValidationError::SplitMissingOutputFiles)
        ));
    }

    #[test]
    fn test_job_metadata_split_mode_with_sequential() {
        let metadata = JobMetadata {
            context_files: vec![],
            output_dir: PathBuf::from("src/"),
            output_file: "runner.rs".to_string(),
            test_file: None,
            output_files: Some(vec![PathBuf::from("src/runner.rs")]),
            sequential: Some(true), // Incompatible!
            mode: OutputMode::Split,
            target_files: None,
            target_file: Some(PathBuf::from("src/core/runner.rs")),
            verify: true,
            struct_name: None,
            new_field: None,
        };
        assert!(matches!(
            metadata.validate(2),
            Err(JobValidationError::SplitModeWithSequential)
        ));
    }

    #[test]
    fn test_job_metadata_split_mode_serialization() {
        let metadata: JobMetadata = serde_yaml::from_str(
            r#"
mode: split
target_file: src/core/runner.rs
output_files:
  - src/core/runner.rs
  - src/core/runner_edit.rs
  - src/core/runner_verify.rs
output_dir: src/core/
output_file: runner.rs
"#,
        )
        .unwrap();
        assert_eq!(metadata.mode, OutputMode::Split);
        assert_eq!(metadata.target_file, Some(PathBuf::from("src/core/runner.rs")));
        assert_eq!(metadata.output_files.as_ref().unwrap().len(), 3);
    }
}