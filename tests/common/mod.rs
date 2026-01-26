//! Common test utilities

use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Create a test project with jobs folder and required files
pub fn create_test_project() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_root = temp_dir.path().to_path_buf();

    // Create jobs directory
    let jobs_dir = project_root.join("jobs");
    fs::create_dir_all(&jobs_dir).expect("Failed to create jobs dir");

    // Create system prompts
    fs::write(
        jobs_dir.join("_systemprompt_create.md"),
        "You are a code generator. Output only code in a code fence.",
    )
    .expect("Failed to write create prompt");

    fs::write(
        jobs_dir.join("_systemprompt_verify.md"),
        "Verify the code. Respond with PASS or FAIL: reason",
    )
    .expect("Failed to write verify prompt");

    // Create empty job status
    fs::write(jobs_dir.join("_jobstatus.json"), "[]").expect("Failed to write job status");

    (temp_dir, project_root)
}

/// Create a test job file
pub fn create_test_job(
    project_root: &PathBuf,
    job_id: &str,
    output_dir: &str,
    output_file: &str,
    instructions: &str,
) {
    let jobs_dir = project_root.join("jobs");
    let job_content = format!(
        r#"---
context_files: []
output_dir: {}
output_file: {}
---

{}
"#,
        output_dir, output_file, instructions
    );

    fs::write(jobs_dir.join(format!("{}.md", job_id)), job_content)
        .expect("Failed to write job file");
}

/// Create a test job with context files
pub fn create_test_job_with_context(
    project_root: &PathBuf,
    job_id: &str,
    context_files: &[&str],
    output_dir: &str,
    output_file: &str,
    instructions: &str,
) {
    let jobs_dir = project_root.join("jobs");
    
    let context_yaml = context_files
        .iter()
        .map(|f| format!("  - {}", f))
        .collect::<Vec<_>>()
        .join("\n");

    let job_content = format!(
        r#"---
context_files:
{}
output_dir: {}
output_file: {}
---

{}
"#,
        context_yaml, output_dir, output_file, instructions
    );

    fs::write(jobs_dir.join(format!("{}.md", job_id)), job_content)
        .expect("Failed to write job file");
}

/// Create a context file
pub fn create_context_file(project_root: &PathBuf, path: &str, content: &str) {
    let full_path = project_root.join(path);
    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent).expect("Failed to create context file parent dir");
    }
    fs::write(full_path, content).expect("Failed to write context file");
}
