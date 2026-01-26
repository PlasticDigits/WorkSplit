---
context_files:
  - src/models/status.rs
output_dir: src/models/
output_file: status.rs
---

# Add TDD Status Variants to JobStatus Enum

## Overview
Extend the `JobStatus` enum to support TDD workflow by adding two new status variants: `PendingTest` and `PendingTestRun`.

## Requirements

### New Status Variants

Add these variants to `JobStatus`:

1. **`PendingTest`** - Job has been sent to Ollama for test generation (TDD first step)
2. **`PendingTestRun`** - Tests and code generated, waiting for test execution

### Updated Status Flow

The new workflow supports two paths:

**Standard workflow** (no test_file specified):
```
Created → PendingWork → PendingVerification → Pass/Fail
```

**TDD workflow** (test_file specified):
```
Created → PendingTest → PendingWork → PendingVerification → PendingTestRun → Pass/Fail
```

### Method Updates

Update the following methods to handle new variants:

1. **`is_complete()`** - Should still only return `true` for `Pass` and `Fail`

2. **`is_stuck()`** - Should return `true` for ALL intermediate states:
   - `PendingTest`
   - `PendingWork`
   - `PendingVerification`
   - `PendingTestRun`

3. **`is_ready()`** - Should still only return `true` for `Created`

### New Helper Methods

Add these new methods to `JobStatus`:

1. **`is_tdd_phase(&self) -> bool`** - Returns `true` for `PendingTest` and `PendingTestRun`

2. **`next_status(&self, tdd_enabled: bool) -> Option<JobStatus>`** - Returns the next status in the workflow:
   - If `tdd_enabled`:
     - `Created` → `Some(PendingTest)`
     - `PendingTest` → `Some(PendingWork)`
     - `PendingWork` → `Some(PendingVerification)`
     - `PendingVerification` → `Some(PendingTestRun)`
     - `PendingTestRun` → `None` (terminal, actual status set to Pass/Fail)
   - If NOT `tdd_enabled`:
     - `Created` → `Some(PendingWork)`
     - `PendingWork` → `Some(PendingVerification)`
     - `PendingVerification` → `None` (terminal)
   - `Pass`/`Fail` → `None`

### Serialization

Ensure new variants serialize to snake_case:
- `PendingTest` → `"pending_test"`
- `PendingTestRun` → `"pending_test_run"`

### Test Updates

Update existing tests and add new tests:

1. **`test_job_status_is_stuck`** - Include new stuck states
2. **`test_job_status_is_tdd_phase`** - Test new helper method
3. **`test_job_status_next_status_tdd`** - Test TDD workflow transitions
4. **`test_job_status_next_status_standard`** - Test standard workflow transitions
5. **`test_job_status_serialization`** - Include new variants

## Implementation Notes

- Maintain backward compatibility - existing status files with only the original 5 statuses should still work
- The order of variants in the enum should reflect the workflow order
- All existing functionality must continue to work unchanged
