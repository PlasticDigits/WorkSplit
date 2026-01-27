use std::fs;
use std::path::{Path, PathBuf};

use crate::core::{
    assemble_edit_prompt, parse_edit_instructions, apply_edit, find_fuzzy_match,
    OllamaClient, EditInstruction, SYSTEM_PROMPT_EDIT,
};
use crate::error::WorkSplitError;
use crate::models::{Config, Job};
use crate::models::status::PartialEditState;

/// Result of a dry-run edit analysis
#[derive(Debug, Clone)]
pub struct DryRunResult {
    pub job_id: String,
    pub planned_edits: Vec<PlannedEdit>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PlannedEdit {
    pub file_path: PathBuf,
    pub line_number: Option<usize>,
    pub find_preview: String,
    pub replace_preview: String,
    pub status: PlannedEditStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlannedEditStatus {
    WillApply,
    WillApplyFuzzy,
    WillFail,
}

/// Result of edit mode processing
#[derive(Debug)]
pub struct EditModeResult {
    pub generated_files: Vec<(PathBuf, String)>,
    pub output_paths: Vec<PathBuf>,
    pub total_lines: usize,
    pub partial_state: Option<PartialEditState>,
    pub suggestions: Vec<String>,
}

/// Analyze what edits would be applied without actually applying them
pub(crate) async fn dry_run_edit_mode(
    ollama: &OllamaClient,
    project_root: &Path,
    config: &Config,
    job: &Job,
    context_files: &[(PathBuf, String)],
    edit_prompt: &str,
) -> Result<DryRunResult, WorkSplitError> {
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
    let mut planned_edits = Vec::new();
    let mut warnings = Vec::new();
    
    for edit in &parsed_edits.edits {
        let file_path = project_root.join(&edit.file_path);
        let content = fs::read_to_string(&file_path)
            .unwrap_or_else(|_| String::new());
        
        let line_number = None;
        
        // Try exact match
        if content.contains(&edit.find) {
            let find_preview = edit.find.chars().take(50).collect::<String>();
            let replace_preview = edit.replace.chars().take(50).collect::<String>();
            planned_edits.push(PlannedEdit {
                file_path: edit.file_path.clone(),
                line_number,
                find_preview,
                replace_preview,
                status: PlannedEditStatus::WillApply,
            });
        }
        // Try fuzzy match
        else if let Some((start, _end, _matched)) = find_fuzzy_match(&content, &edit.find) {
            let find_preview = edit.find.chars().take(50).collect::<String>();
            let replace_preview = edit.replace.chars().take(50).collect::<String>();
            planned_edits.push(PlannedEdit {
                file_path: edit.file_path.clone(),
                line_number: Some(start),
                find_preview,
                replace_preview,
                status: PlannedEditStatus::WillApplyFuzzy,
            });
            warnings.push(format!(
                "Fuzzy match for {} at approximate position {}",
                edit.file_path.display(),
                start
            ));
        }
    }
    
    Ok(DryRunResult {
        job_id: job.id.clone(),
        planned_edits,
        warnings,
    })
}

impl std::fmt::Display for DryRunResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[DRY RUN] Job: {}", self.job_id)?;
        writeln!(f, "Planned edits: {}", self.planned_edits.len())?;
        
        // Group by file
        use std::collections::BTreeMap;
        let mut by_file: BTreeMap<PathBuf, Vec<&PlannedEdit>> = BTreeMap::new();
        for edit in &self.planned_edits {
            by_file.entry(edit.file_path.clone()).or_default().push(edit);
        }
        
        for (file_path, edits) in &by_file {
            writeln!(f, "\n  File: {}", file_path.display())?;
            for edit in edits {
                let status_str = match edit.status {
                    PlannedEditStatus::WillApply => "✓",
                    PlannedEditStatus::WillApplyFuzzy => "~",
                    PlannedEditStatus::WillFail => "✗",
                };
                let line_hint = edit.line_number.map_or(String::new(), |l| format!(" (line {})", l));
                writeln!(f, "    {} {}{}...", status_str, edit.find_preview, line_hint)?;
            }
        }
        
        if !self.warnings.is_empty() {
            writeln!(f, "\nWarnings:")?;
            for warning in &self.warnings {
                writeln!(f, "  - {}", warning)?;
            }
        }
        
        Ok(())
    }
}

