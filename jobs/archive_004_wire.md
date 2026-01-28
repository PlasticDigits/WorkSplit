---
mode: edit
target_files:
  - src/commands/mod.rs
  - src/main.rs
output_dir: src/commands/
output_file: mod.rs
---

# Wire Up Archive and Cleanup Commands

Add the archive and cleanup commands to the module system and CLI.

## Changes to src/commands/mod.rs

### Add module declarations

After `pub mod cancel;` add:
```rust
pub mod archive;
pub mod cleanup;
```

### Add pub use statements

After `pub use cancel::*;` add:
```rust
pub use archive::*;
pub use cleanup::*;
```

## Changes to src/main.rs

### Add to Commands enum

Add these new subcommands after the existing ones:

```rust
/// Archive completed jobs older than X days
Archive {
    /// Days threshold (uses config default if not specified)
    #[arg(short, long)]
    days: Option<u32>,
    
    /// Preview what would be archived without moving files
    #[arg(long)]
    dry_run: bool,
},

/// Clean up old archived jobs
Cleanup {
    /// Days threshold (uses config default if not specified)  
    #[arg(short, long)]
    days: Option<u32>,
    
    /// Preview what would be deleted
    #[arg(long)]
    dry_run: bool,
},
```

### Add command handlers in main match

Add these match arms:

```rust
Commands::Archive { days, dry_run } => {
    let project_root = std::env::current_dir().unwrap();
    match archive_jobs(&project_root, days, dry_run) {
        Ok(result) => {
            if dry_run {
                println!("Dry run: would archive {} job(s)", result.archived_count);
            } else if result.archived_count > 0 {
                println!("\nArchived {} job(s) to jobs/archive/", result.archived_count);
            } else {
                println!("No jobs to archive");
            }
            Ok(())
        }
        Err(e) => Err(e),
    }
}

Commands::Cleanup { days, dry_run } => {
    let project_root = std::env::current_dir().unwrap();
    match cleanup_archived_jobs(&project_root, days, dry_run) {
        Ok(result) => {
            if dry_run {
                println!("Dry run: would delete {} archived job(s)", result.deleted_count);
            } else if result.deleted_count > 0 {
                println!("\nCleaned up {} archived job(s)", result.deleted_count);
            } else {
                println!("No archived jobs to clean up");
            }
            Ok(())
        }
        Err(e) => Err(e),
    }
}
```

### Update imports in main.rs

Add to the `use commands::` import list:
```rust
archive_jobs, cleanup_archived_jobs,
```
