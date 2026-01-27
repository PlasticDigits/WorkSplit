---
mode: replace
output_dir: src/core/
output_file: dependency.rs
---

# Plan3 Task 4: Dependency Resolution Implementation

Implement topological sorting for jobs based on `depends_on`.

## Requirements

1. Create `src/core/dependency.rs`.
2. Implement `order_by_dependencies` using Kahn's algorithm or similar.
3. Handle cyclic dependencies.

~~~worksplit:src/core/dependency.rs
use std::collections::{HashMap, VecDeque};
use crate::models::job::Job;
use crate::error::WorkSplitError;

/// Topological sort of jobs based on depends_on
pub fn order_by_dependencies(jobs: &[Job]) -> Result<Vec<&Job>, WorkSplitError> {
    let mut graph: HashMap<&str, Vec<&str>> = HashMap::new();
    let mut in_degree: HashMap<&str, usize> = HashMap::new();
    
    // Build graph
    for job in jobs {
        in_degree.entry(&job.id).or_insert(0);
        if let Some(deps) = &job.metadata.depends_on {
            for dep in deps {
                graph.entry(dep.as_str()).or_default().push(&job.id);
                *in_degree.entry(&job.id).or_insert(0) += 1;
            }
        }
    }
    
    // Kahn's algorithm
    let mut queue: VecDeque<&str> = in_degree
        .iter()
        .filter(|(_, &deg)| deg == 0)
        .map(|(&id, _)| id)
        .collect();
    
    let mut result = Vec::new();
    while let Some(id) = queue.pop_front() {
        if let Some(job) = jobs.iter().find(|j| j.id == id) {
            result.push(job);
        }
        if let Some(neighbors) = graph.get(id) {
            for &neighbor in neighbors {
                let deg = in_degree.get_mut(neighbor).unwrap();
                *deg -= 1;
                if *deg == 0 {
                    queue.push_back(neighbor);
                }
            }
        }
    }
    
    if result.len() != jobs.len() {
        return Err(WorkSplitError::CyclicDependency);
    }
    
    Ok(result)
}
~~~worksplit
