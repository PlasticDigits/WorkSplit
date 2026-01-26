use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration loaded from worksplit.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub ollama: OllamaConfig,
    #[serde(default)]
    pub limits: LimitsConfig,
    #[serde(default)]
    pub behavior: BehaviorConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ollama: OllamaConfig::default(),
            limits: LimitsConfig::default(),
            behavior: BehaviorConfig::default(),
        }
    }
}

/// Ollama API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    /// Ollama API URL
    #[serde(default = "default_ollama_url")]
    pub url: String,
    /// Model name to use
    #[serde(default = "default_model")]
    pub model: String,
    /// Timeout in seconds for API requests
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            url: default_ollama_url(),
            model: default_model(),
            timeout_seconds: default_timeout(),
        }
    }
}

fn default_ollama_url() -> String {
    "http://localhost:11434".to_string()
}

fn default_model() -> String {
    "qwen-32k:latest".to_string()
}

fn default_timeout() -> u64 {
    300
}

/// Limits configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitsConfig {
    /// Maximum lines of code in output
    #[serde(default = "default_max_output_lines")]
    pub max_output_lines: usize,
    /// Maximum lines of code per context file
    #[serde(default = "default_max_context_lines")]
    pub max_context_lines: usize,
    /// Maximum number of context files
    #[serde(default = "default_max_context_files")]
    pub max_context_files: usize,
}

impl Default for LimitsConfig {
    fn default() -> Self {
        Self {
            max_output_lines: default_max_output_lines(),
            max_context_lines: default_max_context_lines(),
            max_context_files: default_max_context_files(),
        }
    }
}

fn default_max_output_lines() -> usize {
    900
}

fn default_max_context_lines() -> usize {
    1000
}

fn default_max_context_files() -> usize {
    2
}

/// Behavior configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorConfig {
    /// Show streaming output in terminal
    #[serde(default = "default_stream_output")]
    pub stream_output: bool,
    /// Create output directories if missing
    #[serde(default = "default_create_output_dirs")]
    pub create_output_dirs: bool,
}

impl Default for BehaviorConfig {
    fn default() -> Self {
        Self {
            stream_output: default_stream_output(),
            create_output_dirs: default_create_output_dirs(),
        }
    }
}

fn default_stream_output() -> bool {
    true
}

fn default_create_output_dirs() -> bool {
    true
}

impl Config {
    /// Load config from a TOML file
    pub fn load_from_file(path: &PathBuf) -> Result<Self, ConfigError> {
        let contents = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::ReadError(path.clone(), e))?;
        toml::from_str(&contents)
            .map_err(|e| ConfigError::ParseError(path.clone(), e))
    }

    /// Try to load config from worksplit.toml in the given directory
    pub fn load_from_dir(dir: &PathBuf) -> Result<Self, ConfigError> {
        let config_path = dir.join("worksplit.toml");
        if config_path.exists() {
            Self::load_from_file(&config_path)
        } else {
            Ok(Self::default())
        }
    }

    /// Merge CLI overrides into the config
    pub fn with_overrides(
        mut self,
        model: Option<String>,
        url: Option<String>,
        timeout: Option<u64>,
        no_stream: bool,
    ) -> Self {
        if let Some(m) = model {
            self.ollama.model = m;
        }
        if let Some(u) = url {
            self.ollama.url = u;
        }
        if let Some(t) = timeout {
            self.ollama.timeout_seconds = t;
        }
        if no_stream {
            self.behavior.stream_output = false;
        }
        self
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Failed to read config file {0}: {1}")]
    ReadError(PathBuf, std::io::Error),
    #[error("Failed to parse config file {0}: {1}")]
    ParseError(PathBuf, toml::de::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.ollama.url, "http://localhost:11434");
        assert_eq!(config.ollama.model, "qwen-32k:latest");
        assert_eq!(config.ollama.timeout_seconds, 300);
        assert_eq!(config.limits.max_output_lines, 900);
        assert_eq!(config.limits.max_context_lines, 1000);
        assert_eq!(config.limits.max_context_files, 2);
        assert!(config.behavior.stream_output);
        assert!(config.behavior.create_output_dirs);
    }

    #[test]
    fn test_config_with_overrides() {
        let config = Config::default()
            .with_overrides(
                Some("llama3".to_string()),
                Some("http://remote:11434".to_string()),
                Some(600),
                true,
            );
        assert_eq!(config.ollama.model, "llama3");
        assert_eq!(config.ollama.url, "http://remote:11434");
        assert_eq!(config.ollama.timeout_seconds, 600);
        assert!(!config.behavior.stream_output);
    }

    #[test]
    fn test_parse_toml() {
        let toml_str = r#"
[ollama]
url = "http://custom:8080"
model = "codellama"
timeout_seconds = 120

[limits]
max_output_lines = 500

[behavior]
stream_output = false
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.ollama.url, "http://custom:8080");
        assert_eq!(config.ollama.model, "codellama");
        assert_eq!(config.ollama.timeout_seconds, 120);
        assert_eq!(config.limits.max_output_lines, 500);
        assert_eq!(config.limits.max_context_lines, 1000); // default
        assert!(!config.behavior.stream_output);
    }
}
