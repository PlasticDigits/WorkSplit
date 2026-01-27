use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tracing::info;

use dialoguer::{Select, theme::ColorfulTheme};

use crate::error::WorkSplitError;
use crate::models::Language;
use crate::templates::get_templates;

/// Initialize a new WorkSplit project with the specified or selected language and model
pub fn init_project(project_root: &PathBuf, lang: Option<Language>, model: Option<String>) -> Result<(), WorkSplitError> {
    // Determine the language - use provided or prompt interactively
    let language = match lang {
        Some(l) => l,
        None => prompt_for_language()?,
    };

    info!("Initializing {} project", language.display_name());
    println!("Initializing {} WorkSplit project...", language.display_name());

    // Determine the model - use provided or prompt interactively
    let selected_model = match model {
        Some(m) => m,
        None => {
            let models = fetch_ollama_models()?;
            prompt_for_model(models)?
        }
    };

    info!("Using model: {}", selected_model);

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
        &jobs_dir.join("_systemprompt_edit.md"),
        templates.edit_prompt,
    )?;

    create_file_if_not_exists(
        &jobs_dir.join("_systemprompt_verify_edit.md"),
        templates.verify_edit_prompt,
    )?;

    create_file_if_not_exists(
        &jobs_dir.join("_systemprompt_split.md"),
        templates.split_prompt,
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

    // Create config file with language-specific settings and selected model
    let config_content = templates.config.replace(
        "model = \"qwen-32k:latest\"",
        &format!("model = \"{}\"", selected_model)
    );
    create_file_if_not_exists(
        &project_root.join("worksplit.toml"),
        &config_content,
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
    print_next_steps(project_root, language, &selected_model);

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

/// Fetch available Ollama models by running 'ollama list' command
fn fetch_ollama_models() -> Result<Vec<String>, WorkSplitError> {
    let output = Command::new("ollama")
        .arg("list")
        .output()
        .map_err(|e| WorkSplitError::InitError(format!("Failed to run ollama list: {}", e)))?;

    if !output.status.success() {
        return Err(WorkSplitError::InitError(
            "Ollama command failed to execute".to_string(),
        ));
    }

    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| WorkSplitError::InitError(format!("Failed to parse ollama output: {}", e)))?;

    let mut models = Vec::new();
    for line in stdout.lines() {
        if line.starts_with("NAME") {
            continue; // Skip header
        }
        // Extract first column (model name)
        if let Some(model_name) = line.split_whitespace().next() {
            models.push(model_name.to_string());
        }
    }

    if models.is_empty() {
        return Err(WorkSplitError::InitError(
            "No Ollama models found. Please run 'ollama pull <model>' first.".to_string(),
        ));
    }

    Ok(models)
}

/// Sort models by priority: worksplit-32k models first, then alphabetically
fn sort_models_by_priority(models: &[String]) -> Vec<String> {
    let mut worksplit_32k: Vec<String> = models
        .iter()
        .filter(|m| m.starts_with("worksplit-") && m.contains("32k"))
        .cloned()
        .collect();

    let mut others: Vec<String> = models
        .iter()
        .filter(|m| !m.starts_with("worksplit-") || !m.contains("32k"))
        .cloned()
        .collect();

    worksplit_32k.sort();
    others.sort();

    worksplit_32k.extend(others);
    worksplit_32k
}

/// Prompt user to select an Ollama model interactively
fn prompt_for_model(models: Vec<String>) -> Result<String, WorkSplitError> {
    let sorted_models = sort_models_by_priority(&models);
    let items: Vec<&str> = sorted_models.iter().map(|m| m.as_str()).collect();

    println!("\n[AI Manager: If you are an AI assistant running this init command,");
    println!("STOP here and ask the user which model to use. Do not select automatically.]\n");

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select Ollama model")
        .items(&items)
        .default(0)
        .interact()
        .map_err(|e| WorkSplitError::InitError(format!("Failed to get user input: {}", e)))?;

    Ok(sorted_models[selection].clone())
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

fn print_next_steps(project_root: &PathBuf, language: Language, model: &str) {
    println!("WorkSplit project initialized at {}", project_root.display());
    println!("\nLanguage: {}", language.display_name());
    println!("\nModel: {}", model);
    
    // Key tips box for AI managers
    println!("\n┌─────────────────────────────────────────────────────────────────┐");
    println!("│                    KEY TIPS FOR AI MANAGERS                     │");
    println!("├─────────────────────────────────────────────────────────────────┤");
    println!("│ 1. USE `worksplit new-job` to create jobs (don't write manually)│");
    println!("│    Example: worksplit new-job feat_001 --template replace \\    │");
    println!("│             -o src/ -f myfile.ts                                │");
    println!("│                                                                 │");
    println!("│ 2. RUN `worksplit validate` before `worksplit run`              │");
    println!("│    Catches job file errors before expensive LLM calls           │");
    println!("│                                                                 │");
    println!("│ 3. DON'T use WorkSplit for < 100 lines of changes               │");
    println!("│    Direct editing is faster; break-even is ~300 lines           │");
    println!("│                                                                 │");
    println!("│ 4. AVOID edit mode (low reliability ~50%)                       │");
    println!("│    Prefer replace mode or direct editing for small changes      │");
    println!("│                                                                 │");
    println!("│ 5. CHECK status efficiently: `worksplit status --summary`       │");
    println!("│    Only read generated files if status shows FAIL               │");
    println!("└─────────────────────────────────────────────────────────────────┘");
    
    println!("\nNext steps:");
    println!("1. Review jobs/_managerinstruction.md for guidance on creating job files");
    println!("2. Create job files: worksplit new-job <name> --template <type>");
    println!("3. Validate: worksplit validate");
    println!("4. Run: worksplit run");
    println!("5. Check: worksplit status -v");
    
    println!("\nSystem prompts created:");
    println!("  - _systemprompt_create.md  (code generation)");
    println!("  - _systemprompt_verify.md  (code verification)");
    println!("  - _systemprompt_edit.md    (edit mode)");
    println!("  - _systemprompt_split.md   (split mode)");
    println!("  - _systemprompt_test.md    (TDD test generation)");
    
    match language {
        Language::Rust => {
            println!("\nRust-specific tips:");
            println!("- Use .rs extension for output files");
            println!("- Build command: cargo check");
            println!("- Test command: cargo test");
        }
        Language::Solidity => {
            println!("\nSolidity/Foundry-specific tips:");
            println!("- Use .sol extension for output files");
            println!("- Place contracts in src/");
            println!("- Place tests in test/ with .t.sol extension");
            println!("- Build command: forge build");
            println!("- Test command: forge test");
        }
        Language::Typescript => {
            println!("\nTypeScript-specific tips:");
            println!("- Use .ts/.tsx extension for output files");
            println!("- Build command: npm run build");
            println!("- Test command: npm test");
            println!("- For React components: generate .tsx and .css in the SAME job");
            println!("  (ensures CSS class names match JSX classNames)");
        }
    }
    
    println!("\nTip: Add 'test_file: <filename>' to job frontmatter to enable TDD workflow");
}