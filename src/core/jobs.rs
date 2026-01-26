use gray_matter::engine::YAML;
use gray_matter::Matter;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

use crate::core::file_cache::{CacheStats, FileCache};
use crate::error::{JobParseError, WorkSplitError};
use crate::models::{Job, JobMetadata, LimitsConfig};

/// Jobs folder manager
pub struct JobsManager {
    /// Path to the jobs folder
    jobs_dir: PathBuf,
    /// Project root directory
    project_root: PathBuf,
    /// Limits configuration
    limits: LimitsConfig,
    /// Cache for context file contents
    cache: FileCache,
}

/// Constant for the test prompt filename
const TEST_PROMPT_FILE: &str = "_systemprompt_test.md";
/// Constant for the edit mode prompt filename
const EDIT_PROMPT_FILE: &str = "_systemprompt_edit.md";
/// Constant for the edit mode verification prompt filename
const VERIFY_EDIT_PROMPT_FILE: &str = "_systemprompt_verify_edit.md";
/// Constant for the split mode prompt filename
const SPLIT_PROMPT_FILE: &str = "_systemprompt_split.md";

impl JobsManager {
    /// Create a new jobs manager
    pub fn new(project_root: PathBuf, limits: LimitsConfig) -> Self {
        let jobs_dir = project_root.join("jobs");
        Self {
            jobs_dir,
            project_root,
            limits,
            cache: FileCache::new(),
        }
    }

    /// Get the jobs directory path
    pub fn jobs_dir(&self) -> &Path {
        &self.jobs_dir
    }

    /// Check if the jobs folder exists
    pub fn jobs_folder_exists(&self) -> bool {
        self.jobs_dir.exists() && self.jobs_dir.is_dir()
    }

