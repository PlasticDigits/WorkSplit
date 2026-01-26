---
context_files:
  - src/models/job.rs
output_dir: src/models/
output_file: job.rs
---

# Extend JobMetadata with TDD Support

## Overview
Extend the `JobMetadata` struct to support TDD workflow by adding an optional `test_file` field.

## Requirements

### New Field in JobMetadata

Add an optional field to `JobMetadata`:

```rust
/// Optional test file for TDD workflow (generated before implementation)
#[serde(default)]
pub test_file: Option<String>,
```

### Updated JobMetadata Struct

The struct should now have these fields:
- `context_files: Vec<PathBuf>` (existing)
- `output_dir: PathBuf` (existing)
- `output_file: String` (existing)
- `test_file: Option<String>` (NEW) - Optional filename for generated tests

### New Methods

Add these methods to `JobMetadata`:

1. **`is_tdd_enabled(&self) -> bool`**
   - Returns `true` if `test_file` is `Some(_)`
   - Returns `false` if `test_file` is `None`

2. **`test_path(&self) -> Option<PathBuf>`**
   - Returns `Some(output_dir.join(test_file))` if test_file is set
   - Returns `None` if test_file is not set

### Validation Updates

Update `validate()` to add test file validation:
- If `test_file` is `Some(name)` and `name.is_empty()`, return error
- Add new error variant: `JobValidationError::EmptyTestFile`

### Job Struct

The `Job` struct should remain unchanged - it already has `metadata: JobMetadata` which will automatically include the new field.

### New Error Variant

Add to `JobValidationError`:

```rust
#[error("Test file name cannot be empty")]
EmptyTestFile,
```

### Example Job Frontmatter

**Standard job (no TDD):**
```yaml
---
context_files:
  - src/models/user.rs
output_dir: src/services/
output_file: user_service.rs
---
```

**TDD job:**
```yaml
---
context_files:
  - src/models/user.rs
output_dir: src/services/
output_file: user_service.rs
test_file: user_service_test.rs
---
```

### Test Updates

Add new tests:

1. **`test_job_metadata_tdd_enabled`** - Test `is_tdd_enabled()` for both cases
2. **`test_job_metadata_test_path`** - Test `test_path()` returns correct path
3. **`test_job_metadata_validate_empty_test_file`** - Validate empty test_file error
4. **`test_job_metadata_default_no_test_file`** - Ensure backward compatibility (test_file defaults to None)

### Backward Compatibility

- Existing job files without `test_file` should continue to work
- `#[serde(default)]` ensures missing field deserializes to `None`
- All existing tests should pass without modification

## Implementation Notes

- Use `#[serde(skip_serializing_if = "Option::is_none")]` to avoid serializing None values
- The test_file is just a filename (not a path) - it will be joined with output_dir
- This mirrors how output_file works