/// Process edit mode job
pub(crate) async fn process_edit_mode(
    ollama: &OllamaClient,
    project_root: &Path,
    config: &Config,
    job: &Job,
    context_files: &[(PathBuf, String)],
    edit_prompt: &str,
    _dry_run: bool,
) -> Result<EditModeResult, WorkSplitError> {
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
    
    let mut partial_state = PartialEditState::new();
    let mut failed_edits = Vec::new();
    
    for (path, original_content) in &target_file_contents {
        let file_edits: Vec<&EditInstruction> = parsed_edits.edits_for_file(path);
        if file_edits.is_empty() { continue; }
        
        let mut current_content = original_content.clone();
        let mut file_edits_applied = 0;
        
        for edit in &file_edits {
            let result = apply_edit(&current_content, edit);
            match result {
                Ok(edited) => {
                    current_content = edited;
                    file_edits_applied += 1;
                }
                Err(e) => {
                    // Collect failed edit with fuzzy match hint
                    let find_preview = edit.find.chars().take(50).collect::<String>();
                    let fuzzy_hint = if let Some((start, _end, _matched)) = find_fuzzy_match(&current_content, &edit.find) {
                        Some(start)
                    } else {
                        None
                    };
                    
                    failed_edits.push(FailedEdit {
                        file_path: edit.file_path.clone(),
                        find: edit.find.clone(),
                        find_preview,
                        reason: e,
                        suggested_line: fuzzy_hint,
                    });
                    
                    // Add to partial state
                    partial_state.add_failed_edit(edit.file_path.display().to_string(), edit.find.clone());
                }
            }
        }
        
        if file_edits_applied > 0 {
            total_lines += crate::core::count_lines(&current_content);
            let full_path = project_root.join(path);
            fs::write(&full_path, &current_content)?;
            generated_files.push((path.clone(), current_content));
            full_output_paths.push(full_path);
        }
    }
    
    if generated_files.is_empty() {
        return Err(WorkSplitError::EditFailed("Edit mode produced no edits".to_string()));
    }
    
    let suggestions = generate_suggestions(&failed_edits, parsed_edits.edits.len());
    
    Ok(EditModeResult {
        generated_files,
        output_paths: full_output_paths,
        total_lines,
        partial_state: if failed_edits.is_empty() {
            None
        } else {
            Some(partial_state)
        },
        suggestions,
    })
}

/// Failed edit with fuzzy match hint
struct FailedEdit {
    file_path: PathBuf,
    find: String,
    find_preview: String,
    reason: String,
    suggested_line: Option<usize>,
}

/// Generate actionable suggestions from failed edits
fn generate_suggestions(
    failed_edits: &[FailedEdit],
    edit_count: usize,
) -> Vec<String> {
    let mut suggestions = Vec::new();
    
    // Check for whitespace issues
    if failed_edits.iter().any(|e| e.reason.contains("whitespace")) {
        suggestions.push("Check whitespace: file may use different indentation".to_string());
    }
    
    // Check for too many edits
    if edit_count > 10 {
        suggestions.push(format!(
            "Consider replace mode: this job has {}+ edits, replace is safer",
            edit_count
        ));
    }
    
    // Add line hints
    for edit in failed_edits {
        if let Some(line) = edit.suggested_line {
            suggestions.push(format!(
                "For '{}...': check line {} in {}",
                &edit.find_preview[..20.min(edit.find_preview.len())],
                line,
                edit.file_path.display()
            ));
        }
    }
    
    suggestions
}