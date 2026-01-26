use std::fs;
use std::path::PathBuf;
use tracing::info;

use crate::error::WorkSplitError;
use crate::models::JobTemplate;

/// Create a new job from a template
pub fn create_new_job(
    project_root: &PathBuf,
    name: &str,
    template: JobTemplate,
    target_files: Option<Vec<PathBuf>>,
    output_dir: &PathBuf,
    output_file: Option<String>,
    context_files: Option<Vec<PathBuf>>,
) -> Result<(), WorkSplitError> {
    // Validate job name
    validate_job_name(name)?;

    let jobs_dir = project_root.join("jobs");

    // Ensure jobs directory exists
    if !jobs_dir.exists() {
        fs::create_dir_all(&jobs_dir)?;
        info!("Created jobs directory: {}", jobs_dir.display());
    }

    // Check if job file already exists
    let job_file = jobs_dir.join(format!("{}.md", name));
    if job_file.exists() {
        return Err(WorkSplitError::JobAlreadyExists(name.to_string()));
    }

    // Generate template content
    let content = generate_template(
        template,
        name,
        target_files.as_ref(),
        output_dir,
        output_file.as_ref(),
        context_files.as_ref(),
    );

    // Write the job file
    fs::write(&job_file, &content)?;
    info!("Created job file: {}", job_file.display());

    // Print success message
    println!("Created job: jobs/{}.md", name);
    println!();
    println!("Template: {:?}", template);

    if let Some(ref targets) = target_files {
        println!("Target files:");
        for target in targets {
            println!("  - {}", target.display());
        }
    }

    println!();
    println!("Next steps:");
    println!("1. Edit the job file to add specific requirements");
    println!("2. Run 'worksplit validate' to check the job");
    println!("3. Run 'worksplit run --job {}' to execute", name);

    Ok(())
}

/// Validate that the job name is valid
fn validate_job_name(name: &str) -> Result<(), WorkSplitError> {
    if name.is_empty() {
        return Err(WorkSplitError::InvalidJobName(
            "Job name cannot be empty".to_string(),
        ));
    }

    // Check for valid characters (alphanumeric, underscores, hyphens)
    for c in name.chars() {
        if !c.is_alphanumeric() && c != '_' && c != '-' {
            return Err(WorkSplitError::InvalidJobName(format!(
                "Invalid character '{}' in job name. Use only alphanumeric, underscore, or hyphen.",
                c
            )));
        }
    }

    // Check it doesn't start with underscore (reserved for system files)
    if name.starts_with('_') {
        return Err(WorkSplitError::InvalidJobName(
            "Job name cannot start with underscore (reserved for system files)".to_string(),
        ));
    }

    Ok(())
}

/// Generate template content based on template type
fn generate_template(
    template: JobTemplate,
    name: &str,
    target_files: Option<&Vec<PathBuf>>,
    output_dir: &PathBuf,
    output_file: Option<&String>,
    context_files: Option<&Vec<PathBuf>>,
) -> String {
    let output_dir_str = output_dir.display().to_string();
    let default_output_file = format!("{}.rs", name.split('_').last().unwrap_or(name));
    let output_file_str = output_file
        .cloned()
        .unwrap_or(default_output_file);

    match template {
        JobTemplate::Replace => generate_replace_template(
            name,
            &output_dir_str,
            &output_file_str,
            context_files,
        ),
        JobTemplate::Edit => generate_edit_template(
            name,
            target_files,
            &output_dir_str,
            &output_file_str,
        ),
        JobTemplate::Split => generate_split_template(
            name,
            &output_dir_str,
            &output_file_str,
        ),
        JobTemplate::Sequential => generate_sequential_template(
            name,
            &output_dir_str,
            &output_file_str,
        ),
        JobTemplate::Tdd => generate_tdd_template(
            name,
            &output_dir_str,
            &output_file_str,
            context_files,
        ),
    }
}

fn format_context_files(context_files: Option<&Vec<PathBuf>>) -> String {
    match context_files {
        Some(files) if !files.is_empty() => {
            let formatted: Vec<String> = files
                .iter()
                .map(|p| format!("  - {}", p.display()))
                .collect();
            formatted.join("\n")
        }
        _ => "  # - src/example.rs".to_string(),
    }
}

fn format_target_files(target_files: Option<&Vec<PathBuf>>) -> String {
    match target_files {
        Some(files) if !files.is_empty() => {
            let formatted: Vec<String> = files
                .iter()
                .map(|p| format!("  - {}", p.display()))
                .collect();
            formatted.join("\n")
        }
        _ => "  - src/main.rs".to_string(),
    }
}

fn generate_replace_template(
    name: &str,
    output_dir: &str,
    output_file: &str,
    context_files: Option<&Vec<PathBuf>>,
) -> String {
    let ctx = format_context_files(context_files);
    let title = name_to_title(name);

    format!(
        r#"---
context_files:
{ctx}
output_dir: {output_dir}
output_file: {output_file}
---

# {title}

## Requirements
- Describe what to implement
- List key behaviors and constraints

## Functions to Implement

1. `function_name(param: Type) -> ReturnType`
   - Description of what it does
   - Error conditions

2. `another_function() -> Result<T, E>`
   - Description

## Example Usage

```rust
// Show expected usage
```
"#,
        ctx = ctx,
        output_dir = output_dir,
        output_file = output_file,
        title = title,
    )
}

