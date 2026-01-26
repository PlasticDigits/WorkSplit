---
context_files:
  - src/core/runner/mod.rs
  - src/models/job.rs
output_dir: src/core/runner/
output_file: mod.rs
mode: edit
target_files:
  - src/core/runner/mod.rs
---

# Add Runner Support for Replace Pattern and Update Fixtures Modes

Add mode handling in the runner for replace_pattern and update_fixtures modes.

## Requirements

### 1. Import new parser functions

Add to the imports at the top of the file (around line 14):

```rust
use crate::core::{
    // ... existing imports ...
    parse_replace_pattern_instructions, apply_replace_patterns,
    find_struct_literals, insert_field_into_struct_literals,
};
```

### 2. Add handling for replace_pattern mode in run_job

In the `run_job` method, add handling for the new modes after the existing mode checks (after the `is_split_mode()` block, around line 430):

Find this pattern:
```rust
        } else if job.metadata.is_edit_mode() {
```

Add before it:
```rust
        } else if job.metadata.mode == OutputMode::ReplacePattern {
            info!("Replace pattern mode for job '{}'", job_id);
            let result = process_replace_pattern_mode(
                &self.ollama,
                &self.project_root,
                &self.config,
                &job,
                &context_files,
                &create_prompt,
            ).await?;
            generated_files = result.0;
            full_output_paths = result.1;
            total_lines = result.2;
        } else if job.metadata.mode == OutputMode::UpdateFixtures {
            info!("Update fixtures mode for job '{}'", job_id);
            let result = process_update_fixtures_mode(
                &self.ollama,
                &self.project_root,
                &self.config,
                &job,
                &context_files,
                &create_prompt,
            ).await?;
            generated_files = result.0;
            full_output_paths = result.1;
            total_lines = result.2;
```

### 3. Add process_replace_pattern_mode function

Add this function before the `impl Runner` block or as a private function:

```rust
/// Process replace_pattern mode job
async fn process_replace_pattern_mode(
    ollama: &OllamaClient,
    project_root: &Path,
    config: &Config,
    job: &Job,
    context_files: &[(PathBuf, String)],
    create_prompt: &str,
) -> Result<(Vec<(PathBuf, String)>, Vec<PathBuf>, usize), WorkSplitError> {
    let target_files = job.metadata.get_target_files();
    let mut target_file_contents: Vec<(PathBuf, String)> = Vec::new();
    
    for path in &target_files {
        let content = fs::read_to_string(project_root.join(path))?;
        target_file_contents.push((path.clone(), content));
    }
    
    // Build a specialized prompt for replace_pattern mode
    let mut prompt = String::new();
    prompt.push_str("[SYSTEM]\n");
    prompt.push_str(create_prompt);
    prompt.push_str("\n\n[REPLACE PATTERN MODE]\n");
    prompt.push_str("Generate AFTER/INSERT instructions to apply batch text replacements.\n\n");
    prompt.push_str("Format:\n");
    prompt.push_str("SCOPE: #[cfg(test)]  (optional - restrict to scope)\n");
    prompt.push_str("AFTER:\n<text to find>\nINSERT:\n<text to add after>\n\n");
    
    prompt.push_str("[TARGET FILES]\n");
    for (path, content) in &target_file_contents {
        prompt.push_str(&format!("### File: {}\n```\n{}\n```\n\n", path.display(), content));
    }
    
    prompt.push_str("[INSTRUCTIONS]\n");
    prompt.push_str(&job.instructions);
    
    let response = ollama.generate_with_retry(Some(SYSTEM_PROMPT_CREATE), &prompt, config.behavior.stream_output)
        .await
        .map_err(WorkSplitError::Ollama)?;
    
    let patterns = parse_replace_pattern_instructions(&response);
    
    let mut generated_files: Vec<(PathBuf, String)> = Vec::new();
    let mut full_output_paths: Vec<PathBuf> = Vec::new();
    let mut total_lines = 0;
    
    for (path, original_content) in &target_file_contents {
        let modified = apply_replace_patterns(original_content, &patterns)
            .map_err(|e| WorkSplitError::EditFailed(e))?;
        
        total_lines += count_lines(&modified);
        let full_path = project_root.join(path);
        fs::write(&full_path, &modified)?;
        generated_files.push((path.clone(), modified));
        full_output_paths.push(full_path);
    }
    
    if generated_files.is_empty() {
        return Err(WorkSplitError::EditFailed("Replace pattern mode produced no changes".to_string()));
    }
    
    Ok((generated_files, full_output_paths, total_lines))
}
```

