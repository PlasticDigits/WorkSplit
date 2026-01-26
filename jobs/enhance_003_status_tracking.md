---
context_files:
  - src/core/status.rs
  - src/models/status.rs
output_dir: src/core/
output_file: status.rs
---

# Add Partial Completion Tracking and Continue State

Enhance the StatusManager to track partial edit completions and support the --continue flag.

## Requirements

### 1. Add method to set partial completion state

Add to `StatusManager`:

```rust
/// Set a job as partially completed with edit state
pub fn set_partial(&mut self, job_id: &str, state: PartialEditState) -> Result<(), StatusError> {
    let entry = self.entries.get_mut(job_id)
        .ok_or_else(|| StatusError::JobNotFound(job_id.to_string()))?;
    entry.set_partial(state);
    self.save()
}
```

### 2. Add method to get failed edits for continue

Add to `StatusManager`:

```rust
/// Get the failed edits for a partial job (for --continue)
pub fn get_failed_edits(&self, job_id: &str) -> Option<Vec<FailedEdit>> {
    self.entries.get(job_id)
        .and_then(|e| e.partial_state.as_ref())
        .map(|s| s.failed_edits.clone())
}
```

### 3. Add method to clear partial state after successful continue

```rust
/// Clear partial state after successful retry
pub fn clear_partial_state(&mut self, job_id: &str) -> Result<(), StatusError> {
    if let Some(entry) = self.entries.get_mut(job_id) {
        entry.partial_state = None;
    }
    self.save()
}
```

### 4. Update get_summary to count partial jobs

Update `StatusSummary` to include a `partial` field and update `get_summary()` to count Partial status jobs.

```rust
pub struct StatusSummary {
    // ... existing fields ...
    pub partial: usize,
}
```

Update the `Display` impl to show partial count.

### 5. Add method to get partial jobs

```rust
/// Get all jobs with partial completion status
pub fn get_partial_jobs(&self) -> Vec<&JobStatusEntry> {
    self.entries
        .values()
        .filter(|e| e.status == JobStatus::Partial)
        .collect()
}
```

### 6. Add tests

Add tests:
- `test_set_partial_status`
- `test_get_failed_edits`
- `test_clear_partial_state`
- `test_get_summary_with_partial`
- `test_partial_job_persistence`

## Constraints

- Import `PartialEditState` and `FailedEdit` from models::status
- Preserve all existing functionality
- Partial jobs should show in `is_stuck()` but not `is_complete()`
- Partial status persists across restarts (saved to _jobstatus.json)

## Formatting Notes

- Uses 4-space indentation
- Follow existing code patterns in status.rs
- Use `tracing::info!` for logging partial state changes

## Dependencies

This job depends on enhance_001_models.md being completed first (for PartialEditState types).
