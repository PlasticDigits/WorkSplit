#![allow(dead_code)]
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

mod commands;
mod core;
mod error;
mod models;
mod templates;

use commands::{cancel_jobs, create_new_job, init_project, print_validation_result, retry_job, run_jobs, show_status, validate_jobs, RunOptions};
use models::{JobTemplate, Language};

/// WorkSplit - Ollama-powered job orchestrator for code generation
#[derive(Parser)]
#[command(name = "worksplit")]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new WorkSplit project
    Init {
        /// Project directory (defaults to current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Programming language for the project (prompts interactively if not specified)
        #[arg(short, long, value_enum)]
        lang: Option<Language>,

        /// Ollama model to use (prompts interactively if not specified)
        /// 
        /// AI Manager Note: If you are an AI assistant, STOP and ask the user
        /// which model they want before running this command.
        #[arg(short, long)]
        model: Option<String>,
    },

    /// Reset job status
    Reset {
        /// Job ID to reset (or "all" for all failed jobs)
        job: String,

        /// Reset all jobs matching status (e.g., "fail", "partial")
        #[arg(long)]
        status: Option<String>,
    },

    /// Cancel a running job (or all)
    Cancel {
        /// Job ID to cancel (or "all")
        job: String,
    },

    /// Retry a job (reset + run)
    Retry {
        /// Job ID to retry
        job: String,
    },

    /// Run pending jobs
    Run {
        /// Specific job ID to run
        #[arg(short, long)]
        job: Option<String>,

        /// Preview what would run without executing
        #[arg(long)]
        dry_run: bool,

        /// Resume stuck jobs (pending_work or pending_verification)
        #[arg(long)]
        resume: bool,

        /// Per-job timeout in seconds
        #[arg(long)]
        job_timeout: Option<u64>,

        /// Reset a job to created status
        #[arg(long)]
        reset: Option<String>,

        /// Override the model to use
        #[arg(long)]
        model: Option<String>,

        /// Override the Ollama URL
        #[arg(long)]
        url: Option<String>,

        /// Override the timeout in seconds
        #[arg(long)]
        timeout: Option<u64>,

        /// Disable streaming output
        #[arg(long)]
        no_stream: bool,

        /// Stop processing when any job fails
        #[arg(long)]
        stop_on_fail: bool,

        /// Enable batch mode with parallel execution
        #[arg(long)]
        batch: bool,

        /// Maximum concurrent jobs in batch mode (0 = unlimited)
        #[arg(long, default_value = "0")]
        max_concurrent: usize,
    },

    /// Show job status
    Status {
        /// Show detailed status for each job
        #[arg(short, long)]
        verbose: bool,
    },

    /// Validate jobs folder structure
    Validate,

    /// Create a new job from a template
    NewJob {
        /// Job name (e.g., "auth_001_login")
        name: String,

        /// Job template type
        #[arg(long = "type", short = 't', value_enum, default_value = "replace")]
        template: JobTemplate,

        /// Target files for edit mode
        #[arg(long = "targets", value_delimiter = ',')]
        target_files: Option<Vec<PathBuf>>,

        /// Output directory
        #[arg(long, short = 'o', default_value = "src/")]
        output_dir: PathBuf,

        /// Output filename (defaults to job name + appropriate extension)
        #[arg(long, short = 'f')]
        output_file: Option<String>,

        /// Context files to include
        #[arg(long, short = 'c', value_delimiter = ',')]
        context_files: Option<Vec<PathBuf>>,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Set up logging
    let level = if cli.verbose { Level::DEBUG } else { Level::INFO };
    let _subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .with_target(false)
        .without_time()
        .init();

    let result = match cli.command {
        Commands::Init { path, lang, model } => {
            let project_root = path.unwrap_or_else(|| std::env::current_dir().unwrap());
            init_project(&project_root, lang, model)
        }

        Commands::Reset { job, status } => {
            let project_root = std::env::current_dir().unwrap();
            crate::commands::reset::reset_jobs(&project_root, &job, status.as_deref())
        }

        Commands::Cancel { job } => {
            let project_root = std::env::current_dir().unwrap();
            cancel_jobs(&project_root, &job)
        }

        Commands::Retry { job } => {
            let project_root = std::env::current_dir().unwrap();
            retry_job(&project_root, &job).await
        }

        Commands::Run {
            job,
            dry_run,
            resume,
            job_timeout,
            reset,
            model,
            url,
            timeout,
            no_stream,
            stop_on_fail,
            batch,
            max_concurrent,
        } => {
            let project_root = std::env::current_dir().unwrap();
            let options = RunOptions {
                job_id: job,
                dry_run,
                resume,
                reset,
                model,
                url,
                timeout,
                job_timeout,
                no_stream,
                stop_on_fail,
                batch,
                max_concurrent,
            };
            run_jobs(&project_root, options).await
        }

        Commands::Status { verbose } => {
            let project_root = std::env::current_dir().unwrap();
            show_status(&project_root, verbose)
        }

        Commands::Validate => {
            let project_root = std::env::current_dir().unwrap();
            match validate_jobs(&project_root) {
                Ok(result) => {
                    print_validation_result(&result);
                    if result.valid {
                        Ok(())
                    } else {
                        std::process::exit(1);
                    }
                }
                Err(e) => Err(e),
            }
        }

        Commands::NewJob {
            name,
            template,
            target_files,
            output_dir,
            output_file,
            context_files,
        } => {
            let project_root = std::env::current_dir().unwrap();
            create_new_job(
                &project_root,
                &name,
                template,
                target_files,
                &output_dir,
                output_file,
                context_files,
            )
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}