---
context_files:
  - src/core/jobs.rs
  - src/models/job.rs
output_dir: src/core/
output_file: dependency.rs
---

# Job Dependency Detection for Parallel Execution

## Goal

Create a dependency graph detector that identifies which jobs can run in parallel vs which must run sequentially due to context file dependencies.

## Dependency Detection Logic

A job B depends on job A if:
1. Job B's `context_files` includes a path that is Job A's `output_file` or in `output_files`
2. Job B's `target_files` (for edit mode) includes a path that Job A outputs

## Required Implementation

### DependencyGraph Struct

```rust
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Represents job dependencies for parallel execution planning
#[derive(Debug, Default)]
pub struct DependencyGraph {
    /// Maps job_id -> set of job_ids it depends on
    dependencies: HashMap<String, HashSet<String>>,
    /// Maps output_path -> job_id that produces it
    output_producers: HashMap<PathBuf, String>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Build dependency graph from job metadata
    pub fn build(jobs: &[(String, crate::models::JobMetadata)]) -> Self {
        let mut graph = Self::new();
        
        // First pass: register all outputs
        for (job_id, metadata) in jobs {
            // Register single output
            let output = metadata.output_path();
            graph.output_producers.insert(output, job_id.clone());
            
            // Register multi-file outputs
            if let Some(ref files) = metadata.output_files {
                for path in files {
                    graph.output_producers.insert(path.clone(), job_id.clone());
                }
            }
        }
        
        // Second pass: detect dependencies
        for (job_id, metadata) in jobs {
            let mut deps = HashSet::new();
            
            // Check context_files
            if let Some(ref context_files) = metadata.context_files {
                for ctx_path in context_files {
                    if let Some(producer) = graph.output_producers.get(ctx_path) {
                        if producer != job_id {
                            deps.insert(producer.clone());
                        }
                    }
                }
            }
            
            // Check target_files (for edit mode)
            if let Some(ref target_files) = metadata.target_files {
                for target_path in target_files {
                    if let Some(producer) = graph.output_producers.get(target_path) {
                        if producer != job_id {
                            deps.insert(producer.clone());
                        }
                    }
                }
            }
            
            graph.dependencies.insert(job_id.clone(), deps);
        }
        
        graph
    }
    
    /// Get execution groups - jobs in same group can run in parallel
    /// Returns groups in order of execution (earlier groups must complete before later ones)
    pub fn execution_groups(&self, ready_jobs: &[String]) -> Vec<Vec<String>> {
        let ready_set: HashSet<&String> = ready_jobs.iter().collect();
        let mut remaining: HashSet<String> = ready_jobs.iter().cloned().collect();
        let mut completed: HashSet<String> = HashSet::new();
        let mut groups = Vec::new();
        
        while !remaining.is_empty() {
            // Find jobs whose dependencies are all completed (or not in ready set)
            let runnable: Vec<String> = remaining
                .iter()
                .filter(|job_id| {
                    let deps = self.dependencies.get(*job_id)
                        .map(|d| d.iter().collect::<Vec<_>>())
                        .unwrap_or_default();
                    deps.iter().all(|dep| {
                        // Dependency is satisfied if:
                        // 1. It's already completed, or
                        // 2. It's not in the ready set (already done in previous run)
                        completed.contains(*dep) || !ready_set.contains(dep)
                    })
                })
                .cloned()
                .collect();
            
            if runnable.is_empty() {
                // Circular dependency detected - just run remaining sequentially
                let mut fallback: Vec<String> = remaining.into_iter().collect();
                fallback.sort();
                groups.push(fallback);
                break;
            }
            
            for job_id in &runnable {
                remaining.remove(job_id);
                completed.insert(job_id.clone());
            }
            
            let mut group = runnable;
            group.sort(); // Consistent ordering
            groups.push(group);
        }
        
        groups
    }
    
    /// Check if job A depends on job B
    pub fn depends_on(&self, job_a: &str, job_b: &str) -> bool {
        self.dependencies
            .get(job_a)
            .map(|deps| deps.contains(job_b))
            .unwrap_or(false)
    }
    
    /// Get direct dependencies for a job
    pub fn get_dependencies(&self, job_id: &str) -> Vec<String> {
        self.dependencies
            .get(job_id)
            .map(|deps| deps.iter().cloned().collect())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Tests would go here
}
```

## Module Registration

After creating this file, add to `src/core/mod.rs`:
```rust
mod dependency;
pub use dependency::DependencyGraph;
```

## Usage Example

```rust
let jobs_metadata: Vec<(String, JobMetadata)> = jobs_manager
    .discover_jobs()?
    .iter()
    .map(|id| (id.clone(), jobs_manager.parse_job(id).unwrap().metadata))
    .collect();

let graph = DependencyGraph::build(&jobs_metadata);
let groups = graph.execution_groups(&ready_job_ids);

// groups[0] can all run in parallel
// groups[1] can run after groups[0] completes
// etc.
```
