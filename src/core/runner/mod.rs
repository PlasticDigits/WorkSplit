// Core orchestration - the main Runner struct and run methods

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use tracing::{error, info, warn};

use crate::core::{
    assemble_creation_prompt, assemble_test_prompt, assemble_sequential_split_prompt,
    count_lines, extract_code, extract_code_files, JobsManager, OllamaClient, StatusManager,
    SYSTEM_PROMPT_CREATE, SYSTEM_PROMPT_TEST,
};
use crate::error::WorkSplitError;
use crate::models::{Config, ErrorType, JobStatus, Job};

mod edit;
mod sequential;
mod verify;

/// Job runner - orchestrates the creation and verification workflow
pub struct Runner {
    config: Config,
    jobs_manager: JobsManager,
    status_manager: StatusManager,
    ollama: OllamaClient,
    project_root: PathBuf,
    /// Track files modified during current run session
    modified_files: Vec<PathBuf>,
}

/// Result of running a job
#[derive(Debug)]
pub struct JobResult {
    pub job_id: String,
    pub status: JobStatus,
    pub error: Option<String>,
    pub output_paths: Vec<PathBuf>,
    pub output_lines: Option<usize>,
    pub test_path: Option<PathBuf>,
    pub test_lines: Option<usize>,
    pub retry_attempted: bool,
    pub implicit_context_files: Vec<PathBuf>,
}

impl JobResult {
    pub fn output_path(&self) -> Option<&PathBuf> {
        self.output_paths.first()
    }
}

/// Summary of a run
#[derive(Debug, Default)]
pub struct RunSummary {
    pub processed: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub results: Vec<JobResult>,
}

impl Runner {
    pub fn new(config: Config, project_root: PathBuf) -> Result<Self, WorkSplitError> {
        let jobs_manager = JobsManager::new(project_root.clone(), config.limits.clone());
        let status_manager = StatusManager::new(jobs_manager.jobs_dir())?;
        let ollama = OllamaClient::new(config.ollama.clone())?;

        Ok(Self {
            config,
            jobs_manager,
            status_manager,
            ollama,
            project_root,
            modified_files: Vec::new(),
        })
    }

