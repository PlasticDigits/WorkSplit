//! Integration tests for the job runner

use worksplit::core::{JobsManager, StatusManager};
use worksplit::models::{JobStatus, LimitsConfig};

mod common;

use common::{create_context_file, create_test_job, create_test_job_with_context, create_test_project};

#[test]
fn test_job_discovery() {
    let (_temp_dir, project_root) = create_test_project();

    // Create test jobs
    create_test_job(&project_root, "job_001", "src/", "output1.rs", "Create something");
    create_test_job(&project_root, "job_002", "src/", "output2.rs", "Create another thing");

    let jobs_manager = JobsManager::new(project_root.clone(), LimitsConfig::default());
    let jobs = jobs_manager.discover_jobs().unwrap();

    assert_eq!(jobs.len(), 2);
    assert!(jobs.contains(&"job_001".to_string()));
    assert!(jobs.contains(&"job_002".to_string()));
}

#[test]
fn test_job_parsing() {
    let (_temp_dir, project_root) = create_test_project();

    create_test_job(
        &project_root,
        "test_job",
        "src/services/",
        "user_service.rs",
        "# Create User Service\n\nImplement CRUD operations.",
    );

    let jobs_manager = JobsManager::new(project_root.clone(), LimitsConfig::default());
    let job = jobs_manager.parse_job("test_job").unwrap();

    assert_eq!(job.id, "test_job");
    assert_eq!(job.metadata.output_file, "user_service.rs");
    assert!(job.instructions.contains("Create User Service"));
}

#[test]
fn test_job_parsing_with_context() {
    let (_temp_dir, project_root) = create_test_project();

    // Create context files
    create_context_file(&project_root, "src/models/user.rs", "pub struct User { pub id: i32 }");

    create_test_job_with_context(
        &project_root,
        "test_job",
        &["src/models/user.rs"],
        "src/services/",
        "user_service.rs",
        "Create a service using the User model.",
    );

    let mut jobs_manager = JobsManager::new(project_root.clone(), LimitsConfig::default());
    let job = jobs_manager.parse_job("test_job").unwrap();

    assert_eq!(job.metadata.context_files.len(), 1);
    
    // Load context files
    let context = jobs_manager.load_context_files(&job).unwrap();
    assert_eq!(context.len(), 1);
    assert!(context[0].1.contains("pub struct User"));
}

#[test]
fn test_status_sync() {
    let (_temp_dir, project_root) = create_test_project();

    // Create jobs
    create_test_job(&project_root, "job_a", "src/", "a.rs", "Instructions A");
    create_test_job(&project_root, "job_b", "src/", "b.rs", "Instructions B");

    let jobs_dir = project_root.join("jobs");
    let jobs_manager = JobsManager::new(project_root.clone(), LimitsConfig::default());
    let discovered = jobs_manager.discover_jobs().unwrap();

    let mut status_manager = StatusManager::new(&jobs_dir).unwrap();
    status_manager.sync_with_jobs(&discovered).unwrap();

    // Check all jobs are in created status
    let summary = status_manager.get_summary();
    assert_eq!(summary.total, 2);
    assert_eq!(summary.created, 2);
}

#[test]
fn test_status_transitions() {
    let (_temp_dir, project_root) = create_test_project();

    create_test_job(&project_root, "job_a", "src/", "a.rs", "Instructions");

    let jobs_dir = project_root.join("jobs");
    let jobs_manager = JobsManager::new(project_root.clone(), LimitsConfig::default());
    let discovered = jobs_manager.discover_jobs().unwrap();

    let mut status_manager = StatusManager::new(&jobs_dir).unwrap();
    status_manager.sync_with_jobs(&discovered).unwrap();

    // Test transitions
    status_manager.update_status("job_a", JobStatus::PendingWork).unwrap();
    assert_eq!(status_manager.get("job_a").unwrap().status, JobStatus::PendingWork);

    status_manager.update_status("job_a", JobStatus::PendingVerification).unwrap();
    assert_eq!(status_manager.get("job_a").unwrap().status, JobStatus::PendingVerification);

    status_manager.update_status("job_a", JobStatus::Pass).unwrap();
    assert_eq!(status_manager.get("job_a").unwrap().status, JobStatus::Pass);
}

#[test]
fn test_status_persistence() {
    let (temp_dir, project_root) = create_test_project();

    create_test_job(&project_root, "persistent_job", "src/", "p.rs", "Instructions");

    let jobs_dir = project_root.join("jobs");

    // Create and modify status
    {
        let jobs_manager = JobsManager::new(project_root.clone(), LimitsConfig::default());
        let discovered = jobs_manager.discover_jobs().unwrap();

        let mut status_manager = StatusManager::new(&jobs_dir).unwrap();
        status_manager.sync_with_jobs(&discovered).unwrap();
        status_manager.update_status("persistent_job", JobStatus::Pass).unwrap();
    }

    // Reload and verify
    {
        let status_manager = StatusManager::new(&jobs_dir).unwrap();
        assert_eq!(
            status_manager.get("persistent_job").unwrap().status,
            JobStatus::Pass
        );
    }
}

#[test]
fn test_context_file_size_limit() {
    let (_temp_dir, project_root) = create_test_project();

    // Create a large context file (over 1000 lines)
    let large_content = (0..1100)
        .map(|i| format!("// Line {}\n", i))
        .collect::<String>();
    create_context_file(&project_root, "src/large.rs", &large_content);

    create_test_job_with_context(
        &project_root,
        "test_job",
        &["src/large.rs"],
        "src/",
        "output.rs",
        "Use the large file.",
    );

    let mut jobs_manager = JobsManager::new(project_root.clone(), LimitsConfig::default());
    let job = jobs_manager.parse_job("test_job").unwrap();

    // Loading context should fail due to size
    let result = jobs_manager.load_context_files(&job);
    assert!(result.is_err());
}

#[test]
fn test_missing_context_file() {
    let (_temp_dir, project_root) = create_test_project();

    create_test_job_with_context(
        &project_root,
        "test_job",
        &["src/nonexistent.rs"],
        "src/",
        "output.rs",
        "Use a file that doesn't exist.",
    );

    let mut jobs_manager = JobsManager::new(project_root.clone(), LimitsConfig::default());
    let job = jobs_manager.parse_job("test_job").unwrap();

    let result = jobs_manager.load_context_files(&job);
    assert!(result.is_err());
}

#[test]
fn test_system_prompts_loading() {
    let (_temp_dir, project_root) = create_test_project();

    let jobs_manager = JobsManager::new(project_root.clone(), LimitsConfig::default());

    let create_prompt = jobs_manager.load_create_prompt().unwrap();
    assert!(create_prompt.contains("code generator"));

    let verify_prompt = jobs_manager.load_verify_prompt().unwrap();
    assert!(verify_prompt.contains("PASS"));
}