fn generate_edit_template(
    name: &str,
    target_files: Option<&Vec<PathBuf>>,
    output_dir: &str,
    output_file: &str,
) -> String {
    let targets = format_target_files(target_files);
    let title = name_to_title(name);

    format!(
        r#"---
mode: edit
target_files:
{targets}
output_dir: {output_dir}
output_file: {output_file}
---

# {title}

## Overview
Describe the surgical changes to make.

## Changes Required
- Change 1: What and why
- Change 2: What and why

## Edit Instructions

Describe the specific edits using FILE/FIND/REPLACE/END format:

- Add a new field to struct X
- Update function signature for Y
- Add import for Z
"#,
        targets = targets,
        output_dir = output_dir,
        output_file = output_file,
        title = title,
    )
}

fn generate_split_template(
    name: &str,
    output_dir: &str,
    _output_file: &str,
) -> String {
    let title = name_to_title(name);

    format!(
        r#"---
mode: split
target_file: src/large_file.rs
output_dir: {output_dir}
output_file: mod.rs
output_files:
  - {output_dir}mod.rs
  - {output_dir}part1.rs
  - {output_dir}part2.rs
---

# {title}

## Overview
Split large file into a directory-based module structure.

## File Structure

- `mod.rs`: Main struct and public API
- `part1.rs`: First logical group of functions
- `part2.rs`: Second logical group of functions

## Function Signatures (REQUIRED)

### part1.rs
```rust
pub(crate) fn function_one(
    param: &Type,
) -> Result<ReturnType, Error>
```

### part2.rs
```rust
pub(crate) fn function_two(
    param: &Type,
) -> Result<ReturnType, Error>
```

## Extraction Plan

- `function_one` and related helpers -> part1.rs
- `function_two` and related helpers -> part2.rs
- Struct definitions and public API remain in mod.rs
"#,
        output_dir = output_dir,
        title = title,
    )
}

fn generate_sequential_template(
    name: &str,
    output_dir: &str,
    output_file: &str,
) -> String {
    let title = name_to_title(name);

    format!(
        r#"---
output_files:
  - {output_dir}file1.rs
  - {output_dir}file2.rs
  - {output_dir}file3.rs
sequential: true
output_dir: {output_dir}
output_file: {output_file}
---

# {title}

## Overview
Multi-file feature that exceeds single-call token limits.

## File Responsibilities

### file1.rs
- Core types and traits
- Shared utilities

### file2.rs
- Main implementation
- Uses types from file1.rs

### file3.rs
- Additional functionality
- Depends on file1.rs and file2.rs

## Implementation Notes
- Each file gets its own LLM call
- Previously generated files become context for subsequent calls
- Ensure consistent imports across files
"#,
        output_dir = output_dir,
        output_file = output_file,
        title = title,
    )
}

fn generate_tdd_template(
    name: &str,
    output_dir: &str,
    output_file: &str,
    context_files: Option<&Vec<PathBuf>>,
) -> String {
    let ctx = format_context_files(context_files);
    let title = name_to_title(name);
    let test_file = output_file.replace(".rs", "_test.rs");

    format!(
        r#"---
context_files:
{ctx}
output_dir: {output_dir}
output_file: {output_file}
test_file: {test_file}
---

# {title} (TDD)

Tests will be generated first, then implementation.

## Requirements
- Feature requirement 1
- Feature requirement 2

## Functions to Implement

1. `function(param: Type) -> Result<T, E>`
   - Description
   - Error conditions

## Expected Behavior

- `function(valid_input)` returns expected output
- `function(invalid_input)` returns appropriate error
- Edge cases to test

## Test Cases

1. Happy path: valid input produces expected output
2. Error case: invalid input returns error
3. Edge case: boundary conditions
"#,
        ctx = ctx,
        output_dir = output_dir,
        output_file = output_file,
        test_file = test_file,
        title = title,
    )
}

/// Convert job name to title case
fn name_to_title(name: &str) -> String {
    name.split('_')
        .filter(|s| !s.chars().all(|c| c.is_numeric()))
        .map(|s| {
            let mut chars = s.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_job_name_valid() {
        assert!(validate_job_name("auth_001_login").is_ok());
        assert!(validate_job_name("feature-test").is_ok());
        assert!(validate_job_name("simple").is_ok());
    }

    #[test]
    fn test_validate_job_name_empty() {
        assert!(validate_job_name("").is_err());
    }

    #[test]
    fn test_validate_job_name_invalid_chars() {
        assert!(validate_job_name("has space").is_err());
        assert!(validate_job_name("has.dot").is_err());
        assert!(validate_job_name("has/slash").is_err());
    }

    #[test]
    fn test_validate_job_name_underscore_prefix() {
        assert!(validate_job_name("_system").is_err());
    }

    #[test]
    fn test_name_to_title() {
        assert_eq!(name_to_title("auth_001_login"), "Auth Login");
        assert_eq!(name_to_title("template_002_new_job"), "Template New Job");
        assert_eq!(name_to_title("simple"), "Simple");
    }
}
