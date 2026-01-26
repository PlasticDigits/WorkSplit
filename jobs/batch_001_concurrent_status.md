---
mode: edit
context_files:
  - src/core/status.rs
target_files:
  - src/core/status.rs
output_dir: src/core/
output_file: status.rs
---

# Make StatusManager Thread-Safe for Parallel Execution

## Goal

Wrap the `StatusManager` in thread-safe primitives to allow concurrent status updates from multiple parallel jobs.

## Current Implementation

The `StatusManager` in `src/core/status.rs` uses:
- `HashMap<String, JobStatusEntry>` for in-memory entries
- Direct file I/O with atomic rename for persistence
- Mutable `&mut self` methods for updates

## Required Changes

### 1. Add Thread-Safe Wrapper Type

Create a new type alias and wrapper for thread-safe access:

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

/// Thread-safe wrapper for StatusManager
pub type SharedStatusManager = Arc<RwLock<StatusManager>>;
```

### 2. Add Factory Method

Add a method to create a shared instance:

```rust
impl StatusManager {
    /// Create a new thread-safe shared status manager
    pub fn new_shared(jobs_dir: &Path) -> Result<SharedStatusManager, StatusError> {
        let manager = Self::new(jobs_dir)?;
        Ok(Arc::new(RwLock::new(manager)))
    }
}
```

### 3. Add Batch Update Method

Add a method to update multiple jobs atomically (single save):

```rust
/// Update multiple job statuses in a single atomic write
pub fn update_statuses_batch(&mut self, updates: &[(String, JobStatus)]) -> Result<(), StatusError> {
    for (job_id, status) in updates {
        if let Some(entry) = self.entries.get_mut(job_id) {
            entry.update_status(*status);
        }
    }
    self.save()
}
```

## Edit Locations

### Edit 1: Add imports at the top

After the existing imports (around line 4), add:
```rust
use std::sync::Arc;
use tokio::sync::RwLock;
```

### Edit 2: Add SharedStatusManager type alias

After the imports, before `pub struct StatusManager`, add:
```rust
/// Thread-safe wrapper for StatusManager
pub type SharedStatusManager = Arc<RwLock<StatusManager>>;
```

### Edit 3: Add new_shared method

Inside `impl StatusManager`, after the `new` method (around line 27), add the `new_shared` factory method.

### Edit 4: Add update_statuses_batch method

Inside `impl StatusManager`, after `update_status` method (around line 118), add the `update_statuses_batch` method.

## Testing Notes

The existing tests should continue to pass. The thread-safe wrapper is additive and doesn't change existing behavior.
