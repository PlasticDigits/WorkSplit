---
mode: edit
context_files:
  - src/core/mod.rs
  - src/commands/run.rs
target_files:
  - src/error.rs
  - src/core/mod.rs
  - src/commands/run.rs
output_dir: src/
output_file: error.rs
---

# Plan3 Task 4: Integrate Dependency Sorting

Register dependency module and use it in the run command.

## Requirements

1. Make the cyclic dependency error message more actionable.

## Edit Instructions

FILE: src/error.rs
FIND:
    #[error("Cyclic dependency detected in job files")]
REPLACE:
    #[error("Cyclic dependency detected in job files. Check depends_on for cycles.")]
END
