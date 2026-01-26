use std::path::PathBuf;
use tracing::info;

use crate::core::{load_config, JobsManager, DependencyGraph};
use crate::error::WorkSplitError;

/// Show job dependency graph
pub fn show_deps(
    project_root: &PathBuf,
    verbose: bool,
    json: bool,
    quiet: bool,
) -> Result<(), WorkSplitError> {
    let config = load_config(project_root, None, None, None, false)?;
    let jobs_manager = JobsManager::new(project_root.clone(), config.limits.clone());
    
    let discovered = jobs_manager.discover_jobs()?;
    
    // Build metadata for dependency analysis
    let jobs_metadata: Vec<(String, crate::models::JobMetadata)> = discovered
        .iter()
        .filter_map(|id| {
            jobs_manager.parse_job(id).ok()
                .map(|job| (id.clone(), job.metadata))
        })
        .collect();
    
    let graph = DependencyGraph::build(&jobs_metadata);
    let groups = graph.execution_groups(&discovered);
    
    if json {
        // Output JSON format
        let output = serde_json::json!({
            "groups": groups.iter().enumerate().map(|(i, g)| {
                serde_json::json!({
                    "group": i + 1,
                    "jobs": g,
                })
            }).collect::<Vec<_>>(),
            "total_jobs": discovered.len(),
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
        return Ok(());
    }
    
    if !quiet {
        println!("Job Dependencies:");
        println!();
        
        for (idx, group) in groups.iter().enumerate() {
            println!("  Group {}: {} job(s)", idx + 1, group.len());
            for job_id in group {
                if verbose {
                    // Show file dependencies
                    if let Ok(job) = jobs_manager.parse_job(job_id) {
                        let deps: Vec<_> = job.metadata.context_files
                            .iter()
                            .map(|p| p.display().to_string())
                            .collect();
                        if deps.is_empty() {
                            println!("    - {} (no dependencies)", job_id);
                        } else {
                            println!("    - {} (depends on: {})", job_id, deps.join(", "));
                        }
                    }
                } else {
                    println!("    - {}", job_id);
                }
            }
            println!();
        }
        
        println!("Execution Groups: {}", groups.len());
        println!("Total Jobs: {}", discovered.len());
    }
    
    Ok(())
}