### 4. Add process_update_fixtures_mode function

```rust
/// Process update_fixtures mode job
async fn process_update_fixtures_mode(
    ollama: &OllamaClient,
    project_root: &Path,
    config: &Config,
    job: &Job,
    context_files: &[(PathBuf, String)],
    create_prompt: &str,
) -> Result<(Vec<(PathBuf, String)>, Vec<PathBuf>, usize), WorkSplitError> {
    let target_files = job.metadata.get_target_files();
    
    // Get struct_name and new_field from metadata
    let struct_name = job.metadata.struct_name.as_ref()
        .ok_or_else(|| WorkSplitError::EditFailed("update_fixtures mode requires struct_name".to_string()))?;
    let new_field = job.metadata.new_field.as_ref()
        .ok_or_else(|| WorkSplitError::EditFailed("update_fixtures mode requires new_field".to_string()))?;
    
    let mut generated_files: Vec<(PathBuf, String)> = Vec::new();
    let mut full_output_paths: Vec<PathBuf> = Vec::new();
    let mut total_lines = 0;
    
    for path in &target_files {
        let full_path = project_root.join(path);
        let content = fs::read_to_string(&full_path)?;
        
        // Find and update struct literals
        let modified = insert_field_into_struct_literals(&content, struct_name, new_field)
            .map_err(|e| WorkSplitError::EditFailed(e))?;
        
        if modified != content {
            total_lines += count_lines(&modified);
            fs::write(&full_path, &modified)?;
            generated_files.push((path.clone(), modified));
            full_output_paths.push(full_path);
            
            info!("Updated {} struct literals in {}", 
                find_struct_literals(&content, struct_name).len(),
                path.display());
        }
    }
    
    if generated_files.is_empty() {
        return Err(WorkSplitError::EditFailed(format!(
            "No {} struct literals found in target files", struct_name
        )));
    }
    
    Ok((generated_files, full_output_paths, total_lines))
}
```

### 5. Add helper methods to JobMetadata

The `is_replace_pattern_mode()` and `is_update_fixtures_mode()` helpers should be added to JobMetadata. Since we're editing runner/mod.rs, we'll use direct enum comparison instead:

```rust
job.metadata.mode == OutputMode::ReplacePattern
job.metadata.mode == OutputMode::UpdateFixtures
```

### 6. Add OutputMode import

Make sure OutputMode is imported at the top:

```rust
use crate::models::{Config, JobStatus, OutputMode};
```

## Edits to Apply

### Edit 1: Add import for OutputMode

FILE: src/core/runner/mod.rs
FIND:
use crate::models::{Config, JobStatus};
REPLACE:
use crate::models::{Config, JobStatus, OutputMode};
END

### Edit 2: Add new parser function imports

