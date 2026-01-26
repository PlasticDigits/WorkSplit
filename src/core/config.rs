use std::path::PathBuf;
use tracing::info;

use crate::error::WorkSplitError;
use crate::models::Config;

/// Load configuration from project directory with CLI overrides
pub fn load_config(
    project_root: &PathBuf,
    model: Option<String>,
    url: Option<String>,
    timeout: Option<u64>,
    no_stream: bool,
) -> Result<Config, WorkSplitError> {
    let config = Config::load_from_dir(project_root)?;
    let config = config.with_overrides(model, url, timeout, no_stream);

    info!(
        "Configuration loaded: model={}, url={}, timeout={}s",
        config.ollama.model, config.ollama.url, config.ollama.timeout_seconds
    );

    Ok(config)
}

/// Find the project root by looking for a jobs folder
pub fn find_project_root() -> Result<PathBuf, WorkSplitError> {
    let current_dir = std::env::current_dir()?;

    // Check if jobs folder exists in current directory
    if current_dir.join("jobs").is_dir() {
        return Ok(current_dir);
    }

    // Could also walk up the directory tree, but for now just use current dir
    Err(WorkSplitError::JobsFolderNotFound(current_dir.join("jobs")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_load_config_default() {
        let temp_dir = TempDir::new().unwrap();
        let config = load_config(
            &temp_dir.path().to_path_buf(),
            None,
            None,
            None,
            false,
        ).unwrap();

        assert_eq!(config.ollama.model, "qwen-32k:latest");
        assert_eq!(config.ollama.url, "http://localhost:11434");
    }

    #[test]
    fn test_load_config_with_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("worksplit.toml");
        
        fs::write(&config_path, r#"
[ollama]
model = "llama3"
url = "http://custom:8080"
"#).unwrap();

        let config = load_config(
            &temp_dir.path().to_path_buf(),
            None,
            None,
            None,
            false,
        ).unwrap();

        assert_eq!(config.ollama.model, "llama3");
        assert_eq!(config.ollama.url, "http://custom:8080");
    }

    #[test]
    fn test_load_config_with_overrides() {
        let temp_dir = TempDir::new().unwrap();
        let config = load_config(
            &temp_dir.path().to_path_buf(),
            Some("codellama".to_string()),
            Some("http://remote:11434".to_string()),
            Some(600),
            true,
        ).unwrap();

        assert_eq!(config.ollama.model, "codellama");
        assert_eq!(config.ollama.url, "http://remote:11434");
        assert_eq!(config.ollama.timeout_seconds, 600);
        assert!(!config.behavior.stream_output);
    }
}
