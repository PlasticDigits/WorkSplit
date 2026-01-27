---
context_files:
  - src/core/runner/mod.rs
  - src/models/status.rs
output_dir: src/core/
output_file: runner.rs
---

# Modify Runner to Handle Test Generation Phase

## Overview
Extend the `Runner` to support TDD workflow by adding a test generation phase before the creation phase.

## Requirements

### Updated Workflow

The runner should now support two workflows based on job configuration:

**Standard workflow** (job.metadata.test_file is None):
```
Created → PendingWork → [generate code] → PendingVerification → [verify] → Pass/Fail
```

**TDD workflow** (job.metadata.test_file is Some):
```
Created → PendingTest → [generate tests] → PendingWork → [generate code] → PendingVerification → [verify] → PendingTestRun → Pass/Fail
```

Note: Actual test execution in `PendingTestRun` is NOT implemented yet - we just transition to Pass for now.

### Import the New Parser Function

Import `assemble_test_prompt` from the parser module:

```rust
use crate::core::{
    assemble_creation_prompt, assemble_test_prompt, assemble_verification_prompt,
    // ... other imports
};
```

### Changes to run_job Method

Modify `run_job` to:

1. Check if `job.metadata.is_tdd_enabled()`
2. If TDD enabled:
   a. Load test prompt (fail if missing - TDD jobs require test prompt)
   b. Set status to `PendingTest`
   c. Generate tests using Ollama
   d. Write tests to `job.metadata.test_path()`
   e. Continue to `PendingWork` phase (existing creation logic)
3. After verification:
   a. If TDD enabled, set status to `PendingTestRun`
   b. For now, immediately transition to `Pass` (test execution not implemented)
   c. If NOT TDD enabled, use existing verification result directly

### Changes to run_all Method

The `run_all` method should:
- Load test prompt once at the start (optional - may not exist)
- Pass it to `run_job` 
- Handle case where TDD job exists but test prompt is missing (error)

### Updated Method Signature

```rust
async fn run_job(
    &mut self,
    job_id: &str,
    create_prompt: &str,
    verify_prompt: &str,
    test_prompt: Option<&str>,  // NEW - optional test prompt
) -> Result<JobResult, WorkSplitError>
```

### Updated JobResult

Add fields to track test file output:

```rust
pub struct JobResult {
    pub job_id: String,
    pub status: JobStatus,
    pub error: Option<String>,
    pub output_path: Option<PathBuf>,
    pub output_lines: Option<usize>,
    pub test_path: Option<PathBuf>,      // NEW
    pub test_lines: Option<usize>,       // NEW
}
```

### Error Handling

Add appropriate error cases:
- TDD job without test prompt file → Error with helpful message
- Test file write failure → Error
- Test generation timeout → Error (same as creation timeout)

### Logging

Add info/debug logging for:
- "TDD workflow enabled for job '{job_id}'"
- "Generating tests to {test_path}..."
- "Generated {lines} lines of test code"
- "Test execution skipped (not implemented)"

### Test Phase Implementation

Insert this section BEFORE the creation phase in `run_job`:

```rust
// === TEST GENERATION PHASE (TDD) ===
let mut test_result_path: Option<PathBuf> = None;
let mut test_result_lines: Option<usize> = None;

if job.metadata.is_tdd_enabled() {
    let test_prompt_str = test_prompt.ok_or_else(|| WorkSplitError::Config(
        "TDD job requires _systemprompt_test.md but it was not found".to_string()
    ))?;

    info!("TDD workflow enabled for job '{}'", job_id);
    
    // Update status to pending test
    self.status_manager.update_status(job_id, JobStatus::PendingTest)?;

    let test_path = job.metadata.test_path().unwrap();
    let test_path_str = test_path.display().to_string();

    let test_gen_prompt = assemble_test_prompt(
        test_prompt_str,
        &context_files,
        &job.instructions,
        &test_path_str,
    );

    info!("Generating tests to {}...", test_path_str);
    let test_response = self
        .ollama
        .generate(&test_gen_prompt, self.config.behavior.stream_output)
        .await
        .map_err(|e| {
            let _ = self.status_manager.set_failed(job_id, e.to_string());
            WorkSplitError::Ollama(e)
        })?;

    // Extract and write test code
    let test_code = extract_code(&test_response);
    let test_line_count = count_lines(&test_code);

    info!("Generated {} lines of test code", test_line_count);

    let full_test_path = self.project_root.join(&test_path);
    if let Some(parent) = full_test_path.parent() {
        if !parent.exists() && self.config.behavior.create_output_dirs {
            fs::create_dir_all(parent)?;
        }
    }

    fs::write(&full_test_path, &test_code)?;
    info!("Wrote tests to: {}", full_test_path.display());

    test_result_path = Some(full_test_path);
    test_result_lines = Some(test_line_count);
}
```

### Post-Verification TDD Handling

After the verification phase, add:

```rust
// For TDD jobs, transition through PendingTestRun
if job.metadata.is_tdd_enabled() && status == JobStatus::Pass {
    self.status_manager.update_status(job_id, JobStatus::PendingTestRun)?;
    info!("Test execution skipped (not implemented yet)");
    // For now, keep the Pass status from verification
}
```

### Updated JobResult Return

```rust
Ok(JobResult {
    job_id: job_id.to_string(),
    status,
    error,
    output_path: Some(full_output_path),
    output_lines: Some(line_count),
    test_path: test_result_path,
    test_lines: test_result_lines,
})
```

## Implementation Notes

- Keep the existing standard workflow unchanged when test_file is None
- The test generation uses streaming just like code generation
- Test prompt is loaded once per run_all, not per job
- For run_single, load test prompt only if the job needs it
- The `PendingTestRun` → `Pass` transition is a placeholder until test execution is implemented
