use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::core::{load_config, parse_edit_instructions, apply_edit, JobsManager, OllamaClient};
use crate::error::WorkSplitError;

/// Auto-fix linter errors using LLM
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
    let full_cmd = format!("{} {}", lint_cmd, full_output_path.display());
    let output = Command::new("sh")
        .arg("-c")
        .arg(&full_cmd)
        .current_dir(project_root)
        .output()
        .map_err(|e| WorkSplitError::IoError(format!("Failed to run lint command: {}", e)))?;

    let lint_output = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    if output.status.success() && lint_output.trim().is_empty() {
        println!("No lint errors found!");
        return Ok(());
    }

    println!("Lint errors found. Generating fixes...\n");
    println!("{}", lint_output);

    // Load fix system prompt
    let jobs_dir = project_root.join("jobs");
    let fix_prompt_path = jobs_dir.join("_systemprompt_fix.md");
    let system_prompt = fs::read_to_string(&fix_prompt_path).map_err(|e| {
        WorkSplitError::IoError(format!(
            "Failed to read {}: {}. Run `worksplit init` to create it.",
            fix_prompt_path.display(),
            e
        ))
    })?;

    // Read the source file content
    let file_content = fs::read_to_string(&full_output_path)?;

    // Build user prompt
    let user_prompt = format!(
        "## Linter Output\n\n```\n{}\n```\n\n## Source File: {}\n\n```\n{}\n```\n\nGenerate FIND/REPLACE blocks to fix the issues above.",
        lint_output.trim(),
        output_path.display(),
        file_content
    );

    // Call Ollama
    let client = OllamaClient::new(config.ollama.clone())
        .map_err(WorkSplitError::Ollama)?;

    println!("Calling LLM for fixes...");
    let response = client
        .generate(Some(&system_prompt), &user_prompt, config.behavior.stream_output)
        .await
        .map_err(WorkSplitError::Ollama)?;

    // Parse edit instructions
    let parsed_edits = parse_edit_instructions(&response);

    if parsed_edits.edits.is_empty() {
        println!("No fixes generated. The issues may require manual intervention.");
        return Ok(());
    }

    println!("\nApplying {} edit(s)...", parsed_edits.edits.len());

    // Apply edits to the file
    let mut current_content = file_content;
    let mut applied_count = 0;
    let mut failed_count = 0;

    for edit in &parsed_edits.edits {
        match apply_edit(&current_content, edit) {
            Ok(new_content) => {
                current_content = new_content;
                applied_count += 1;
                println!("  Applied edit to {}", edit.file_path.display());
            }
            Err(e) => {
                failed_count += 1;
                eprintln!("  Failed to apply edit: {}", e);
            }
        }
    }

    if applied_count > 0 {
        // Write the updated content back
        fs::write(&full_output_path, &current_content)?;
        println!("\nWrote {} edit(s) to {}", applied_count, output_path.display());
    }

    if failed_count > 0 {
        println!("\n{} edit(s) failed to apply.", failed_count);
    }

    println!("\nRunning linter again to verify...");

    // Re-run linter to verify
    let verify_output = Command::new("sh")
        .arg("-c")
        .arg(&full_cmd)
        .current_dir(project_root)
        .output()
        .map_err(|e| WorkSplitError::IoError(format!("Failed to run lint command: {}", e)))?;

    if verify_output.status.success() {
        println!("All lint errors fixed!");
        Ok(())
    } else {
        let remaining = format!(
            "{}{}",
            String::from_utf8_lossy(&verify_output.stdout),
            String::from_utf8_lossy(&verify_output.stderr)
        );
        println!("Some issues remain:\n{}", remaining);
        println!("\nRun `worksplit fix {}` again or fix manually.", job_id);
        Ok(())
    }
}
