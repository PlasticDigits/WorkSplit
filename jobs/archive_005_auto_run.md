---
mode: edit
target_files:
  - src/commands/run.rs
context_files:
  - src/commands/archive.rs
output_dir: src/commands/
output_file: run.rs
---

# Integrate Auto-Archive and Auto-Cleanup into Run Command

Modify the run command to automatically trigger archive (and subsequently cleanup) after each successful run.

## Changes to src/commands/run.rs

### Add import

At the top of the file, add:
```rust
use crate::commands::archive::run_auto_archive;
```

### Add auto-archive call at end of run_jobs

Find the end of the `run_jobs` function (just before the final `Ok(())`). Add this code:

```rust
// Run auto-archive after jobs complete (which triggers auto-cleanup)
if let Err(e) = run_auto_archive(&project_root) {
    // Don't fail the run for archive errors, just log
    tracing::warn!("Auto-archive failed: {}", e);
}
```

This should be placed AFTER all job processing is complete, but before returning Ok(()).

### Important placement notes

The auto-archive call should happen:
1. After all jobs have been processed (pass or fail)
2. Before the function returns
3. Not during dry_run mode

Add a condition to skip auto-archive during dry_run:

```rust
// Run auto-archive after jobs complete (which triggers auto-cleanup)
if !options.dry_run {
    if let Err(e) = run_auto_archive(&project_root) {
        // Don't fail the run for archive errors, just log
        tracing::warn!("Auto-archive failed: {}", e);
    }
}
```

## Expected behavior

After running `worksplit run`:
1. Jobs execute normally
2. If archive is enabled (default: true) and jobs completed:
   - Archive moves Pass jobs older than 3 days to `jobs/archive/`
3. If cleanup is enabled (default: true) and archive ran:
   - Cleanup deletes archived jobs older than 30 days

This happens silently unless there are jobs to archive/cleanup, in which case it logs:
```
Archived 2 job(s) to jobs/archive/
Cleaned up 1 archived job(s)
```
