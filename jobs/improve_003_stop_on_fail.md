---
context_files:
  - src/main.rs
  - src/commands/run.rs
output_dir: src/
output_file: main.rs
---

# Add --stop-on-fail CLI Flag

## Overview
Add a `--stop-on-fail` flag to the `run` command that halts job processing when any job fails verification. This is useful for CI/CD pipelines or when jobs have dependencies.

## Problem
Currently, `worksplit run` processes all pending jobs even if some fail. When jobs depend on previous jobs' outputs, continuing after a failure wastes time and produces incorrect results.

## Solution
Add a `--stop-on-fail` flag that:
1. Stops processing immediately when a job fails
2. Reports which job failed and exits with non-zero status
3. Works with both `run_all()` and single job runs

## Requirements

### 1. Update CLI in main.rs
Add to the `Run` variant:
```rust
/// Stop processing when any job fails
#[arg(long)]
stop_on_fail: bool,
```

### 2. Update `RunOptions` in commands/run.rs
Add field:
```rust
pub stop_on_fail: bool,
```

And update Default impl.

### 3. Pass flag through main.rs
Update the `Commands::Run` match arm to include `stop_on_fail` in the `RunOptions` struct construction.

### 4. Update runner to support stop-on-fail
The `run_all()` method signature should be updated to:
```rust
pub async fn run_all(&mut self, resume_stuck: bool, stop_on_fail: bool) -> Result<RunSummary, WorkSplitError>
```

When `stop_on_fail` is true and a job fails:
- Set the summary's failed count
- Add the failed result to results
- Return early instead of continuing to the next job
- The remaining jobs stay in their current status (not processed)

### 5. Update run_jobs function
Pass the `stop_on_fail` flag to `runner.run_all()`.

### 6. Exit code
When `--stop-on-fail` is used and a job fails, the command should exit with code 1.

## Implementation Notes
- Single job runs (`--job`) should also respect `--stop-on-fail` (though it's implicit since only one job runs)
- Print a clear message when stopping due to failure
- The summary should show how many were processed before stopping

## Expected Behavior
```bash
# Without flag - all jobs run
$ worksplit run
# Processing 3 jobs...
# job_001 [PASS]
# job_002 [FAIL]: Missing function
# job_003 [PASS]
# Summary: 2 passed, 1 failed

# With flag - stops at first failure
$ worksplit run --stop-on-fail
# Processing 3 jobs...
# job_001 [PASS]
# job_002 [FAIL]: Missing function
# Stopping due to failure (--stop-on-fail)
# Summary: 1 passed, 1 failed, 1 not processed
$ echo $?
1
```

## Files to Modify
This job modifies `src/main.rs`. Note that `src/commands/run.rs` and `src/core/runner.rs` will also need updates - those should be handled by the code generator recognizing the necessary changes based on main.rs updates.
