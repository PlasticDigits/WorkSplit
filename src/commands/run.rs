use std::path::PathBuf;
use tracing::{info, warn};

use crate::commands::archive::run_auto_archive;
use crate::core::{load_config, Runner};
use crate::error::WorkSplitError;
use crate::models::JobStatus;

/// Run options
pub struct RunOptions {
    /// Specific job to run (if None, run all pending)
    pub job_id: Option<String>,
    /// Preview what would run without executing jobs
    pub dry_run: bool,
    /// Resume stuck jobs
    pub resume: bool,
    /// Reset specific job to created status
    pub reset: Option<String>,
    /// Model override
    pub model: Option<String>,
    /// URL override
    pub url: Option<String>,
    /// Timeout override
    pub timeout: Option<u64>,
    /// Per-job timeout in seconds (not yet implemented)
    pub job_timeout: Option<u64>,
    /// Disable streaming output
    pub no_stream: bool,
    /// Stop processing when any job fails
    pub stop_on_fail: bool,
    /// Enable batch mode with dependency-based parallel execution
    pub batch: bool,
    /// Maximum concurrent jobs (0 = unlimited)
    pub max_concurrent: usize,
    /// Include jobs that have already been run (ran=true)
    pub rerun: bool,
}

impl Default for RunOptions {
    fn default() -> Self {
        Self {
            job_id: None,
            dry_run: false,
            resume: false,
            reset: None,
            model: None,
            url: None,
            timeout: None,
            job_timeout: None,
            no_stream: false,
            stop_on_fail: false,
            batch: false,
            max_concurrent: 0,
            rerun: false,
        }
    }
}

/// Run jobs
pub async fn run_jobs(project_root: &PathBuf, options: RunOptions) -> Result<(), WorkSplitError> {
    let config = load_config(
        project_root,
        options.model,
        options.url,
        options.timeout,
        options.no_stream,
    )?;

    let mut runner = Runner::new(config, project_root.clone())?;

    // Handle reset
    if let Some(job_id) = options.reset {
        runner.reset_job(&job_id)?;
        println!("Reset job '{}' to created status", job_id);
        return Ok(());
    }

    // Run specific job or all jobs
    if let Some(job_id) = options.job_id {
        info!("Running single job: {}", job_id);
        // Keep behavior, but update dry-run output text to mention jobs
        
        if options.dry_run {
            println!("=== DRY RUN ===\n");
            if let Ok(job) = runner.jobs_manager().parse_job(&job_id) {
                println!("  {} [{:?}]", job_id, job.metadata.mode);
                println!("    Context: {:?}", job.metadata.context_files);
                println!("    Output:  {}", job.metadata.output_path().display());
                if let Some(targets) = &job.metadata.target_files {
                    println!("    Targets: {:?}", targets);
                }
            } else {
                println!("  {} [Error parsing job]", job_id);
            }
            println!("\nRun without --dry-run to execute.");
            return Ok(());
        }

        let result = runner.run_single(&job_id).await?;
        
        print_job_result(&result.job_id, result.status, result.error.as_deref(), result.output_lines);
        
        // Exit with error if job failed and stop_on_fail is set
        if options.stop_on_fail && result.status == JobStatus::Fail {
            println!("\nStopping due to failure (--stop-on-fail)");
            std::process::exit(1);
        }
    } else if options.batch {
        info!("Running in batch mode");

        if options.dry_run {
            println!("=== DRY RUN (BATCH) ===\n");
            // Simplified preview for batch mode
            println!("Would process jobs in dependency order/batches.");
            println!("Run without --dry-run to execute.");
            return Ok(());
        }

        let summary = runner.run_batch(options.resume, options.stop_on_fail, options.max_concurrent, options.rerun).await?;
        
        println!("\n=== Batch Run Summary ===");
        println!("Processed: {}", summary.processed);
        println!("Passed:    {}", summary.passed);
        println!("Failed:    {}", summary.failed);
        if summary.skipped > 0 {
            println!("Skipped:   {} (not processed)", summary.skipped);
        }
        
        if !summary.results.is_empty() {
            println!("\nResults:");
            for result in &summary.results {
                print_job_result(&result.job_id, result.status, result.error.as_deref(), result.output_lines);
            }
        }
        
        if options.stop_on_fail && summary.failed > 0 {
            println!("\nStopping due to failure (--stop-on-fail)");
            std::process::exit(1);
        }
    } else {
        info!("Running all pending jobs");

        if options.dry_run {
            println!("=== DRY RUN (ALL) ===\n");
            println!("Would process all pending jobs in order.");
            println!("Run without --dry-run to execute.");
            return Ok(());
        }

        let summary = runner.run_all(options.resume, options.stop_on_fail, options.rerun).await?;
        
        println!("\n=== Run Summary ===");
        println!("Processed: {}", summary.processed);
        println!("Passed:    {}", summary.passed);
        println!("Failed:    {}", summary.failed);
        if summary.skipped > 0 {
            println!("Skipped:   {} (not processed)", summary.skipped);
        }
        
        if !summary.results.is_empty() {
            println!("\nResults:");
            for result in &summary.results {
                print_job_result(&result.job_id, result.status, result.error.as_deref(), result.output_lines);
            }
        }
        
        // Exit with error if any job failed and stop_on_fail is set
        if options.stop_on_fail && summary.failed > 0 {
            println!("\nStopping due to failure (--stop-on-fail)");
            std::process::exit(1);
        }
    }

    // Run auto-archive after jobs complete (which triggers auto-cleanup)
    if !options.dry_run {
        if let Err(e) = run_auto_archive(project_root) {
            // Don't fail the run for archive errors, just log
            warn!("Auto-archive failed: {}", e);
        }
    }

    Ok(())
}

fn print_job_result(job_id: &str, status: JobStatus, error: Option<&str>, lines: Option<usize>) {
    let status_str = match status {
        JobStatus::Pass => "PASS",
        JobStatus::Fail => "FAIL",
        _ => "???",
    };
    
    let lines_str = lines.map(|l| format!(" ({} lines)", l)).unwrap_or_default();
    
    match error {
        Some(err) => println!("  {} [{}]{}: {}", job_id, status_str, lines_str, err),
        None => println!("  {} [{}]{}", job_id, status_str, lines_str),
    }
}
