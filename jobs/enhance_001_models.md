---
context_files:
  - src/models/status.rs
  - src/models/job.rs
output_dir: src/models/
output_file: status.rs
---

# Add Model Types for Enhanced Edit Mode Features

This job adds the foundational types needed for partial completion tracking and new edit modes.

## Requirements

### 1. Add `Partial` status variant to JobStatus enum (src/models/status.rs)

Add a new `Partial` variant to the `JobStatus` enum to track jobs that partially completed:

```rust
/// Verification partially passed (some edits succeeded, some failed)
Partial,
```

Update the `is_complete()` method to return `false` for Partial status (it needs recovery).

Update the `is_stuck()` method to return `true` for Partial status.

Add new method `is_partial()` that returns true only for Partial status.

### 2. Add PartialEditState struct to JobStatusEntry (src/models/status.rs)

Add a new struct to track partial edit state:

```rust
/// State for partially completed edit jobs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialEditState {
    /// Edits that were successfully applied
    pub successful_edits: Vec<SuccessfulEdit>,
    /// Edits that failed to apply
    pub failed_edits: Vec<FailedEdit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessfulEdit {
    pub file_path: String,
    pub find_preview: String,  // First 50 chars of FIND text
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedEdit {
    pub file_path: String,
    pub find_preview: String,  // First 50 chars of FIND text
    pub reason: String,
    pub suggested_line: Option<usize>,  // Line number hint from fuzzy match
}
```

Add an optional `partial_state` field to `JobStatusEntry`:

```rust
#[serde(skip_serializing_if = "Option::is_none")]
pub partial_state: Option<PartialEditState>,
```

Add methods to `JobStatusEntry`:
- `set_partial(state: PartialEditState)` - Set status to Partial with state
- `get_partial_state() -> Option<&PartialEditState>` - Get partial state if any

### 3. Add new mode variants to OutputMode enum (src/models/job.rs)

Add two new variants to the `OutputMode` enum:

```rust
/// Batch text replacements using AFTER/INSERT pattern
ReplacePattern,
/// Update struct literals in test fixtures
UpdateFixtures,
```

### 4. Add new fields to JobMetadata for new modes (src/models/job.rs)

Add these optional fields to `JobMetadata`:

```rust
/// Struct name for update_fixtures mode
#[serde(default, skip_serializing_if = "Option::is_none")]
pub struct_name: Option<String>,

/// New field to add for update_fixtures mode (e.g., "verify: true")
#[serde(default, skip_serializing_if = "Option::is_none")]
pub new_field: Option<String>,
```

### 5. Add validation for new modes in JobMetadata::validate()

Add validation rules:
- ReplacePattern mode requires target_files
- UpdateFixtures mode requires target_files, struct_name, and new_field

## Constraints

- Preserve all existing functionality
- Update tests to include the new variants
- Use `#[serde(rename_all = "snake_case")]` for consistent serialization
- Initialize `partial_state` to None in JobStatusEntry::new()

## Formatting Notes

- Uses 4-space indentation
- Derive macros in order: Debug, Clone, Serialize, Deserialize (or Copy where applicable)
- Skip serializing None values

## Output Files

Generate both files:
- `src/models/status.rs` - With Partial status and PartialEditState
- `src/models/job.rs` - With new OutputMode variants and JobMetadata fields

Use `~~~worksplit:path` delimiters for each file.
