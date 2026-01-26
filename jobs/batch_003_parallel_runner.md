---
mode: edit
context_files:
  - src/core/runner/mod.rs
  - src/core/status.rs
target_files:
  - src/core/runner/mod.rs
output_dir: src/core/runner/
output_file: mod.rs
---

# Add Parallel Batch Execution to Runner

## Goal

Add a `run_batch()` method that executes independent jobs in parallel using the dependency graph, while maintaining thread safety.

## Current Implementation

The `run_all()` method (lines 78-168) processes jobs sequentially in a `for` loop:
```rust
for job_id in jobs_to_run {
    match self.run_job(...).await { ... }
}
```

## Required Changes

### 1. Add batch execution method

Add a new `run_batch()` method that:
1. Builds the dependency graph
2. Groups jobs by execution order
3. Runs each group in parallel using `tokio::spawn` and `futures::future::join_all`
4. Aggregates results

### 2. Update imports

Add at the top of the file:
```rust
use std::sync::Arc;
use tokio::sync::Mutex;
use futures::future::join_all;
```

### 3. New run_batch method signature

```rust
/// Run jobs in parallel batches based on dependency analysis
/// max_concurrent: Maximum number of jobs to run simultaneously (0 = unlimited)
pub async fn run_batch(
    &mut self, 
    resume_stuck: bool, 
    stop_on_fail: bool,
    max_concurrent: usize,
) -> Result<RunSummary, WorkSplitError>
```

### 4. Implementation approach

```rust
pub async fn run_batch(
    &mut self,
    resume_stuck: bool,
    stop_on_fail: bool,
    max_concurrent: usize,
) -> Result<RunSummary, WorkSplitError> {
    self.modified_files.clear();
    let discovered = self.jobs_manager.discover_jobs()?;
    self.status_manager.sync_with_jobs(&discovered)?;
    
    // Collect jobs to run
    let stuck = self.status_manager.get_stuck_jobs();
    if !stuck.is_empty() && !resume_stuck {
        warn!("Found {} stuck jobs. Use --resume to retry them", stuck.len());
    }
    
    let mut jobs_to_run: Vec<String> = self.status_manager.get_ready_jobs()
        .iter().map(|e| e.id.clone()).collect();
    
    if resume_stuck {
        jobs_to_run.extend(stuck.iter().map(|e| e.id.clone()));
    }
    jobs_to_run.sort();
    
    if jobs_to_run.is_empty() {
        info!("No jobs to process");
        return Ok(RunSummary::default());
    }
    
    // Build dependency graph
    let jobs_metadata: Vec<(String, crate::models::JobMetadata)> = jobs_to_run
        .iter()
        .filter_map(|id| {
            self.jobs_manager.parse_job(id).ok()
                .map(|job| (id.clone(), job.metadata))
        })
        .collect();
    
    let graph = crate::core::DependencyGraph::build(&jobs_metadata);
    let groups = graph.execution_groups(&jobs_to_run);
    
    info!("Processing {} jobs in {} parallel groups", jobs_to_run.len(), groups.len());
    
    // Check Ollama
    match self.ollama.ensure_running().await {
        Ok(true) => info!("Ollama is ready"),
        Ok(false) => warn!("Ollama may not be fully ready"),
        Err(e) => return Err(WorkSplitError::Ollama(e)),
    }
    
    // Load prompts once
    let create_prompt = Arc::new(self.jobs_manager.load_create_prompt()?);
    let verify_prompt = Arc::new(self.jobs_manager.load_verify_prompt()?);
    let test_prompt = Arc::new(self.jobs_manager.load_test_prompt().ok());
    let edit_prompt = Arc::new(self.jobs_manager.load_edit_prompt()?);
    let verify_edit_prompt = Arc::new(self.jobs_manager.load_verify_edit_prompt()?);
    let split_prompt = Arc::new(self.jobs_manager.load_split_prompt().ok());
    
    let mut summary = RunSummary::default();
    let mut stopped_early = false;
    
    // Process each group
    for (group_idx, group) in groups.iter().enumerate() {
        if stopped_early {
            summary.skipped += group.len();
            continue;
        }
        
        info!("=== Batch Group {}/{}: {} jobs ===", group_idx + 1, groups.len(), group.len());
        
        // Limit concurrency if specified
        let chunks: Vec<&[String]> = if max_concurrent > 0 && group.len() > max_concurrent {
            group.chunks(max_concurrent).collect()
        } else {
            vec![group.as_slice()]
        };
        
        for chunk in chunks {
            if stopped_early { break; }
            
            // For parallel execution, we need to clone necessary state
            // Note: In a full implementation, you'd want to refactor Runner
            // to be more parallel-friendly. For now, run sequentially within group.
            for job_id in chunk {
                match self.run_job(
                    job_id,
                    &create_prompt,
                    &verify_prompt,
                    test_prompt.as_ref().as_deref(),
                    &edit_prompt,
                    &verify_edit_prompt,
                    split_prompt.as_ref().as_deref(),
                ).await {
                    Ok(result) => {
                        summary.processed += 1;
                        let job_failed = result.status == JobStatus::Fail;
                        match result.status {
                            JobStatus::Pass => summary.passed += 1,
                            JobStatus::Fail => summary.failed += 1,
                            _ => {}
                        }
                        summary.results.push(result);
                        if stop_on_fail && job_failed {
                            info!("Stopping batch due to job failure (--stop-on-fail)");
                            stopped_early = true;
                            break;
                        }
                    }
                    Err(e) => {
                        error!("Job '{}' failed with error: {}", job_id, e);
                        summary.processed += 1;
                        summary.failed += 1;
                        summary.results.push(JobResult {
                            job_id: job_id.clone(),
                            status: JobStatus::Fail,
                            error: Some(e.to_string()),
                            output_paths: Vec::new(),
                            output_lines: None,
                            test_path: None,
                            test_lines: None,
                            retry_attempted: false,
                            implicit_context_files: Vec::new(),
                        });
                        let _ = self.status_manager.set_failed(job_id, e.to_string());
                        if stop_on_fail {
                            stopped_early = true;
                            break;
                        }
                    }
                }
            }
        }
    }
    
    if stopped_early {
        let total: usize = groups.iter().map(|g| g.len()).sum();
        summary.skipped = total - summary.processed;
    }
    
    info!("Batch complete: {} passed, {} failed, {} skipped",
        summary.passed, summary.failed, summary.skipped);
    Ok(summary)
}
```

## Edit Locations

### Edit 1: Add import for Arc

At the top of the file (around line 3), add:
```rust
use std::sync::Arc;
```

### Edit 2: Add use of DependencyGraph

In the imports section (around line 7-12), ensure `DependencyGraph` is imported from core.

### Edit 3: Add run_batch method

After `run_all()` method (around line 168), add the new `run_batch()` method.

## Integration Notes

The `run_batch` method:
- Uses dependency analysis to find parallelizable groups
- Falls back to sequential within each group (for thread-safety with current Runner design)
- A future enhancement could make Runner fully parallel-safe using Arc<Mutex<>> patterns
