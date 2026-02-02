use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::core::{extract_code_files, load_config, JobsManager, OllamaClient, StatusManager};
use crate::error::WorkSplitError;
use crate::models::{Config, ErrorType, JobStatus, LimitsConfig};

/// Result of a fix attempt
#[derive(Debug)]
pub struct FixResult {
    /// Whether the fix was successful
    pub success: bool,
    /// Number of files written
    pub files_written: usize,
    /// Error message if fix failed
    pub error: Option<String>,
}

impl FixResult {
    pub fn success(files_written: usize) -> Self {
        Self {
            success: true,
            files_written,
            error: None,
        }
    }

    pub fn failure(error: String) -> Self {
        Self {
            success: false,
            files_written: 0,
            error: Some(error),
        }
    }

    pub fn no_errors() -> Self {
        Self {
            success: true,
            files_written: 0,
            error: None,
        }
    }
}

/// Summary of fix-all operation
#[derive(Debug)]
pub struct FixSummary {
    /// Number of jobs fixed successfully
    pub fixed: usize,
    /// Number of jobs that failed to fix
    pub failed: usize,
    /// Number of jobs skipped (no errors)
    pub skipped: usize,
}

/// Fix errors in a file using LLM - core function that can be called with any error type
pub async fn fix_with_error_context(
    project_root: &PathBuf,
    output_path: &PathBuf,
    error_output: &str,
    error_type: ErrorType,
    config: &Config,
) -> Result<FixResult, WorkSplitError> {
    let full_output_path = project_root.join(output_path);

    if !full_output_path.exists() {
        return Err(WorkSplitError::JobError(format!(
            "Output file does not exist: {}",
            full_output_path.display()
        )));
    }

    // Load fix system prompt (auto-recreates from template if missing)
    let jobs_manager = JobsManager::new(project_root.clone(), LimitsConfig::default());
    let system_prompt = jobs_manager.load_system_prompt("_systemprompt_fix.md")?;

    // Read the source file content
    let file_content = fs::read_to_string(&full_output_path)?;

    // Build user prompt with error type context
    let user_prompt = format!(
        "{}\n\n```\n{}\n```\n\n## Source File: {}\n\n```\n{}\n```\n\n{}\n\nOutput the complete fixed file using ~~~worksplit:{} delimiters.",
        error_type.prompt_header(),
        error_output.trim(),
        output_path.display(),
        file_content,
        error_type.fix_instructions(),
        output_path.display()
    );

    // Call Ollama
    let client = OllamaClient::new(config.ollama.clone())
        .map_err(WorkSplitError::Ollama)?;

    println!("Calling LLM to fix {} errors...", error_type.lowercase_name());
    let response = client
        .generate(Some(&system_prompt), &user_prompt, config.behavior.stream_output)
        .await
        .map_err(WorkSplitError::Ollama)?;

    // Parse output using replace mode (~~~worksplit delimiters)
    let extracted_files = extract_code_files(&response);

    if extracted_files.is_empty() {
        println!("No fixes generated. The issues may require manual intervention.");
        return Ok(FixResult::failure("No code extracted from LLM response".to_string()));
    }

    // Write the fixed file(s)
    let mut files_written = 0;
    for file in &extracted_files {
        let target_path = if let Some(ref path) = file.path {
            project_root.join(path)
        } else {
            full_output_path.clone()
        };

        if let Some(parent) = target_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        fs::write(&target_path, &file.content)?;
        println!("  Wrote fixed file: {}", target_path.display());
        files_written += 1;
    }

    Ok(FixResult::success(files_written))
}

/// Run a verification command and return the output
fn run_verification_command(
    project_root: &PathBuf,
    command: &str,
    file_path: &PathBuf,
) -> Result<(bool, String), WorkSplitError> {
    let full_cmd = format!("{} {}", command, file_path.display());
    let output = Command::new("sh")
        .arg("-c")
        .arg(&full_cmd)
        .current_dir(project_root)
        .output()
        .map_err(|e| WorkSplitError::IoError(format!("Failed to run command: {}", e)))?;

    let combined_output = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    Ok((output.status.success(), combined_output))
}

