---
context_files:
  - src/core/runner/mod.rs
  - src/core/parser/extract.rs
output_dir: src/core/runner/
output_file: mod.rs
---

# Retry with Feedback on Verification Failure

## Overview
When verification fails, retry the creation phase once more, this time including the verification error message as additional context. This gives the LLM a chance to fix its mistakes without requiring manual intervention.

## Problem
Currently, when verification fails, the job is marked as failed immediately. The user must manually reset and retry. Often the verification error contains actionable feedback that, if provided to the LLM, would result in better output.

## Solution
Add a retry mechanism in the `run_job` method:
1. After verification fails, retry creation once with the error message included
2. If the retry also fails verification, mark the job as failed with the final error
3. Track that a retry occurred for logging/reporting

## Requirements

### 1. Add `assemble_retry_prompt()` function to parser.rs (add to parser.rs exports in mod.rs)
This function should be exported from `src/core/parser.rs` and assembled in runner.rs:

```rust
pub fn assemble_retry_prompt(
    system_prompt: &str,
    context_files: &[(std::path::PathBuf, String)],
    instructions: &str,
    output_path: &str,
    previous_output: &str,
    verification_error: &str,
) -> String
```

The prompt should include:
- [SYSTEM] section with system prompt
- [CONTEXT] section with context files
- [PREVIOUS ATTEMPT] section showing the code that failed
- [VERIFICATION FEEDBACK] section with the error message
- [INSTRUCTIONS] section with original instructions
- Clear instruction to fix the issues mentioned in feedback

### 2. Update `run_job()` in runner.rs
- After verification fails, check if this is the first attempt
- If first attempt, call `assemble_retry_prompt()` with the error message
- Send retry prompt to Ollama
- Extract code, write to file, verify again
- If retry succeeds, mark as pass
- If retry fails, mark as fail with the retry error
- Log that a retry occurred

### 3. Update `JobResult` struct
Add a field to track retry:
```rust
pub retry_attempted: bool,
```

### 4. Logging
- Info level: "Verification failed, retrying with feedback..."
- Info level: "Retry successful" or "Retry also failed"
- Debug level: The feedback message being included

## Implementation Notes
- Only retry ONCE to avoid infinite loops
- The retry should use the same timeout and streaming settings
- Keep existing error handling intact
- If Ollama fails during retry, treat as job failure (don't retry the retry)

## Test Cases to Add
- Test `assemble_retry_prompt()` output format
- Unit tests for the new functionality
