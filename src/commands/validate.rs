use std::path::PathBuf;

use crate::core::JobsManager;
use crate::error::WorkSplitError;
use crate::models::Config;

/// Validation result
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Validate jobs folder structure
pub fn validate_jobs(project_root: &PathBuf) -> Result<ValidationResult, WorkSplitError> {
    let mut result = ValidationResult {
        valid: true,
        errors: Vec::new(),
        warnings: Vec::new(),
    };

    let jobs_dir = project_root.join("jobs");

    // Check jobs folder exists
    if !jobs_dir.exists() {
        result.errors.push("Jobs folder not found".to_string());
        result.valid = false;
        return Ok(result);
    }

    // Check system prompts exist
    let create_prompt = jobs_dir.join("_systemprompt_create.md");
    if !create_prompt.exists() {
        result.errors.push("Missing _systemprompt_create.md".to_string());
        result.valid = false;
    }

    let verify_prompt = jobs_dir.join("_systemprompt_verify.md");
    if !verify_prompt.exists() {
        result.errors.push("Missing _systemprompt_verify.md".to_string());
        result.valid = false;
    }

    // Check manager instructions (optional)
    let manager = jobs_dir.join("_managerinstruction.md");
    if !manager.exists() {
        result.warnings.push("Missing _managerinstruction.md (optional)".to_string());
    }

    // Check job status file
    let status_file = jobs_dir.join("_jobstatus.json");
    if !status_file.exists() {
        result.warnings.push("Missing _jobstatus.json (will be created on first run)".to_string());
    }

    // Load config from worksplit.toml (or use defaults)
    let config = Config::load_from_dir(project_root).unwrap_or_default();

    // Validate individual job files
    let jobs_manager = JobsManager::new(project_root.clone(), config.limits);
    match jobs_manager.discover_jobs() {
        Ok(jobs) => {
            if jobs.is_empty() {
                result.warnings.push("No job files found".to_string());
            } else {
                for job_id in jobs {
                    match jobs_manager.parse_job(&job_id) {
                        Ok(job) => {
                            // Validate context files exist
                            for context_file in &job.metadata.context_files {
                                let full_path = project_root.join(context_file);
                                if !full_path.exists() {
                                    result.warnings.push(format!(
                                        "Job '{}': Context file not found: {}",
                                        job_id,
                                        context_file.display()
                                    ));
                                }
                            }

                            // Check output directory
                            let output_dir = project_root.join(&job.metadata.output_dir);
                            if !output_dir.exists() {
                                result.warnings.push(format!(
                                    "Job '{}': Output directory does not exist: {} (will be created)",
                                    job_id,
                                    job.metadata.output_dir.display()
                                ));
                            }
                        }
                        Err(e) => {
                            result.errors.push(format!("Job '{}': {}", job_id, e));
                            result.valid = false;
                        }
                    }
                }
            }
        }
        Err(e) => {
            result.errors.push(format!("Failed to discover jobs: {}", e));
            result.valid = false;
        }
    }

    // Check config file
    let config_file = project_root.join("worksplit.toml");
    if !config_file.exists() {
        result.warnings.push("Missing worksplit.toml (using defaults)".to_string());
    }

    Ok(result)
}

/// Print validation result
pub fn print_validation_result(result: &ValidationResult) {
    println!("=== Validation Result ===\n");

    if result.valid {
        println!("Status: VALID\n");
    } else {
        println!("Status: INVALID\n");
    }

    if !result.errors.is_empty() {
        println!("Errors:");
        for error in &result.errors {
            println!("  - {}", error);
        }
        println!();
    }

    if !result.warnings.is_empty() {
        println!("Warnings:");
        for warning in &result.warnings {
            println!("  - {}", warning);
        }
        println!();
    }

    if result.valid && result.errors.is_empty() && result.warnings.is_empty() {
        println!("All checks passed!");
    }
}