/// Auto-fix linter errors for a specific job using LLM
pub async fn fix_job(project_root: &PathBuf, job_id: &str) -> Result<(), WorkSplitError> {
    let config = load_config(project_root, None, None, None, false)?;

    // Get lint command
    let lint_cmd = config
        .build
        .lint_command
        .as_ref()
        .ok_or_else(|| WorkSplitError::ConfigError("No lint_command configured in worksplit.toml. Add [build] lint_command = \"your-linter\"".into()))?;

    // Get job output path
    let jobs_manager = JobsManager::new(project_root.clone(), config.limits.clone());
    let job = jobs_manager.parse_job(job_id)?;
    let output_path = job.metadata.output_path();
    let full_output_path = project_root.join(&output_path);

    if !full_output_path.exists() {
        return Err(WorkSplitError::JobError(format!(
            "Output file does not exist: {}",
            full_output_path.display()
        )));
    }

    println!("Running linter on {}...", output_path.display());

    // Run linter and capture output
    let (success, lint_output) = run_verification_command(project_root, lint_cmd, &full_output_path)?;

    if success && lint_output.trim().is_empty() {
        println!("No lint errors found!");
        return Ok(());
    }

    println!("Lint errors found. Generating fixes...\n");
    println!("{}", lint_output);

    // Try to fix with configured number of attempts
    let max_attempts = config.build.auto_fix_attempts;
    let mut attempt = 0;
    let mut current_error = lint_output;

    while attempt < max_attempts {
        attempt += 1;
        println!("\nFix attempt {}/{}...", attempt, max_attempts);

        let result = fix_with_error_context(
            project_root,
            &output_path,
            &current_error,
            ErrorType::Lint,
            &config,
        ).await?;

        if !result.success {
            println!("Fix attempt {} failed: {}", attempt, result.error.unwrap_or_default());
            continue;
        }

        // Verify the fix
        println!("\nVerifying fix...");
        let (success, new_output) = run_verification_command(project_root, lint_cmd, &full_output_path)?;

        if success {
            println!("All lint errors fixed!");
            return Ok(());
        }

        current_error = new_output;
        println!("Some issues remain:\n{}", current_error);
    }

    println!("\nRun `worksplit fix {}` again or fix manually.", job_id);
    Ok(())
}

/// Fix all failed jobs
pub async fn fix_all_jobs(project_root: &PathBuf) -> Result<FixSummary, WorkSplitError> {
    let config = load_config(project_root, None, None, None, false)?;

    // Load status manager to find failed jobs
    let jobs_dir = project_root.join("jobs");
    let status_manager = StatusManager::new(&jobs_dir)?;
    let failed_jobs = status_manager.get_by_status(JobStatus::Fail);

    let jobs_manager = JobsManager::new(project_root.clone(), config.limits.clone());

    let mut summary = FixSummary {
        fixed: 0,
        failed: 0,
        skipped: 0,
    };

    // Get lint command (required for fix-all)
    let lint_cmd = match &config.build.lint_command {
        Some(cmd) => cmd.clone(),
        None => {
            println!("No lint_command configured. Skipping lint-based fixes.");
            return Ok(summary);
        }
    };

    // Collect job IDs to process (need to clone since we borrow from status_manager)
    let job_ids: Vec<String> = failed_jobs.iter().map(|e| e.id.clone()).collect();

    for job_id in job_ids {

        println!("\n--- Fixing job: {} ---", job_id);

        // Parse job to get output path
        let job = match jobs_manager.parse_job(&job_id) {
            Ok(j) => j,
            Err(e) => {
                println!("Failed to parse job {}: {}", job_id, e);
                summary.failed += 1;
                continue;
            }
        };

        let output_path = job.metadata.output_path();
        let full_output_path = project_root.join(&output_path);

        if !full_output_path.exists() {
            println!("Output file does not exist: {}", full_output_path.display());
            summary.skipped += 1;
            continue;
        }

        // Run linter to get current errors
        let (success, lint_output) = match run_verification_command(project_root, &lint_cmd, &full_output_path) {
            Ok(result) => result,
            Err(e) => {
                println!("Failed to run linter: {}", e);
                summary.failed += 1;
                continue;
            }
        };

        if success && lint_output.trim().is_empty() {
            println!("No lint errors found for {}.", job_id);
            summary.skipped += 1;
            continue;
        }

        // Try to fix
        let max_attempts = config.build.auto_fix_attempts;
        let mut attempt = 0;
        let mut current_error = lint_output;
        let mut fixed = false;

        while attempt < max_attempts && !fixed {
            attempt += 1;
            println!("Fix attempt {}/{}...", attempt, max_attempts);

            match fix_with_error_context(
                project_root,
                &output_path,
                &current_error,
                ErrorType::Lint,
                &config,
            ).await {
                Ok(result) => {
                    if !result.success {
                        continue;
                    }

                    // Verify the fix
                    match run_verification_command(project_root, &lint_cmd, &full_output_path) {
                        Ok((success, new_output)) => {
                            if success {
                                println!("Fixed: {}", job_id);
                                fixed = true;
                            } else {
                                current_error = new_output;
                            }
                        }
                        Err(e) => {
                            println!("Verification failed: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("Fix error: {}", e);
                }
            }
        }

        if fixed {
            summary.fixed += 1;
        } else {
            summary.failed += 1;
        }
    }

    println!("\n--- Fix Summary ---");
    println!("Fixed: {}", summary.fixed);
    println!("Failed: {}", summary.failed);
    println!("Skipped: {}", summary.skipped);

    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fix_result_success() {
        let result = FixResult::success(3);
        assert!(result.success);
        assert_eq!(result.files_written, 3);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_fix_result_failure() {
        let result = FixResult::failure("Some error".to_string());
        assert!(!result.success);
        assert_eq!(result.files_written, 0);
        assert_eq!(result.error, Some("Some error".to_string()));
    }

    #[test]
    fn test_fix_result_no_errors() {
        let result = FixResult::no_errors();
        assert!(result.success);
        assert_eq!(result.files_written, 0);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_fix_summary_default() {
        let summary = FixSummary {
            fixed: 5,
            failed: 2,
            skipped: 3,
        };
        assert_eq!(summary.fixed, 5);
        assert_eq!(summary.failed, 2);
        assert_eq!(summary.skipped, 3);
    }
}
