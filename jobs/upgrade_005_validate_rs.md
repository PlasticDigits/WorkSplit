---
context_files:
  - src/commands/validate.rs
  - src/models/job.rs
output_dir: src/commands/
output_file: validate.rs
---

# Add Quality Validation Warnings to validate.rs

Implement Task 5: Extend `worksplit validate` to warn about low-quality jobs.

## Quality Checks to Add

1. Jobs with no "## Requirements" section (case-insensitive)
2. Jobs with very short instructions (<100 chars of markdown body)
3. Jobs with no function signatures (no backtick code blocks with function-like patterns)
4. Jobs referencing non-existent context files (already implemented, keep it)

## Complete Implementation

```rust
use std::path::PathBuf;

use crate::core::JobsManager;
use crate::error::WorkSplitError;
use crate::models::LimitsConfig;

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

    // Validate individual job files
    let jobs_manager = JobsManager::new(project_root.clone(), LimitsConfig::default());
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

                            // === NEW QUALITY CHECKS ===

                            // Check 1: No "## Requirements" section
                            if !has_requirements_section(&job.instructions) {
                                result.warnings.push(format!(
                                    "Job '{}': No '## Requirements' section found (jobs may be vague)",
                                    job_id
                                ));
                            }

                            // Check 2: Very short instructions (<100 chars)
                            let instruction_len = job.instructions.trim().len();
                            if instruction_len < 100 {
                                result.warnings.push(format!(
                                    "Job '{}': Very short instructions ({} chars, recommend >100)",
                                    job_id,
                                    instruction_len
                                ));
                            }

                            // Check 3: No function signatures or code examples
                            if !has_code_examples(&job.instructions) {
                                result.warnings.push(format!(
                                    "Job '{}': No code examples or function signatures found",
                                    job_id
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

/// Check if instructions contain a "## Requirements" section (case-insensitive)
fn has_requirements_section(instructions: &str) -> bool {
    let lower = instructions.to_lowercase();
    lower.contains("## requirements") || lower.contains("##requirements")
}

/// Check if instructions contain code examples (backtick blocks with function-like patterns)
fn has_code_examples(instructions: &str) -> bool {
    // Look for code fences
    if !instructions.contains("```") && !instructions.contains("`") {
        return false;
    }

    // Look for function-like patterns inside backticks
    // Patterns: fn, pub fn, async fn, def, function, ->, ()
    let patterns = [
        "fn ",
        "pub fn",
        "async fn",
        "def ",
        "function ",
        "func ",
        "->",
        "()",
        "pub struct",
        "struct ",
        "impl ",
        "trait ",
        "class ",
    ];

    for pattern in patterns {
        if instructions.contains(pattern) {
            return true;
        }
    }

    false
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
```

## Quality Check Details

### Check 1: Requirements Section
Looks for `## Requirements` or `##Requirements` (case-insensitive). This is the most common section for listing what a job should produce.

### Check 2: Short Instructions
Instructions under 100 characters are likely too vague. This catches jobs like:
```markdown
# Create a user service
Make it work with the user model.
```

### Check 3: Code Examples
Looks for:
- Code fence markers (```)
- Function-like patterns: `fn `, `pub fn`, `async fn`, `def `, `function `, `->`, `()`
- Type definitions: `struct `, `impl `, `trait `, `class `

Jobs without code examples often produce unpredictable results.

## Constraints

- Use 4-space indentation
- Keep existing validation logic intact
- Add new checks as warnings (not errors) since they're quality hints
- Warnings don't set valid = false