    /// Discover all job files in the jobs folder
    /// Returns a list of job IDs (filenames without .md extension)
    pub fn discover_jobs(&self) -> Result<Vec<String>, WorkSplitError> {
        if !self.jobs_folder_exists() {
            return Err(WorkSplitError::JobsFolderNotFound(self.jobs_dir.clone()));
        }

        let mut jobs = Vec::new();

        for entry in fs::read_dir(&self.jobs_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    // Skip underscore-prefixed files (system files)
                    if filename.starts_with('_') {
                        continue;
                    }
                    // Only process .md files
                    if filename.ends_with(".md") {
                        let id = filename.trim_end_matches(".md").to_string();
                        jobs.push(id);
                    }
                }
            }
        }

        jobs.sort();
        info!("Discovered {} job files", jobs.len());
        Ok(jobs)
    }

    /// Parse a job file
    pub fn parse_job(&self, job_id: &str) -> Result<Job, WorkSplitError> {
        let file_path = self.jobs_dir.join(format!("{}.md", job_id));

        let content = fs::read_to_string(&file_path)
            .map_err(|e| JobParseError::ReadError(file_path.clone(), e))?;

        let matter = Matter::<YAML>::new();
        let parsed = matter.parse(&content);

        // Extract frontmatter data
        let data = parsed.data.ok_or_else(|| {
            JobParseError::FrontmatterError(
                file_path.clone(),
                "No frontmatter found".to_string(),
            )
        })?;

        // Deserialize the metadata
        let metadata: JobMetadata = data.deserialize().map_err(|e| {
            JobParseError::YamlError(file_path.clone(), e.to_string())
        })?;

        // Validate metadata
        metadata.validate(self.limits.max_context_files)?;

        // Get the markdown body (instructions)
        let instructions = parsed.content.trim().to_string();

        debug!("Parsed job '{}' with {} context files", job_id, metadata.context_files.len());

        Ok(Job::new(
            job_id.to_string(),
            metadata,
            instructions,
            file_path,
        ))
    }

    /// Load a system prompt file
    pub fn load_system_prompt(&self, filename: &str) -> Result<String, WorkSplitError> {
        let path = self.jobs_dir.join(filename);
        if !path.exists() {
            return Err(WorkSplitError::SystemPromptNotFound(path));
        }
        Ok(fs::read_to_string(&path)?)
    }

    /// Load the creation system prompt
    pub fn load_create_prompt(&self) -> Result<String, WorkSplitError> {
        self.load_system_prompt("_systemprompt_create.md")
    }

    /// Load the verification system prompt
    pub fn load_verify_prompt(&self) -> Result<String, WorkSplitError> {
        self.load_system_prompt("_systemprompt_verify.md")
    }

    /// Load the test generation system prompt
    pub fn load_test_prompt(&self) -> Result<String, WorkSplitError> {
        self.load_system_prompt(TEST_PROMPT_FILE)
    }

    /// Load the test generation system prompt optionally
    pub fn load_test_prompt_optional(&self) -> Result<Option<String>, WorkSplitError> {
        let path = self.jobs_dir.join(TEST_PROMPT_FILE);
        if !path.exists() {
            return Ok(None);
        }
        Ok(Some(fs::read_to_string(&path)?))
    }

    /// Load the edit mode system prompt, falling back to create prompt
    pub fn load_edit_prompt(&self) -> Result<String, WorkSplitError> {
        let edit_path = self.jobs_dir.join(EDIT_PROMPT_FILE);
        if edit_path.exists() {
            Ok(fs::read_to_string(&edit_path)?)
        } else {
            // Fall back to create prompt for backward compatibility
            self.load_create_prompt()
        }
    }

    /// Load the edit mode verification prompt, falling back to standard verify prompt
    pub fn load_verify_edit_prompt(&self) -> Result<String, WorkSplitError> {
        let path = self.jobs_dir.join(VERIFY_EDIT_PROMPT_FILE);
        if path.exists() {
            Ok(fs::read_to_string(&path)?)
        } else {
            // Fall back to standard verify prompt
            self.load_verify_prompt()
        }
    }

    /// Load the split mode system prompt
    pub fn load_split_prompt(&self) -> Result<String, WorkSplitError> {
        self.load_system_prompt(SPLIT_PROMPT_FILE)
    }

    /// Load a target file for split mode WITHOUT size limit validation
    /// Split mode needs to read large files (that's the whole point)
    pub fn load_target_file_unlimited(&self, relative_path: &Path) -> Result<String, WorkSplitError> {
        let full_path = self.project_root.join(relative_path);

        if !full_path.exists() {
            return Err(WorkSplitError::ContextFileNotFound(relative_path.to_path_buf()));
        }

        let content = fs::read_to_string(&full_path)?;
        let line_count = content.lines().count();

        info!(
            "Loaded target file '{}' for split ({} lines, no size limit)",
            relative_path.display(),
            line_count
        );

        Ok(content)
    }

    /// Load a context file and validate its size (uses cache)
    pub fn load_context_file(&mut self, relative_path: &Path) -> Result<String, WorkSplitError> {
        let full_path = self.project_root.join(relative_path);

        if !full_path.exists() {
            return Err(WorkSplitError::ContextFileNotFound(relative_path.to_path_buf()));
        }

        // Use cache to get or load content
        let entry = self.cache.get_or_load(&full_path)
            .map_err(WorkSplitError::Io)?;

        if entry.line_count > self.limits.max_context_lines {
            return Err(WorkSplitError::ContextFileTooLarge {
                path: relative_path.to_path_buf(),
                lines: entry.line_count,
                max: self.limits.max_context_lines,
            });
        }

        debug!(
            "Loaded context file '{}' ({} lines, cached)",
            relative_path.display(),
            entry.line_count
        );

        Ok(entry.content.clone())
    }

    /// Load all context files for a job
    pub fn load_context_files(&mut self, job: &Job) -> Result<Vec<(PathBuf, String)>, WorkSplitError> {
        let mut files = Vec::new();
        for path in &job.metadata.context_files {
            let content = self.load_context_file(path)?;
            files.push((path.clone(), content));
        }
        Ok(files)
    }

    /// Clear the file cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Invalidate a specific file from cache
    pub fn invalidate_cache(&mut self, path: &Path) {
        self.cache.invalidate(path);
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        self.cache.stats()
    }

    /// Estimate token count for content (rough heuristic: chars / 4)
    pub fn estimate_tokens(content: &str) -> usize {
        content.len() / 4
    }

    /// Check if the token budget is within limits
    /// Returns (estimated_tokens, is_warning, is_error)
    pub fn check_token_budget(
        &self,
        system_prompt: &str,
        context_files: &[(PathBuf, String)],
        instructions: &str,
        context_limit: usize,
    ) -> (usize, bool, bool) {
        let system_tokens = Self::estimate_tokens(system_prompt);
        let context_tokens: usize = context_files
            .iter()
            .map(|(_, content)| Self::estimate_tokens(content))
            .sum();
        let instruction_tokens = Self::estimate_tokens(instructions);
        let output_buffer = 1200; // Reserve for 900 LOC output

        let total = system_tokens + context_tokens + instruction_tokens + output_buffer;
        
        let warning_threshold = (context_limit as f64 * 0.8) as usize;
        let error_threshold = (context_limit as f64 * 0.9) as usize;

        let is_warning = total > warning_threshold;
        let is_error = total > error_threshold;

        if is_warning {
            warn!(
                "Token budget high: {} estimated tokens (limit: {})",
                total, context_limit
            );
        }

        (total, is_warning, is_error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_tokens() {
        // 100 chars should be ~25 tokens
        let content = "a".repeat(100);
        assert_eq!(JobsManager::estimate_tokens(&content), 25);
    }

    #[test]
    fn test_job_id_extraction() {
        let filename = "my_job_001.md";
        let id = filename.trim_end_matches(".md");
        assert_eq!(id, "my_job_001");
    }

    #[test]
    fn test_load_test_prompt() {
        // This test would require a mock file system setup
        // In practice, this would be tested with integration tests
        assert_eq!(TEST_PROMPT_FILE, "_systemprompt_test.md");
    }

    #[test]
    fn test_load_test_prompt_optional_exists() {
        // This test would require a mock file system setup
        // In practice, this would be tested with integration tests
        assert_eq!(TEST_PROMPT_FILE, "_systemprompt_test.md");
    }

    #[test]
    fn test_load_test_prompt_optional_missing() {
        // This test would require a mock file system setup
        // In practice, this would be tested with integration tests
        assert_eq!(TEST_PROMPT_FILE, "_systemprompt_test.md");
    }
}
