use std::fs;
use std::path::PathBuf;
use tracing::info;

use dialoguer::{Select, theme::ColorfulTheme};

use crate::error::WorkSplitError;
use crate::models::Language;
use crate::templates::get_templates;

/// Initialize a new WorkSplit project with the specified or selected language
pub fn init_project(project_root: &PathBuf, lang: Option<Language>) -> Result<(), WorkSplitError> {
    // Determine the language - use provided or prompt interactively
    let language = match lang {
        Some(l) => l,
        None => prompt_for_language()?,
    };

    info!("Initializing {} project", language.display_name());
    println!("Initializing {} WorkSplit project...", language.display_name());

    let templates = get_templates(language);
    let jobs_dir = project_root.join("jobs");

    // Create jobs directory
    if !jobs_dir.exists() {
        fs::create_dir_all(&jobs_dir)?;
        info!("Created jobs directory: {}", jobs_dir.display());
    } else {
        info!("Jobs directory already exists: {}", jobs_dir.display());
    }

    // Create system prompts from templates
    create_file_if_not_exists(
        &jobs_dir.join("_systemprompt_create.md"),
        templates.create_prompt,
    )?;

    create_file_if_not_exists(
        &jobs_dir.join("_systemprompt_verify.md"),
        templates.verify_prompt,
    )?;

    create_file_if_not_exists(
        &jobs_dir.join("_systemprompt_test.md"),
        templates.test_prompt,
    )?;

    create_file_if_not_exists(
        &jobs_dir.join("_managerinstruction.md"),
        templates.manager_instruction,
    )?;

    // Create empty job status file
    create_file_if_not_exists(
        &jobs_dir.join("_jobstatus.json"),
        "[]",
    )?;

    // Create config file with language-specific settings
    create_file_if_not_exists(
        &project_root.join("worksplit.toml"),
        templates.config,
    )?;

    // Create example jobs
    create_file_if_not_exists(
        &jobs_dir.join("example_001.md"),
        templates.example_job,
    )?;

    create_file_if_not_exists(
        &jobs_dir.join("example_002_tdd.md"),
        templates.tdd_example_job,
    )?;

    info!("WorkSplit project initialized successfully!");
    print_next_steps(project_root, language);

    Ok(())
}

/// Prompt user to select a language interactively
fn prompt_for_language() -> Result<Language, WorkSplitError> {
    let languages = Language::all();
    let items: Vec<&str> = languages.iter().map(|l| l.display_name()).collect();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select project language")
        .items(&items)
        .default(0)
        .interact()
        .map_err(|e| WorkSplitError::InitError(format!("Failed to get user input: {}", e)))?;

    Ok(languages[selection])
}

fn create_file_if_not_exists(path: &PathBuf, content: &str) -> Result<(), WorkSplitError> {
    if !path.exists() {
        fs::write(path, content)?;
        info!("Created file: {}", path.display());
    } else {
        info!("File already exists: {}", path.display());
    }
    Ok(())
}

fn print_next_steps(project_root: &PathBuf, language: Language) {
    println!("WorkSplit project initialized at {}", project_root.display());
    println!("\nLanguage: {}", language.display_name());
    println!("\nNext steps:");
    println!("1. Edit jobs/_systemprompt_create.md to customize code generation instructions");
    println!("2. Edit jobs/_systemprompt_verify.md to customize verification instructions");
    println!("3. Edit jobs/_systemprompt_test.md to customize test generation (for TDD workflow)");
    println!("4. Create job files in the jobs/ directory");
    println!("5. Run 'worksplit run' to process jobs");
    println!("\nTip: Add 'test_file: <filename>' to job frontmatter to enable TDD workflow");
    
    match language {
        Language::Rust => {
            println!("\nRust-specific tips:");
            println!("- Use .rs extension for output files");
            println!("- Build command: cargo check");
            println!("- Test command: cargo test");
        }
        Language::Typescript => {
            println!("\nTypeScript-specific tips:");
            println!("- Use .ts extension for output files");
            println!("- Build command: npm run build");
            println!("- Test command: npm test");
        }
    }
}
