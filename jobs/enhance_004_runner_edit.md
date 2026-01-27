---
context_files:
  - src/core/runner/edit.rs
  - src/core/parser/edit.rs
output_dir: src/core/runner/
output_file: edit.rs
verify: false
---

# Enhance Edit Runner with Dry-Run, Partial Completion, and Suggestions

Improve the edit mode runner to support dry-run preview, partial completion tracking, and actionable error suggestions.

## Requirements

### 1. Add DryRunResult struct

```rust
/// Result of a dry-run edit analysis
#[derive(Debug, Clone)]
pub struct DryRunResult {
    pub job_id: String,
    pub planned_edits: Vec<PlannedEdit>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PlannedEdit {
    pub file_path: PathBuf,
    pub line_number: Option<usize>,  // Approximate line if found
    pub find_preview: String,        // First 50 chars
    pub replace_preview: String,     // First 50 chars
    pub status: PlannedEditStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlannedEditStatus {
    /// Exact match found, will apply
    WillApply,
    /// Fuzzy match found, will apply with warning
    WillApplyFuzzy,
    /// No match found, will fail
    WillFail,
}
```

### 2. Add dry_run_edit_mode function

```rust
/// Analyze what edits would be applied without actually applying them
pub(crate) async fn dry_run_edit_mode(
    ollama: &OllamaClient,
    project_root: &Path,
    config: &Config,
    job: &Job,
    context_files: &[(PathBuf, String)],
    edit_prompt: &str,
) -> Result<DryRunResult, WorkSplitError>
```

This function should:
1. Generate edits from Ollama (same as process_edit_mode)
2. Parse the edit instructions
3. For each edit, check if it would apply:
   - Try exact match first
   - Try fuzzy match
   - Record status and line number hint
4. Return DryRunResult without modifying any files

### 3. Modify process_edit_mode for partial completion

Update `process_edit_mode` to:
1. Apply edits one-by-one instead of all-at-once
2. Track which edits succeed and which fail
3. On any failure:
   - Continue applying remaining edits to other files
   - Collect all failures with fuzzy match hints
   - Return a PartialEditResult instead of immediate error

```rust
/// Result of edit mode processing
#[derive(Debug)]
pub struct EditModeResult {
    pub generated_files: Vec<(PathBuf, String)>,
    pub output_paths: Vec<PathBuf>,
    pub total_lines: usize,
    pub partial_state: Option<PartialEditState>,
    pub suggestions: Vec<String>,
}
```

### 4. Add error suggestion collection

When edits fail, collect actionable suggestions:

```rust
fn generate_suggestions(
    failed_edits: &[FailedEdit],
    edit_count: usize,
) -> Vec<String> {
    let mut suggestions = Vec::new();
    
    // Check for whitespace issues
    if failed_edits.iter().any(|e| e.reason.contains("whitespace")) {
        suggestions.push("Check whitespace: file may use different indentation".to_string());
    }
    
    // Check for too many edits
    if edit_count > 10 {
        suggestions.push(format!(
            "Consider replace mode: this job has {}+ edits, replace is safer",
            edit_count
        ));
    }
    
    // Add line hints
    for edit in failed_edits {
        if let Some(line) = edit.suggested_line {
            suggestions.push(format!(
                "For '{}...': check line {} in {}",
                &edit.find_preview[..20.min(edit.find_preview.len())],
                line,
                edit.file_path
            ));
        }
    }
    
    suggestions
}
```

### 5. Update function signature

Change the return type of `process_edit_mode`:

```rust
pub(crate) async fn process_edit_mode(
    ollama: &OllamaClient,
    project_root: &Path,
    config: &Config,
    job: &Job,
    context_files: &[(PathBuf, String)],
    edit_prompt: &str,
    dry_run: bool,  // New parameter
) -> Result<EditModeResult, WorkSplitError>
```

### 6. Add display helpers for dry-run output

```rust
impl std::fmt::Display for DryRunResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[DRY RUN] Job: {}", self.job_id)?;
        // Group by file
        // Show each edit with status indicator
        Ok(())
    }
}
```

## Constraints

- Preserve backward compatibility with callers in runner/mod.rs
- Handle EditApplyError from the updated parser_edit module
- Import PartialEditState from models::status
- Write successful edits to disk even if some fail (partial completion)

## Formatting Notes

- Uses 4-space indentation
- Use tracing macros for logging
- Follow async patterns from existing code

## Dependencies

This job depends on:
- enhance_001_models.md (for PartialEditState)
- enhance_002_parser_fuzzy.md (for EditApplyError)