FILE: src/core/runner/mod.rs
FIND:
use crate::core::{
    assemble_creation_prompt, assemble_sequential_creation_prompt, assemble_test_prompt, 
    assemble_verification_prompt_multi, assemble_retry_prompt_multi, assemble_edit_prompt,
    assemble_sequential_split_prompt,
    count_lines, extract_code, extract_code_files, parse_verification, parse_edit_instructions, 
    apply_edits, JobsManager, OllamaClient, StatusManager, EditInstruction,
    DependencyGraph,
    SYSTEM_PROMPT_CREATE, SYSTEM_PROMPT_TEST,
};
REPLACE:
use crate::core::{
    assemble_creation_prompt, assemble_sequential_creation_prompt, assemble_test_prompt, 
    assemble_verification_prompt_multi, assemble_retry_prompt_multi, assemble_edit_prompt,
    assemble_sequential_split_prompt,
    count_lines, extract_code, extract_code_files, parse_verification, parse_edit_instructions, 
    apply_edits, JobsManager, OllamaClient, StatusManager, EditInstruction,
    DependencyGraph,
    parse_replace_pattern_instructions, apply_replace_patterns,
    find_struct_literals, insert_field_into_struct_literals,
    SYSTEM_PROMPT_CREATE, SYSTEM_PROMPT_TEST,
};
END

### Edit 3: Add new mode handling (before is_edit_mode check)

FILE: src/core/runner/mod.rs
FIND:
        } else if job.metadata.is_edit_mode() {
            let files = edit::process_edit_mode(
REPLACE:
        } else if job.metadata.mode == OutputMode::ReplacePattern {
            info!("Replace pattern mode for job '{}'", job_id);
            let target_files = job.metadata.get_target_files();
            for path in &target_files {
                let content = fs::read_to_string(self.project_root.join(path))?;
                let prompt = format!(
                    "[REPLACE PATTERN MODE]\nGenerate AFTER/INSERT instructions.\n\n\
                    Format:\nAFTER:\n<text>\nINSERT:\n<text>\n\n\
                    [TARGET FILE: {}]\n```\n{}\n```\n\n[INSTRUCTIONS]\n{}",
                    path.display(), content, job.instructions
                );
                let response = self.ollama.generate_with_retry(Some(SYSTEM_PROMPT_CREATE), &prompt, self.config.behavior.stream_output)
                    .await.map_err(|e| WorkSplitError::Ollama(e))?;
                let patterns = parse_replace_pattern_instructions(&response);
                let modified = apply_replace_patterns(&content, &patterns)
                    .map_err(|e| WorkSplitError::EditFailed(e))?;
                total_lines += count_lines(&modified);
                let full_path = self.project_root.join(path);
                fs::write(&full_path, &modified)?;
                generated_files.push((path.clone(), modified));
                full_output_paths.push(full_path);
            }
        } else if job.metadata.mode == OutputMode::UpdateFixtures {
            info!("Update fixtures mode for job '{}'", job_id);
            let struct_name = job.metadata.struct_name.as_ref()
                .ok_or_else(|| WorkSplitError::EditFailed("update_fixtures requires struct_name".to_string()))?;
            let new_field = job.metadata.new_field.as_ref()
                .ok_or_else(|| WorkSplitError::EditFailed("update_fixtures requires new_field".to_string()))?;
            let target_files = job.metadata.get_target_files();
            for path in &target_files {
                let full_path = self.project_root.join(path);
                let content = fs::read_to_string(&full_path)?;
                let modified = insert_field_into_struct_literals(&content, struct_name, new_field)
                    .map_err(|e| WorkSplitError::EditFailed(e))?;
                if modified != content {
                    total_lines += count_lines(&modified);
                    fs::write(&full_path, &modified)?;
                    generated_files.push((path.clone(), modified));
                    full_output_paths.push(full_path);
                }
            }
            if generated_files.is_empty() {
                return Err(WorkSplitError::EditFailed(format!("No {} literals found", struct_name)));
            }
        } else if job.metadata.is_edit_mode() {
            let files = edit::process_edit_mode(
END

## Constraints

- Preserve all existing mode handling
- Replace pattern mode should work on multiple target files
- Update fixtures mode doesn't require LLM call (it's deterministic)
- Handle errors gracefully with descriptive messages

## Formatting Notes

- Uses 4-space indentation
- Follow existing async patterns
- Use tracing::info! for logging mode operations

## Dependencies

This job depends on:
- enhance_001_models.md (for OutputMode variants)
- enhance_007_parser_patterns.md (for parser functions)
