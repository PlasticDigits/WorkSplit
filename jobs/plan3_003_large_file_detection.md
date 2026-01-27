---
mode: edit
context_files:
  - src/core/jobs.rs
  - src/core/runner/mod.rs
target_files:
  - src/error.rs
  - src/core/jobs.rs
  - src/core/runner/mod.rs
output_dir: src/
output_file: error.rs
---

# Plan3 Task 1: Large File Detection

Fail early if any file exceeds the LOC limit (900 lines).

## Requirements

1. Update the `FileTooLarge` error message to include a clearer action hint.

## Edit Instructions

FILE: src/error.rs
FIND:
    #[error("File too large: {path} has {lines} lines (max: {limit})\n\n{suggestion}")]
REPLACE:
    #[error("File too large: {path} has {lines} lines (max: {limit})\n\nManager action required:\n{suggestion}")]
END
