use std::fs;
use std::path::{Path, PathBuf};

use crate::core::{
    assemble_sequential_creation_prompt, extract_code, extract_code_files, count_lines,
    OllamaClient, SYSTEM_PROMPT_CREATE,
};
use crate::error::WorkSplitError;
use crate::models::{Config, Job};

/// Process sequential mode job
pub(crate) async fn process_sequential_mode(
    ollama: &OllamaClient,
    project_root: &Path,
    config: &Config,
    job: &Job,
    context_files: &[(PathBuf, String)],
    create_prompt: &str,
) -> Result<(Vec<(PathBuf, String)>, Vec<PathBuf>, usize), WorkSplitError> {
    let output_files = job.metadata.get_output_files();
    let mut previously_generated: Vec<(PathBuf, String)> = Vec::new();
    let mut generated_files: Vec<(PathBuf, String)> = Vec::new();
    let mut full_output_paths: Vec<PathBuf> = Vec::new();
    let mut total_lines = 0;

    for (idx, output_path) in output_files.iter().enumerate() {
        let remaining: Vec<PathBuf> = output_files[idx + 1..].to_vec();
        let prompt = assemble_sequential_creation_prompt(create_prompt, context_files,
            &previously_generated, &job.instructions, &output_path.display().to_string(), &remaining);
        
        let response = ollama.generate_with_retry(Some(SYSTEM_PROMPT_CREATE), &prompt, config.behavior.stream_output)
            .await
            .map_err(|e| { WorkSplitError::Ollama(e) })?;
        
        let extracted = extract_code_files(&response);
        let content = if extracted.is_empty() { 
            extract_code(&response) 
        } else { 
            extracted[0].content.clone() 
        };
        total_lines += count_lines(&content);
        
        let full_path = project_root.join(output_path);
        if let Some(parent) = full_path.parent() {
            if !parent.exists() && config.behavior.create_output_dirs { 
                fs::create_dir_all(parent)?; 
            }
        }
        fs::write(&full_path, &content)?;
        
        previously_generated.push((output_path.clone(), content.clone()));
        generated_files.push((output_path.clone(), content));
        full_output_paths.push(full_path);
    }

    Ok((generated_files, full_output_paths, total_lines))
}