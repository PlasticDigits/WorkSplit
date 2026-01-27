//! Language enumeration for multi-language project support

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Supported programming languages for WorkSplit projects
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    /// Rust programming language
    Rust,
    /// TypeScript programming language
    Typescript,
}

impl Language {
    /// Returns the display name for the language
    pub fn display_name(&self) -> &'static str {
        match self {
            Language::Rust => "Rust",
            Language::Typescript => "TypeScript",
        }
    }

    /// Returns the file extension for source files
    pub fn file_extension(&self) -> &'static str {
        match self {
            Language::Rust => "rs",
            Language::Typescript => "ts",
        }
    }

    /// Returns all available languages
    pub fn all() -> &'static [Language] {
        &[Language::Rust, Language::Typescript]
    }
}

impl Default for Language {
    fn default() -> Self {
        Language::Rust
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_display_name() {
        assert_eq!(Language::Rust.display_name(), "Rust");
        assert_eq!(Language::Typescript.display_name(), "TypeScript");
    }

    #[test]
    fn test_language_file_extension() {
        assert_eq!(Language::Rust.file_extension(), "rs");
        assert_eq!(Language::Typescript.file_extension(), "ts");
    }

    #[test]
    fn test_language_default() {
        assert_eq!(Language::default(), Language::Rust);
    }

    #[test]
    fn test_language_serialization() {
        let rust = Language::Rust;
        let json = serde_json::to_string(&rust).unwrap();
        assert_eq!(json, "\"rust\"");

        let ts = Language::Typescript;
        let json = serde_json::to_string(&ts).unwrap();
        assert_eq!(json, "\"typescript\"");
    }

    #[test]
    fn test_language_deserialization() {
        let rust: Language = serde_json::from_str("\"rust\"").unwrap();
        assert_eq!(rust, Language::Rust);

        let ts: Language = serde_json::from_str("\"typescript\"").unwrap();
        assert_eq!(ts, Language::Typescript);
    }
}
