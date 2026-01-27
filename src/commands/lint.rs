use std::path::PathBuf;
use std::process::Command;

use crate::core::{load_config, JobsManager, StatusManager};
use crate::error::WorkSplitError;
use crate::models::JobStatus;

/// Run linter on generated files
pub fn lint_jobs(project_root: &PathBuf, job_id: Option<&str>) -> Result<(), WorkSplitError> {
    let config = load_config(project_root, None, None, None, false)?;

    let lint_cmd = config
        .build
        .lint_command
        .as_ref()
        .ok_or_else(|| WorkSplitError::ConfigError("No lint_command configured in worksplit.toml. Add [build] lint_command = \"your-linter\"".into()))?;

    let jobs_manager = JobsManager::new(project_root.clone(), config.limits.clone());

    // Get files to lint
    let files: Vec<PathBuf> = if let Some(id) = job_id {
        // Lint specific job's output
        let job = jobs_manager.parse_job(id)?;
        let output_path = job.metadata.output_path();
        let full_path = project_root.join(&output_path);
        if full_path.exists() {
            vec![output_path]
        } else {
            return Err(WorkSplitError::JobError(format!(
                "Output file does not exist: {}",
                full_path.display()
            )));
        }
    } else {
        // Lint all passed jobs' outputs
        let status_manager = StatusManager::new(jobs_manager.jobs_dir())?;
        let job_ids = jobs_manager.discover_jobs()?;
        
        job_ids
            .into_iter()
            .filter(|id| {
                status_manager.get(id)
                    .map(|entry| entry.status == JobStatus::Pass)
                    .unwrap_or(false)
            })
            .filter_map(|id| {
                jobs_manager.parse_job(&id).ok().and_then(|job| {
                    let output_path = job.metadata.output_path();
                    let full_path = project_root.join(&output_path);
                    if full_path.exists() {
                        Some(output_path)
                    } else {
                        None
                    }
                })
            })
            .collect()
    };

    if files.is_empty() {
        println!("No files to lint");
        return Ok(());
    }

    println!("Linting {} file(s)...\n", files.len());

    let mut has_errors = false;
    for file in &files {
        // Build command with file argument
        let full_cmd = format!("{} {}", lint_cmd, file.display());
        println!("$ {}", full_cmd);

        let output = Command::new("sh")
            .arg("-c")
            .arg(&full_cmd)
            .current_dir(project_root)
            .output()
            .map_err(|e| WorkSplitError::IoError(format!("Failed to run lint command: {}", e)))?;

        if !output.stdout.is_empty() {
            println!("{}", String::from_utf8_lossy(&output.stdout));
        }
        if !output.stderr.is_empty() {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }

        if !output.status.success() {
            has_errors = true;
        }
    }

    if has_errors {
        println!("\nLint errors found. Run `worksplit fix <job>` to auto-fix.");
        Err(WorkSplitError::LintError("Lint errors found".into()))
    } else {
        println!("\nAll files passed lint checks!");
        Ok(())
    }
}
