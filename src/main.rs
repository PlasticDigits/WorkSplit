use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

mod commands;
mod core;
mod error;
mod models;

use commands::{create_new_job, init_project, print_validation_result, run_jobs, show_status, validate_jobs, RunOptions};
use models::JobTemplate;

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
    },

    /// Run pending jobs
    Run {
        /// Specific job ID to run
        #[arg(short, long)]
        job: Option<String>,

        /// Resume stuck jobs (pending_work or pending_verification)
        #[arg(long)]
        resume: bool,

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
    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .with_target(false)
        .without_time()
        .init();

    let result = match cli.command {
        Commands::Init { path } => {
            let project_root = path.unwrap_or_else(|| std::env::current_dir().unwrap());
            init_project(&project_root)
        }

        Commands::Run {
            job,
            resume,
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
                resume,
                reset,
                model,
                url,
                timeout,
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