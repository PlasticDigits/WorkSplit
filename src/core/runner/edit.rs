use std::fs;
use std::path::{Path, PathBuf};

use crate::core::{
    assemble_edit_prompt, parse_edit_instructions, apply_edits,
    OllamaClient, EditInstruction, SYSTEM_PROMPT_EDIT,
};
use crate::error::WorkSplitError;
use crate::models::{Config, Job};

/// Process edit mode job
pub(crate) async fn process_edit_mode(
    ollama: &OllamaClient,
    project_root: &Path,
    config: &Config,
    job: &Job,
    context_files: &[(PathBuf, String)],
    edit_prompt: &str,
) -> Result<(Vec<(PathBuf, String)>, Vec<PathBuf>, usize), WorkSplitError> {
    let target_files = job.metadata.get_target_files();
    let mut target_file_contents: Vec<(PathBuf, String)> = Vec::new();
    for path in &target_files {
        let content = fs::read_to_string(project_root.join(path))?;
        target_file_contents.push((path.clone(), content));
    }
    
    let prompt = assemble_edit_prompt(edit_prompt, &target_file_contents, context_files, &job.instructions);
    let response = ollama.generate_with_retry(Some(SYSTEM_PROMPT_EDIT), &prompt, config.behavior.stream_output)
        .await
        .map_err(|e| { WorkSplitError::Ollama(e) })?;
    
    let parsed_edits = parse_edit_instructions(&response);
    let mut generated_files: Vec<(PathBuf, String)> = Vec::new();
    let mut full_output_paths: Vec<PathBuf> = Vec::new();
    let mut total_lines = 0;
    
    for (path, original_content) in &target_file_contents {
        let file_edits: Vec<&EditInstruction> = parsed_edits.edits_for_file(path);
        if file_edits.is_empty() { continue; }
        let edited = apply_edits(original_content, &file_edits)
            .map_err(|e| { WorkSplitError::EditFailed(e) })?;
        total_lines += crate::core::count_lines(&edited);
        let full_path = project_root.join(path);
        fs::write(&full_path, &edited)?;
        generated_files.push((path.clone(), edited));
        full_output_paths.push(full_path);
    }
    
    if generated_files.is_empty() {
        return Err(WorkSplitError::EditFailed("Edit mode produced no edits".to_string()));
    }
    
    Ok((generated_files, full_output_paths, total_lines))
}