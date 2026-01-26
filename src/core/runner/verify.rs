use std::path::PathBuf;
use tracing::info;

use crate::core::{
    assemble_verification_prompt_multi, assemble_retry_prompt_multi, extract_code_files,
    parse_verification, OllamaClient, VerificationResult,
    SYSTEM_PROMPT_VERIFY, SYSTEM_PROMPT_RETRY,
};
use crate::error::WorkSplitError;

/// Run verification on generated files
pub(crate) async fn run_verification(
    ollama: &OllamaClient,
    verify_prompt: &str,
    context_files: &[(PathBuf, String)],
    generated_files: &[(PathBuf, String)],
    instructions: &str,
) -> Result<(VerificationResult, Option<String>), WorkSplitError> {
    let file_names: Vec<_> = generated_files.iter()
        .map(|(p, _)| p.display().to_string())
        .collect();
    info!("Starting verification of {} file(s): {:?}", generated_files.len(), file_names);
    
    let verify_prompt_str = assemble_verification_prompt_multi(verify_prompt, context_files,
        generated_files, instructions);
    
    info!("Verification prompt size: {} chars", verify_prompt_str.len());
    
    let verify_response = ollama.generate(Some(SYSTEM_PROMPT_VERIFY), &verify_prompt_str, false)
        .await
        .map_err(|e| { WorkSplitError::Ollama(e) })?;
    
    info!("Verification response received: {} chars", verify_response.len());
    
    let (result, error) = parse_verification(&verify_response);
    info!("Verification result: {:?}", result);
    Ok((result, error))
}

/// Run retry logic for failed verification
pub(crate) async fn run_retry(
    ollama: &OllamaClient,
    create_prompt: &str,
    context_files: &[(PathBuf, String)],
    generated_files: &[(PathBuf, String)],
    instructions: &str,
    error_msg: &str,
) -> Result<Vec<(PathBuf, String)>, WorkSplitError> {
    let retry_prompt = assemble_retry_prompt_multi(create_prompt, context_files,
        instructions, generated_files, error_msg);
    let retry_response = ollama.generate(Some(SYSTEM_PROMPT_RETRY), &retry_prompt, true)
        .await
        .map_err(|e| { WorkSplitError::Ollama(e) })?;
    
    let mut retry_files: Vec<(PathBuf, String)> = Vec::new();
    for file in extract_code_files(&retry_response) {
        let path = file.path.clone();
        if let Some(p) = path {
            retry_files.push((p, file.content.clone()));
        }
    }
    
    Ok(retry_files)
}