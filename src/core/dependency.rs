use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use crate::models::JobMetadata;

/// Represents job dependencies for parallel execution planning
#[derive(Debug, Default)]
pub struct DependencyGraph {
    /// Maps job_id -> set of job_ids it depends on
    dependencies: HashMap<String, HashSet<String>>,
    /// Maps output_path -> job_id that produces it
    output_producers: HashMap<PathBuf, String>,
}

impl DependencyGraph {
    /// Create a new dependency graph
    pub fn new() -> Self {
        Self::default()
    }

    /// Build dependency graph from job metadata
    pub fn build(jobs: &[(String, JobMetadata)]) -> Self {
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
            for ctx_path in &metadata.context_files {
                if let Some(producer) = graph.output_producers.get(ctx_path) {
                    if producer != job_id {
                        deps.insert(producer.clone());
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
    #[allow(dead_code)]
    pub fn depends_on(&self, job_a: &str, job_b: &str) -> bool {
        self.dependencies
            .get(job_a)
            .map(|deps| deps.contains(job_b))
            .unwrap_or(false)
    }

    /// Get direct dependencies for a job
    #[allow(dead_code)]
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
    use crate::models::OutputMode;

    /// Helper to create a minimal JobMetadata for testing
    fn make_metadata(output_file: &str) -> JobMetadata {
        JobMetadata {
            context_files: vec![],
            output_dir: PathBuf::from("src/"),
            output_file: output_file.to_string(),
            test_file: None,
            output_files: None,
            sequential: None,
            mode: OutputMode::Replace,
            target_files: None,
            target_file: None,
        }
    }

    fn make_metadata_with_context(output_file: &str, context: Vec<&str>) -> JobMetadata {
        JobMetadata {
            context_files: context.into_iter().map(PathBuf::from).collect(),
            output_dir: PathBuf::from("src/"),
            output_file: output_file.to_string(),
            test_file: None,
            output_files: None,
            sequential: None,
            mode: OutputMode::Replace,
            target_files: None,
            target_file: None,
        }
    }

    fn make_metadata_with_target(output_file: &str, targets: Vec<&str>) -> JobMetadata {
        JobMetadata {
            context_files: vec![],
            output_dir: PathBuf::from("src/"),
            output_file: output_file.to_string(),
            test_file: None,
            output_files: None,
            sequential: None,
            mode: OutputMode::Edit,
            target_files: Some(targets.into_iter().map(PathBuf::from).collect()),
            target_file: None,
        }
    }

    #[test]
    fn test_empty_graph() {
        let graph = DependencyGraph::new();
        let groups = graph.execution_groups(&[]);
        assert!(groups.is_empty());
    }

    #[test]
    fn test_no_dependencies() {
        let jobs = vec![
            ("job1".to_string(), make_metadata("output1.rs")),
            ("job2".to_string(), make_metadata("output2.rs")),
        ];
        let graph = DependencyGraph::build(&jobs);
        let groups = graph.execution_groups(&["job1".to_string(), "job2".to_string()]);
        // Both jobs can run in parallel (no dependencies)
        assert_eq!(groups.len(), 1);
        assert!(groups[0].contains(&"job1".to_string()));
        assert!(groups[0].contains(&"job2".to_string()));
    }

    #[test]
    fn test_simple_dependency() {
        let mut graph = DependencyGraph::new();
        // job2 depends on job1
        graph.dependencies.insert("job1".to_string(), HashSet::new());
        graph.dependencies.insert("job2".to_string(), ["job1".to_string()].into_iter().collect());
        
        let groups = graph.execution_groups(&["job1".to_string(), "job2".to_string()]);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0], vec!["job1".to_string()]);
        assert_eq!(groups[1], vec!["job2".to_string()]);
    }

    #[test]
    fn test_chain_dependency() {
        let mut graph = DependencyGraph::new();
        // job1 -> job2 -> job3
        graph.dependencies.insert("job1".to_string(), HashSet::new());
        graph.dependencies.insert("job2".to_string(), ["job1".to_string()].into_iter().collect());
        graph.dependencies.insert("job3".to_string(), ["job2".to_string()].into_iter().collect());
        
        let groups = graph.execution_groups(&["job1".to_string(), "job2".to_string(), "job3".to_string()]);
        assert_eq!(groups.len(), 3);
        assert_eq!(groups[0], vec!["job1".to_string()]);
        assert_eq!(groups[1], vec!["job2".to_string()]);
        assert_eq!(groups[2], vec!["job3".to_string()]);
    }

    #[test]
    fn test_parallel_with_shared_dependency() {
        let mut graph = DependencyGraph::new();
        // Both job2 and job3 depend on job1
        graph.dependencies.insert("job1".to_string(), HashSet::new());
        graph.dependencies.insert("job2".to_string(), ["job1".to_string()].into_iter().collect());
        graph.dependencies.insert("job3".to_string(), ["job1".to_string()].into_iter().collect());
        
        let groups = graph.execution_groups(&["job1".to_string(), "job2".to_string(), "job3".to_string()]);
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0], vec!["job1".to_string()]);
        // job2 and job3 should be in the second group (parallel)
        assert_eq!(groups[1].len(), 2);
        assert!(groups[1].contains(&"job2".to_string()));
        assert!(groups[1].contains(&"job3".to_string()));
    }

    #[test]
    fn test_build_from_context_files() {
        // job1 produces src/output1.rs
        // job2 uses src/output1.rs as context -> depends on job1
        let jobs = vec![
            ("job1".to_string(), make_metadata("output1.rs")),
            ("job2".to_string(), make_metadata_with_context("output2.rs", vec!["src/output1.rs"])),
        ];
        let graph = DependencyGraph::build(&jobs);
        assert!(graph.depends_on("job2", "job1"));
    }

    #[test]
    fn test_build_from_target_files() {
        // job1 produces src/target.rs
        // job2 edits src/target.rs -> depends on job1
        let jobs = vec![
            ("job1".to_string(), make_metadata("target.rs")),
            ("job2".to_string(), make_metadata_with_target("output2.rs", vec!["src/target.rs"])),
        ];
        let graph = DependencyGraph::build(&jobs);
        assert!(graph.depends_on("job2", "job1"));
    }

    #[test]
    fn test_depends_on() {
        let mut graph = DependencyGraph::new();
        graph.dependencies.insert("job2".to_string(), ["job1".to_string()].into_iter().collect());
        
        assert!(graph.depends_on("job2", "job1"));
        assert!(!graph.depends_on("job1", "job2"));
        assert!(!graph.depends_on("job3", "job1"));
    }

    #[test]
    fn test_get_dependencies() {
        let mut graph = DependencyGraph::new();
        graph.dependencies.insert("job2".to_string(), ["job1".to_string()].into_iter().collect());
        
        let deps = graph.get_dependencies("job2");
        assert_eq!(deps.len(), 1);
        assert!(deps.contains(&"job1".to_string()));
        
        let no_deps = graph.get_dependencies("job1");
        assert!(no_deps.is_empty());
    }
}
