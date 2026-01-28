---
context_files:
  - src/models/config.rs
output_dir: src/commands/
output_file: cleanup.rs
---

# Create Cleanup Command

Create a command that deletes archived jobs from `jobs/archive/` that are older than X days.

## Module Structure

```rust
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use chrono::{DateTime, Duration, Utc};
use tracing::info;

use crate::error::WorkSplitError;
use crate::models::Config;
```

## CleanupResult Struct

```rust
/// Result of cleanup operation
#[derive(Debug)]
pub struct CleanupResult {
    pub deleted_count: usize,
    pub deleted_jobs: Vec<String>,
}
```

## Helper Function: get_file_modified_time

```rust
fn get_file_modified_time(path: &Path) -> Result<DateTime<Utc>, std::io::Error> {
    let metadata = fs::metadata(path)?;
    let modified = metadata.modified()?;
    let duration = modified.duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    Ok(DateTime::from_timestamp(duration.as_secs() as i64, 0)
        .unwrap_or_else(Utc::now))
}
```

## Main Function: cleanup_archived_jobs

```rust
pub fn cleanup_archived_jobs(
    project_root: &PathBuf,
    days: Option<u32>,
    dry_run: bool,
) -> Result<CleanupResult, WorkSplitError>
```

### Logic

1. Load config with `Config::load_from_dir(project_root)?`
2. Create path: `archive_dir = project_root.join("jobs").join("archive")`
3. If `!archive_dir.exists()`, return early with count 0
4. Get threshold: `days.unwrap_or(config.cleanup.days)`
5. Calculate cutoff: `Utc::now() - Duration::days(threshold_days as i64)`
6. Read dir: `fs::read_dir(&archive_dir)?`
7. For each entry:
   - Get path, skip if not `.md` extension
   - Get modified time with helper function, skip on error
   - If `modified < cutoff`:
     - Get file_name and job_id (trim .md suffix)
     - Calculate days_old
     - If dry_run: print "Would delete: {} ({} days old)"
     - Else:
       - Delete: `fs::remove_file(&path)?`
       - Print "Deleted: {} ({} days old)"
     - Push to deleted_jobs
8. Return CleanupResult

## Helper Function: run_auto_cleanup

```rust
pub fn run_auto_cleanup(project_root: &PathBuf) -> Result<(), WorkSplitError>
```

### Logic

1. Load config
2. If `!config.cleanup.enabled`, return Ok(())
3. Call `cleanup_archived_jobs(project_root, None, false)?`
4. If `result.deleted_count > 0`, log with info!
5. Return Ok(())

## Error Handling

- Use `?` operator - WorkSplitError has From impl for std::io::Error
- Skip files that can't be read instead of failing

## Console Output Format

```
Deleted: job_name.md (45 days old)
```
