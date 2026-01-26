---
context_files:
  - src/core/jobs.rs
output_dir: src/core/
output_file: jobs.rs
---

# Extend JobsManager with Test Prompt Loading

## Overview
Extend the `JobsManager` to support loading the test generation system prompt (`_systemprompt_test.md`).

## Requirements

### New Constant

Add a constant for the test prompt filename:

```rust
const TEST_PROMPT_FILE: &str = "_systemprompt_test.md";
```

### New Method

Add this method to `JobsManager`:

**`load_test_prompt(&self) -> Result<String, WorkSplitError>`**
- Loads `_systemprompt_test.md` from the jobs directory
- Returns the file contents as a String
- Returns error if file doesn't exist or can't be read
- Uses the same pattern as `load_create_prompt()` and `load_verify_prompt()`

### Optional Test Prompt Loading

Add an alternative method that doesn't fail if the file is missing:

**`load_test_prompt_optional(&self) -> Result<Option<String>, WorkSplitError>`**
- Returns `Ok(Some(content))` if file exists and can be read
- Returns `Ok(None)` if file doesn't exist
- Returns `Err` only for actual read errors (permissions, etc.)

This is useful for backward compatibility - projects without TDD support won't have this file.

### Implementation Pattern

Follow the existing pattern from `load_create_prompt`:

```rust
pub fn load_create_prompt(&self) -> Result<String, WorkSplitError> {
    let path = self.jobs_dir.join(CREATE_PROMPT_FILE);
    fs::read_to_string(&path).map_err(|e| WorkSplitError::Io {
        path: path.clone(),
        source: e,
    })
}
```

### Test Prompt Purpose

The test prompt will be used to instruct the LLM to generate tests BEFORE generating the implementation code. This is the TDD approach:

1. Read job instructions
2. Generate tests that verify the requirements
3. Generate implementation that passes the tests
4. Verify implementation against original requirements

### Test Updates

Add new tests:

1. **`test_load_test_prompt`** - Test loading test prompt when file exists
2. **`test_load_test_prompt_missing`** - Test error when file doesn't exist
3. **`test_load_test_prompt_optional_exists`** - Test optional loading when file exists
4. **`test_load_test_prompt_optional_missing`** - Test optional loading when file doesn't exist (should return None, not error)

### Existing Methods to Keep

Ensure all existing methods remain unchanged:
- `new()`
- `jobs_dir()`
- `discover_jobs()`
- `parse_job()`
- `load_context_files()`
- `load_create_prompt()`
- `load_verify_prompt()`
- `check_token_budget()`

## Implementation Notes

- The test prompt file is optional for backward compatibility
- Use `Path::exists()` check in the optional variant
- Keep error handling consistent with existing methods
