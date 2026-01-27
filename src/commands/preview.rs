use std::fs;
use std::path::PathBuf;

use crate::core::{load_config, JobsManager};
use crate::error::WorkSplitError;
use crate::models::OutputMode;

/// Preview the prompt for a job without running it
pub fn preview_job(project_root: &PathBuf, job_id: &str) -> Result<(), WorkSplitError> {
    // Load config
    let config = load_config(project_root, None, None, None, false)?;

    // Create jobs manager
    let jobs_manager = JobsManager::new(project_root.clone(), config.limits.clone());

    // Parse job
    let job = jobs_manager.parse_job(job_id)?;

    // Display job info
    println!("=== JOB PREVIEW: {} ===\n", job_id);
    println!("Mode: {:?}", job.metadata.mode);
    println!("Output: {}", job.metadata.output_path().display());

    // Show context files
    if !job.metadata.context_files.is_empty() {
        println!("\nContext files:");
        for ctx_file in &job.metadata.context_files {
            let full_path = project_root.join(ctx_file);
            if let Ok(content) = fs::read_to_string(&full_path) {
                println!("  - {} ({} lines)", ctx_file.display(), content.lines().count());
            } else {
                println!("  - {} (not found)", ctx_file.display());
            }
        }
    }

    // Show target files for edit mode
    if let Some(target_files) = &job.metadata.target_files {
        println!("\nTarget files (edit mode):");
        for target in target_files {
            let full_path = project_root.join(target);
            if let Ok(content) = fs::read_to_string(&full_path) {
                println!("  - {} ({} lines)", target.display(), content.lines().count());
            } else {
                println!("  - {} (not found)", target.display());
            }
        }
    }

    // Load and display system prompt
    let jobs_dir = project_root.join("jobs");
    let system_prompt_name = match job.metadata.mode {
        OutputMode::Edit => "_systemprompt_edit.md",
        OutputMode::Split => "_systemprompt_split.md",
        _ => "_systemprompt_create.md",
    };
    let system_prompt_path = jobs_dir.join(system_prompt_name);

    println!("\n=== SYSTEM PROMPT ({}) ===", system_prompt_name);
    if let Ok(system_prompt) = fs::read_to_string(&system_prompt_path) {
        // Show first 20 lines to keep output manageable
        let preview_lines: Vec<&str> = system_prompt.lines().take(20).collect();
        for line in &preview_lines {
            println!("{}", line);
        }
        let total_lines = system_prompt.lines().count();
        if total_lines > 20 {
            println!("... ({} more lines)", total_lines - 20);
        }
    } else {
        println!("(system prompt file not found)");
    }

    // Display the job instructions
    println!("\n=== JOB INSTRUCTIONS ===");
    let instruction_lines: Vec<&str> = job.instructions.lines().take(30).collect();
    for line in &instruction_lines {
        println!("{}", line);
    }
    let total_instruction_lines = job.instructions.lines().count();
    if total_instruction_lines > 30 {
        println!("... ({} more lines)", total_instruction_lines - 30);
    }

    // Show context file contents (abbreviated)
    if !job.metadata.context_files.is_empty() {
        for ctx_file in &job.metadata.context_files {
            let full_path = project_root.join(ctx_file);
            if let Ok(content) = fs::read_to_string(&full_path) {
                println!("\n=== CONTEXT: {} ===", ctx_file.display());
                let context_lines: Vec<&str> = content.lines().take(20).collect();
                for line in &context_lines {
                    println!("{}", line);
                }
                let total = content.lines().count();
                if total > 20 {
                    println!("... ({} more lines)", total - 20);
                }
            }
        }
    }

    // Estimate tokens (rough estimate: 4 chars per token)
    let mut total_chars = job.instructions.len();
    for ctx_file in &job.metadata.context_files {
        let full_path = project_root.join(ctx_file);
        if let Ok(content) = fs::read_to_string(&full_path) {
            total_chars += content.len();
        }
    }
    if let Some(target_files) = &job.metadata.target_files {
        for target in target_files {
            let full_path = project_root.join(target);
            if let Ok(content) = fs::read_to_string(&full_path) {
                total_chars += content.len();
            }
        }
    }

    let token_estimate = total_chars / 4;
    println!("\n=== METADATA ===");
    println!("Model: {}", config.ollama.model);
    println!("Estimated input tokens: ~{}", token_estimate);
    println!("Timeout: {}s", config.ollama.timeout_seconds);

    Ok(())
}
