---
context_files:
  - src/models/config.rs
  - src/core/status.rs
output_dir: src/commands/
output_file: archive.rs
---

# Create Archive Command

Create a new command module that moves completed (Pass status) jobs older than X days to `jobs/archive/`.

## Module Structure

```rust
use std::fs;
use std::path::PathBuf;
use chrono::{Duration, Utc};
use tracing::info;

use crate::error::WorkSplitError;
use crate::models::Config;
use crate::core::status::StatusManager;
use crate::models::JobStatus;
use crate::commands::cleanup::run_auto_cleanup;
```

## ArchiveResult Struct

```rust
/// Result of archive operation
#[derive(Debug)]
pub struct ArchiveResult {
    pub archived_count: usize,
    pub archived_jobs: Vec<String>,
}
```

## Main Function: archive_jobs

```rust
pub fn archive_jobs(
    project_root: &PathBuf,
    days: Option<u32>,
    dry_run: bool,
) -> Result<ArchiveResult, WorkSplitError>
```

### Logic

1. Load config with `Config::load_from_dir(project_root)?`
2. Create paths: `jobs_dir = project_root.join("jobs")`, `archive_dir = jobs_dir.join("archive")`
3. Load status with `StatusManager::new(&jobs_dir)?`
4. Get threshold: `days.unwrap_or(config.archive.days)`
5. Calculate cutoff: `Utc::now() - Duration::days(threshold_days as i64)`
6. Iterate `status_manager.all_entries()`:
   - If `entry.status == JobStatus::Pass && entry.updated_at < cutoff`:
     - Check if job file exists: `jobs_dir.join(format!("{}.md", entry.id))`
     - If not exists, skip with continue
     - Calculate days_old: `(Utc::now() - entry.updated_at).num_days()`
     - If dry_run: print "Would archive: {} ({} days old)"
     - Else:
       - Create archive_dir if needed: `fs::create_dir_all(&archive_dir)?`
       - Move file: `fs::rename(&job_file, &archive_file)?`
       - Print "Archived: {} ({} days old)"
     - Push entry.id to archived_jobs
7. Return ArchiveResult

## Helper Function: run_auto_archive

```rust
pub fn run_auto_archive(project_root: &PathBuf) -> Result<(), WorkSplitError>
```

### Logic

1. Load config
2. If `!config.archive.enabled`, return Ok(())
3. Call `archive_jobs(project_root, None, false)?`
4. If `result.archived_count > 0`, log with info!
5. Call `run_auto_cleanup(project_root)?` to trigger cleanup after archive
6. Return Ok(())

## Error Handling

- Use `?` operator with WorkSplitError - it has From impls for std::io::Error and ConfigError
- Don't wrap errors in format! strings

## Console Output Format

```
Archived: job_name (3 days old)
```