    pub async fn run_all(&mut self, resume_stuck: bool, stop_on_fail: bool, include_ran: bool) -> Result<RunSummary, WorkSplitError> {
        self.modified_files.clear();
        let discovered = self.jobs_manager.discover_jobs()?;
        self.status_manager.sync_with_jobs(&discovered)?;

        let stuck = self.status_manager.get_stuck_jobs();
        if !stuck.is_empty() && !resume_stuck {
            warn!("Found {} stuck jobs. Use --resume to retry them: {:?}",
                stuck.len(), stuck.iter().map(|e| &e.id).collect::<Vec<_>>());
        }

        // Get ready jobs, optionally including those that have already run
        let ready_jobs = if include_ran {
            self.status_manager.get_ready_jobs_include_ran()
        } else {
            self.status_manager.get_ready_jobs()
        };
        let mut jobs_to_run: Vec<String> = ready_jobs.iter().map(|e| e.id.clone()).collect();
        
        // Show info about skipped ran jobs if not including them
        if !include_ran {
            let ran_jobs = self.status_manager.get_ran_non_pass_jobs();
            if !ran_jobs.is_empty() {
                info!("Skipping {} job(s) that already ran. Use --rerun to include them.", ran_jobs.len());
            }
        }

        if resume_stuck {
            jobs_to_run.extend(stuck.iter().map(|e| e.id.clone()));
        }
        jobs_to_run.sort();

        if jobs_to_run.is_empty() {
            info!("No jobs to process");
            return Ok(RunSummary::default());
        }

        let total_jobs = jobs_to_run.len();
        info!("Processing {} jobs", total_jobs);

        match self.ollama.ensure_running().await {
            Ok(true) => info!("Ollama is ready"),
            Ok(false) => warn!("Ollama may not be fully ready"),
            Err(e) => {
                error!("Cannot connect to Ollama: {}", e);
                return Err(WorkSplitError::Ollama(e));
            }
        }

        let create_prompt = self.jobs_manager.load_create_prompt()?;
        let verify_prompt = self.jobs_manager.load_verify_prompt()?;
        let test_prompt = self.jobs_manager.load_test_prompt().ok();
        let edit_prompt = self.jobs_manager.load_edit_prompt()?;
        let verify_edit_prompt = self.jobs_manager.load_verify_edit_prompt()?;
        let split_prompt = self.jobs_manager.load_split_prompt().ok();

        let mut summary = RunSummary::default();
        let mut stopped_early = false;

        for job_id in jobs_to_run {
            match self.run_job(&job_id, &create_prompt, &verify_prompt, test_prompt.as_deref(),
                              &edit_prompt, &verify_edit_prompt, split_prompt.as_deref()).await {
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
                        info!("Stopping due to job failure (--stop-on-fail)");
                        stopped_early = true;
                        break;
                    }
                }
                Err(e) => {
                    error!("Job '{}' failed with error: {}", job_id, e);
                    summary.processed += 1;
                    summary.failed += 1;
                    summary.results.push(JobResult {
                        job_id: job_id.clone(), status: JobStatus::Fail,
                        error: Some(e.to_string()), output_paths: Vec::new(),
                        output_lines: None, test_path: None, test_lines: None,
                        retry_attempted: false, implicit_context_files: Vec::new(),
                    });
                    let _ = self.status_manager.set_failed(&job_id, e.to_string());
                    if stop_on_fail {
                        stopped_early = true;
                        break;
                    }
                }
            }
        }

        if stopped_early {
            summary.skipped = total_jobs - summary.processed;
        }

        info!("Run complete: {} passed, {} failed, {} remaining",
            summary.passed, summary.failed, self.status_manager.get_ready_jobs().len());
        Ok(summary)
    }

    /// Run jobs in parallel batches based on dependency analysis
    /// max_concurrent: Maximum number of jobs to run simultaneously (0 = unlimited)
    pub async fn run_batch(
        &mut self,
        resume_stuck: bool,
        stop_on_fail: bool,
        max_concurrent: usize,
        include_ran: bool,
    ) -> Result<RunSummary, WorkSplitError> {
        self.modified_files.clear();
        let discovered = self.jobs_manager.discover_jobs()?;
        self.status_manager.sync_with_jobs(&discovered)?;

        // Collect jobs to run
        let stuck = self.status_manager.get_stuck_jobs();
        if !stuck.is_empty() && !resume_stuck {
            warn!("Found {} stuck jobs. Use --resume to retry them", stuck.len());
        }

        // Get ready jobs, optionally including those that have already run
        let ready_jobs = if include_ran {
            self.status_manager.get_ready_jobs_include_ran()
        } else {
            self.status_manager.get_ready_jobs()
        };
        let mut jobs_to_run: Vec<String> = ready_jobs.iter().map(|e| e.id.clone()).collect();
        
        // Show info about skipped ran jobs if not including them
        if !include_ran {
            let ran_jobs = self.status_manager.get_ran_non_pass_jobs();
            if !ran_jobs.is_empty() {
                info!("Skipping {} job(s) that already ran. Use --rerun to include them.", ran_jobs.len());
            }
        }

        if resume_stuck {
            jobs_to_run.extend(stuck.iter().map(|e| e.id.clone()));
        }
        jobs_to_run.sort();

        if jobs_to_run.is_empty() {
            info!("No jobs to process");
            return Ok(RunSummary::default());
        }

        let mut sorted_jobs = Vec::new();
        for id in &jobs_to_run {
            if let Ok(job) = self.jobs_manager.parse_job(id) {
                sorted_jobs.push(job);
            }
        }
        let ordered = crate::core::dependency::order_by_dependencies(&sorted_jobs)?
            .into_iter()
            .map(|job| job.id.clone())
            .collect::<Vec<_>>();

        let mut groups = Vec::new();
        if max_concurrent > 0 {
            for chunk in ordered.chunks(max_concurrent) {
                groups.push(chunk.to_vec());
            }
        } else {
            groups.push(ordered);
        }

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
                summary.skipped += group.len() as usize;
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
                                job_id: job_id.to_string(),
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
            let total: usize = groups.iter().map(|g| g.len()).sum::<usize>();
            summary.skipped = total - summary.processed;
        }

        info!("Batch complete: {} passed, {} failed, {} skipped",
            summary.passed, summary.failed, summary.skipped);
        Ok(summary)
    }

    pub async fn run_single(&mut self, job_id: &str) -> Result<JobResult, WorkSplitError> {
        self.modified_files.clear();
        let discovered = self.jobs_manager.discover_jobs()?;
        self.status_manager.sync_with_jobs(&discovered)?;

        let create_prompt = self.jobs_manager.load_create_prompt()?;
        let verify_prompt = self.jobs_manager.load_verify_prompt()?;
        let test_prompt = self.jobs_manager.load_test_prompt().ok();
        let edit_prompt = self.jobs_manager.load_edit_prompt()?;
        let verify_edit_prompt = self.jobs_manager.load_verify_edit_prompt()?;
        let split_prompt = self.jobs_manager.load_split_prompt().ok();

        self.run_job(job_id, &create_prompt, &verify_prompt, test_prompt.as_deref(),
                    &edit_prompt, &verify_edit_prompt, split_prompt.as_deref()).await
    }

    /// Run build command and return (success, output)
    fn run_build_command(&self, cmd: &str) -> Result<(bool, String), WorkSplitError> {
        let output = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .current_dir(&self.project_root)
            .output()?;

        let combined = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        Ok((output.status.success(), combined))
    }

    /// Attempt to auto-fix build errors using LLM
    async fn attempt_auto_fix(
        &self,
        files: &[(PathBuf, String)],
        error_output: &str,
        error_type: ErrorType,
    ) -> Result<bool, WorkSplitError> {
        // Load fix system prompt (auto-recreates from template if missing)
        let system_prompt = match self.jobs_manager.load_system_prompt("_systemprompt_fix.md") {
            Ok(content) => content,
            Err(e) => {
                warn!("Could not load fix prompt: {}. Skipping auto-fix.", e);
                return Ok(false);
            }
        };

        // Build user prompt with all affected files
        let mut user_prompt = format!(
            "{}\n\n```\n{}\n```\n\n{}\n\n",
            error_type.prompt_header(),
            error_output.trim(),
            error_type.fix_instructions()
        );

        // Add each file's content
        for (path, content) in files {
            user_prompt.push_str(&format!(
                "## Source File: {}\n\n```\n{}\n```\n\n",
                path.display(),
                content
            ));
        }

        // Request output for each file
        user_prompt.push_str("Output the complete fixed file(s) using ~~~worksplit:path/to/file delimiters.\n");

        info!("Calling LLM to fix {} errors...", error_type.lowercase_name());
        let response = match self.ollama.generate(Some(&system_prompt), &user_prompt, self.config.behavior.stream_output).await {
            Ok(r) => r,
            Err(e) => {
                warn!("LLM call failed: {}. Skipping auto-fix.", e);
                return Ok(false);
            }
        };

        // Parse output
        let extracted_files = extract_code_files(&response);
        if extracted_files.is_empty() {
            warn!("No code extracted from LLM response");
            return Ok(false);
        }

        // Write fixed files
        let mut files_written = 0;
        for file in &extracted_files {
            let target_path = if let Some(ref path) = file.path {
                self.project_root.join(path)
            } else if files.len() == 1 {
                files[0].0.clone()
            } else {
                continue;
            };

            if let Some(parent) = target_path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)?;
                }
            }

            fs::write(&target_path, &file.content)?;
            info!("Wrote fixed file: {}", target_path.display());
            files_written += 1;
        }

        Ok(files_written > 0)
    }

    async fn verify_with_build(&self, _job: &Job, files: &[(PathBuf, String)]) -> Result<(), WorkSplitError> {
        if !self.config.build.verify_build {
            return Ok(());
        }

        let Some(ref cmd) = self.config.build.build_command else {
            return Ok(());
        };

        info!("Running build verification command: {}", cmd);

        let (success, build_output) = self.run_build_command(cmd)?;

        if success {
            return Ok(());
        }

        // Build failed - try auto-fix if enabled
        let error_context = format!(
            "Build failed after generating files:\n\nFiles generated:\n{}\n\nBuild output:\n{}",
            files.iter().map(|(p, _)| p.display().to_string()).collect::<Vec<_>>().join("\n"),
            build_output
        );

        if !self.config.build.auto_fix {
            return Err(WorkSplitError::BuildFailed {
                command: cmd.clone(),
                output: error_context,
            });
        }

        // Auto-fix loop
        let max_attempts = self.config.build.auto_fix_attempts;
        let mut current_error = build_output;

        for attempt in 1..=max_attempts {
            info!("Auto-fix attempt {}/{}", attempt, max_attempts);

            // Read current file contents (may have been modified)
            let current_files: Vec<(PathBuf, String)> = files.iter()
                .filter_map(|(path, _)| {
                    fs::read_to_string(path).ok().map(|content| (path.clone(), content))
                })
                .collect();

            let fixed = self.attempt_auto_fix(&current_files, &current_error, ErrorType::Build).await?;

            if !fixed {
                warn!("Auto-fix attempt {} produced no changes", attempt);
                continue;
            }

            // Re-run build
            let (success, new_output) = self.run_build_command(cmd)?;

            if success {
                info!("Build succeeded after auto-fix attempt {}", attempt);
                return Ok(());
            }

            current_error = new_output;
            warn!("Build still failing after auto-fix attempt {}", attempt);
        }

        // All attempts exhausted
        Err(WorkSplitError::BuildFailed {
            command: cmd.clone(),
            output: format!(
                "Build failed after {} auto-fix attempts:\n\nFiles:\n{}\n\nFinal error:\n{}",
                max_attempts,
                files.iter().map(|(p, _)| p.display().to_string()).collect::<Vec<_>>().join("\n"),
                current_error
            ),
        })
    }

    async fn run_job(&mut self, job_id: &str, create_prompt: &str, verify_prompt: &str,
                     test_prompt: Option<&str>, edit_prompt: &str, verify_edit_prompt: &str,
                     split_prompt: Option<&str>) -> Result<JobResult, WorkSplitError> {
        info!("Processing job: {}", job_id);
        let job = self.jobs_manager.parse_job(job_id)?;
        let context_files = self.load_context_files_with_implicit(&job)?;

        let (tokens, is_warning, is_error) = self.jobs_manager.check_token_budget(
            create_prompt, &context_files, &job.instructions, 32000);
        if is_error {
            return Err(WorkSplitError::TokenBudgetExceeded { estimated: tokens, max: 32000 });
        }
        if is_warning {
            warn!("Job '{}' has high token usage: {} estimated", job_id, tokens);
        }

        self.status_manager.update_status(job_id, JobStatus::PendingWork)?;

        let mut test_result_path: Option<PathBuf> = None;
        let mut test_result_lines: Option<usize> = None;

        if job.metadata.is_tdd_enabled() {
            let test_prompt_str = test_prompt.ok_or_else(|| WorkSplitError::SystemPromptNotFound(
                self.jobs_manager.jobs_dir().join("_systemprompt_test.md")))?;
            info!("TDD workflow enabled for job '{}'", job_id);
            self.status_manager.update_status(job_id, JobStatus::PendingTest)?;

            let test_path = job.metadata.test_path().unwrap();
            let test_gen_prompt = assemble_test_prompt(test_prompt_str, &context_files,
                &job.instructions, &test_path.display().to_string());

            let test_response = self.ollama.generate_with_retry(Some(SYSTEM_PROMPT_TEST), &test_gen_prompt, self.config.behavior.stream_output)
                .await.map_err(|e| { let _ = self.status_manager.set_failed(job_id, e.to_string()); WorkSplitError::Ollama(e) })?;

            let test_code = extract_code(&test_response);
            let full_test_path = self.project_root.join(&test_path);
            if let Some(parent) = full_test_path.parent() {
                if !parent.exists() && self.config.behavior.create_output_dirs {
                    fs::create_dir_all(parent)?;
                }
            }
            self.safe_write(&full_test_path, &test_code)?;
            test_result_path = Some(full_test_path);
            test_result_lines = Some(count_lines(&test_code));
        }

        let default_output_path = job.metadata.output_path();
        let mut generated_files: Vec<(PathBuf, String)> = Vec::new();
        let mut full_output_paths: Vec<PathBuf> = Vec::new();
        let mut total_lines = 0;

        if job.metadata.is_split_mode() {
            let split_system_prompt = split_prompt.ok_or_else(|| WorkSplitError::SystemPromptNotFound(
                self.jobs_manager.jobs_dir().join("_systemprompt_split.md")))?;
            let target_file_path = job.metadata.target_file.as_ref().unwrap();
            let output_files = job.metadata.get_output_files();
            info!("Split mode (sequential): splitting {} into {} file(s)", target_file_path.display(), output_files.len());
            
            let target_content = self.jobs_manager.load_target_file_unlimited(target_file_path)?;
            let mut previously_generated: Vec<(PathBuf, String)> = Vec::new();
            
            for (idx, output_path) in output_files.iter().enumerate() {
                let remaining_files: Vec<PathBuf> = output_files[idx + 1..].to_vec();
                info!("[{}/{}] Splitting into: {}", idx + 1, output_files.len(), output_path.display());
                
                let prompt = assemble_sequential_split_prompt(split_system_prompt,
                    (target_file_path, &target_content), &context_files, &previously_generated,
                    &job.instructions, &output_path.display().to_string(), &remaining_files);
                
                let response = self.ollama.generate_with_retry(Some(SYSTEM_PROMPT_CREATE), &prompt, self.config.behavior.stream_output)
                    .await.map_err(|e| { let _ = self.status_manager.set_failed(job_id, e.to_string()); WorkSplitError::Ollama(e) })?;
                
                let extracted = extract_code_files(&response);
                let content = if extracted.is_empty() { extract_code(&response) } else { extracted[0].content.clone() };
                
                if content.is_empty() {
                    let msg = format!("Split produced no content for {}", output_path.display());
                    self.status_manager.set_failed(job_id, msg.clone())?;
                    return Err(WorkSplitError::EditFailed(msg));
                }
                
                total_lines += count_lines(&content);
                let full_path = self.project_root.join(output_path);
                if let Some(parent) = full_path.parent() {
                    if !parent.exists() && self.config.behavior.create_output_dirs { fs::create_dir_all(parent)?; }
                }
                self.safe_write(&full_path, &content)?;
                
                previously_generated.push((output_path.clone(), content.clone()));
                generated_files.push((output_path.clone(), content));
                self.modified_files.push(full_path.clone());
                full_output_paths.push(full_path);
            }
        } else if job.metadata.is_edit_mode() {
            let result = edit::process_edit_mode(
                &self.ollama,
                &self.project_root,
                &self.config,
                &job,
                &context_files,
                &edit_prompt,
                false, // dry_run
            ).await?;
            generated_files = result.generated_files;
            full_output_paths = result.output_paths;
            total_lines = result.total_lines;
        } else if job.metadata.is_sequential() {
            let files = sequential::process_sequential_mode(
                &self.ollama,
                &self.project_root,
                &self.config,
                &job,
                &context_files,
                &create_prompt,
            ).await?;
            generated_files = files.0;
            full_output_paths = files.1;
            total_lines = files.2;
        } else {
            let prompt = assemble_creation_prompt(create_prompt, &context_files, &job.instructions,
                &default_output_path.display().to_string());
            let response = self.ollama.generate_with_retry(Some(SYSTEM_PROMPT_CREATE), &prompt, self.config.behavior.stream_output)
                .await.map_err(|e| { let _ = self.status_manager.set_failed(job_id, e.to_string()); WorkSplitError::Ollama(e) })?;
            
            for file in extract_code_files(&response) {
                let path = file.path.clone().unwrap_or_else(|| default_output_path.clone());
                total_lines += count_lines(&file.content);
                generated_files.push((path, file.content.clone()));
            }
            
            for (path, content) in &generated_files {
                let full_path = self.project_root.join(path);
                if let Some(parent) = full_path.parent() {
                    if !parent.exists() && self.config.behavior.create_output_dirs { fs::create_dir_all(parent)?; }
                }
                self.safe_write(&full_path, content)?;
                self.modified_files.push(full_path.clone());
                full_output_paths.push(full_path);
            }
        }

        self.verify_with_build(&job, &generated_files).await?;

        // Check if verification is disabled for this job
        let mut final_status = JobStatus::Pass;
        let mut final_error: Option<String> = None;
        let mut retry_attempted = false;

        if !job.metadata.verify {
            info!("Verification skipped (verify: false in job metadata)");
            self.status_manager.update_status(job_id, JobStatus::Pass)?;
        } else {
            self.status_manager.update_status(job_id, JobStatus::PendingVerification)?;

            let effective_verify = if job.metadata.is_edit_mode() { verify_edit_prompt } else { verify_prompt };
            let (mut final_result, mut err) = verify::run_verification(
                &self.ollama,
                effective_verify,
                &context_files,
                &generated_files,
                &job.instructions,
            ).await?;

            final_status = final_result.to_job_status();
            final_error = err;

            if !final_result.is_pass() {
                info!("Verification failed, retrying...");
                retry_attempted = true;
                let error_msg = final_error.clone().unwrap_or_default();
                
                let retry_files = verify::run_retry(
                    &self.ollama,
                    create_prompt,
                    &context_files,
                    &generated_files,
                    &job.instructions,
                    &error_msg,
                ).await?;

                for (path, content) in &retry_files {
                    let full_path = self.project_root.join(path);
                    if let Some(parent) = full_path.parent() {
                        if !parent.exists() && self.config.behavior.create_output_dirs { fs::create_dir_all(parent)?; }
                    }
                    self.safe_write(&full_path, content)?;
                    self.modified_files.push(full_path.clone());
                }
                
                full_output_paths = retry_files.iter().map(|(p, _)| self.project_root.join(p)).collect();
                
                let (r, e) = verify::run_verification(
                    &self.ollama,
                    effective_verify,
                    &context_files,
                    &retry_files,
                    &job.instructions,
                ).await?;
                final_result = r;
                final_error = e;
                final_status = final_result.to_job_status();
            }

            if let Some(ref msg) = final_error {
                self.status_manager.set_failed(job_id, msg.clone())?;
            } else {
                self.status_manager.update_status(job_id, final_status)?;
            }
        }

        // Mark the job as having been run (regardless of outcome)
        // This prevents unnecessary reruns when the output was manually fixed
        if let Err(e) = self.status_manager.mark_ran(job_id) {
            warn!("Failed to mark job as ran: {}", e);
        }

        if final_status == JobStatus::Pass {
            info!("Generation complete. REMINDER: Ensure new code is wired into callers.");
        }

        info!("Job '{}' completed with status: {:?}", job_id, final_status);
        Ok(JobResult {
            job_id: job_id.to_string(), status: final_status, error: final_error,
            output_paths: full_output_paths, output_lines: Some(total_lines),
            test_path: test_result_path, test_lines: test_result_lines,
            retry_attempted, implicit_context_files: Vec::new(),
        })
    }

    fn load_context_files_with_implicit(&mut self, job: &crate::models::Job) -> Result<Vec<(PathBuf, String)>, WorkSplitError> {
        let mut context_files = self.jobs_manager.load_context_files(job)?;
        if !self.modified_files.is_empty() {
            let max = self.config.limits.max_context_files;
            let available = max.saturating_sub(context_files.len());
            if available > 0 {
                let output_path = self.project_root.join(job.metadata.output_path());
                let implicit: Vec<&PathBuf> = self.modified_files.iter()
                    .filter(|p| p.exists() && *p != &output_path)
                    .take(available).collect();
                for path in implicit {
                    if let Ok(content) = fs::read_to_string(path) {
                        context_files.push((path.clone(), content));
                    }
                }
            }
        }
        Ok(context_files)
    }

    fn is_protected_path(&self, path: &Path) -> bool {
        let jobs_dir = self.jobs_manager.jobs_dir();
        if let Ok(canonical_jobs) = jobs_dir.canonicalize() {
            if let Ok(canonical_path) = path.canonicalize() {
                if canonical_path.starts_with(&canonical_jobs) {
                    if let Some(name) = canonical_path.file_name().and_then(|f| f.to_str()) {
                        return name.starts_with('_');
                    }
                }
            }
        }
        false
    }

    fn safe_write(&mut self, path: &Path, content: &str) -> Result<(), WorkSplitError> {
        if self.is_protected_path(path) {
            return Err(WorkSplitError::ProtectedPathViolation(path.to_path_buf()));
        }
        fs::write(path, content)?;
        // Invalidate cache entry since file was modified
        self.jobs_manager.invalidate_cache(path);
        Ok(())
    }

    pub fn get_summary(&self) -> crate::core::StatusSummary { self.status_manager.get_summary() }
    pub fn reset_job(&mut self, job_id: &str) -> Result<(), WorkSplitError> {
        self.status_manager.reset_job(job_id)?;
        Ok(())
    }
    pub fn status_manager(&self) -> &StatusManager { &self.status_manager }
    pub fn jobs_manager(&self) -> &JobsManager { &self.jobs_manager }
    
    pub fn cache_stats(&self) -> crate::core::file_cache::CacheStats {
        self.jobs_manager.cache_stats()
    }
}