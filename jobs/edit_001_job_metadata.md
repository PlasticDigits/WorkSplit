---
context_files:
  - src/models/job.rs
output_dir: src/models/
output_file: job.rs
---

# Add Edit Mode Support to JobMetadata

## Overview
Add support for `mode: edit` in job frontmatter, enabling surgical file edits rather than full file replacement. This is the foundation for the edit instructions feature.

## Problem
Currently, WorkSplit only supports full file replacement. For small changes (adding a field, modifying a function), regenerating entire files is token-inefficient and error-prone.

## Solution
Add two new fields to `JobMetadata`:
1. `mode` - Either "replace" (default) or "edit"
2. `target_files` - List of files to edit (used when mode is "edit")

## Requirements

### 1. Add Mode Enum
Create a new enum for the output mode:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum OutputMode {
    #[default]
    Replace,
    Edit,
}
```

### 2. Update JobMetadata
Add new fields:

```rust
/// Output mode: "replace" (default) generates full files, "edit" applies surgical changes
#[serde(default)]
pub mode: OutputMode,

/// Target files for edit mode (files to apply edits to)
#[serde(default, skip_serializing_if = "Option::is_none")]
pub target_files: Option<Vec<PathBuf>>,
```

### 3. Add Helper Methods
Add methods to `JobMetadata`:

```rust
/// Check if this job uses edit mode
pub fn is_edit_mode(&self) -> bool {
    self.mode == OutputMode::Edit
}

/// Get target files for edit mode
/// Returns target_files if set, otherwise returns output_path as single-item vec
pub fn get_target_files(&self) -> Vec<PathBuf> {
    if let Some(ref files) = self.target_files {
        files.clone()
    } else {
        vec![self.output_path()]
    }
}
```

### 4. Update Validation
Add validation in `validate()`:
- If `mode` is `Edit`, `target_files` should be specified (warning if not)
- `target_files` entries should not be empty paths
- Edit mode is incompatible with sequential mode

```rust
// In validate():
if self.mode == OutputMode::Edit {
    if let Some(ref files) = self.target_files {
        if files.is_empty() {
            return Err(JobValidationError::EmptyTargetFiles);
        }
        for file in files {
            if file.as_os_str().is_empty() {
                return Err(JobValidationError::EmptyTargetFilePath);
            }
        }
    }
    // Edit mode is incompatible with sequential mode
    if self.is_sequential() {
        return Err(JobValidationError::EditModeWithSequential);
    }
}
```

### 5. Add New Validation Errors
Add to `JobValidationError`:
```rust
#[error("target_files list cannot be empty in edit mode")]
EmptyTargetFiles,
#[error("target_files contains an empty path")]
EmptyTargetFilePath,
#[error("edit mode cannot be combined with sequential mode")]
EditModeWithSequential,
```

### 6. Add Tests
Add tests for:
- `is_edit_mode()` returns correct value
- `get_target_files()` returns correct files
- Validation catches empty target_files in edit mode
- Validation catches edit + sequential conflict
- Serialization/deserialization works correctly

## Expected Usage

```yaml
---
mode: edit
target_files:
  - src/main.rs
  - src/commands/run.rs
output_dir: src/
output_file: main.rs  # Fallback, not used in edit mode
---

# Add New CLI Flag

Add the following edits...
```

## Implementation Notes
- Keep full backward compatibility - default mode is "replace"
- The `output_dir` and `output_file` fields remain for backward compatibility
- Edit mode will use target_files exclusively when specified
