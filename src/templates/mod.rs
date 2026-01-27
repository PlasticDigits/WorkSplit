//! Language-specific templates for WorkSplit project initialization
//!
//! This module provides bundled templates for different programming languages,
//! including system prompts, configuration defaults, and example jobs.
//!
//! Templates are stored as external markdown files in the `templates/` directory
//! and embedded at compile time using `include_str!`.

pub mod rust;
pub mod solidity;
pub mod typescript;

use crate::models::Language;

/// Template content for a specific language
pub struct Templates {
    /// System prompt for code generation (replace mode)
    pub create_prompt: &'static str,
    /// System prompt for code verification
    pub verify_prompt: &'static str,
    /// System prompt for edit mode
    pub edit_prompt: &'static str,
    /// System prompt for edit mode verification
    pub verify_edit_prompt: &'static str,
    /// System prompt for split mode
    pub split_prompt: &'static str,
    /// System prompt for test generation (TDD)
    pub test_prompt: &'static str,
    /// Manager instructions for creating jobs
    pub manager_instruction: &'static str,
    /// Default configuration content
    pub config: &'static str,
    /// Example job file content
    pub example_job: &'static str,
    /// TDD example job file content
    pub tdd_example_job: &'static str,
}

/// Get templates for the specified language
pub fn get_templates(language: Language) -> Templates {
    match language {
        Language::Rust => rust::templates(),
        Language::Solidity => solidity::templates(),
        Language::Typescript => typescript::templates(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_rust_templates() {
        let templates = get_templates(Language::Rust);
        assert!(templates.create_prompt.contains("Rust"));
        assert!(templates.verify_prompt.contains("PASS"));
        assert!(templates.edit_prompt.contains("FIND"));
        assert!(templates.verify_edit_prompt.contains("PASS"));
        assert!(templates.split_prompt.contains("mod.rs"));
        assert!(templates.config.contains("cargo"));
    }

    #[test]
    fn test_get_solidity_templates() {
        let templates = get_templates(Language::Solidity);
        assert!(templates.create_prompt.contains("Solidity"));
        assert!(templates.verify_prompt.contains("PASS"));
        assert!(templates.edit_prompt.contains("FIND"));
        assert!(templates.verify_edit_prompt.contains("PASS"));
        assert!(templates.split_prompt.contains("abstract"));
        assert!(templates.config.contains("forge"));
    }

    #[test]
    fn test_get_typescript_templates() {
        let templates = get_templates(Language::Typescript);
        assert!(templates.create_prompt.contains("TypeScript"));
        assert!(templates.verify_prompt.contains("PASS"));
        assert!(templates.edit_prompt.contains("FIND"));
        assert!(templates.verify_edit_prompt.contains("PASS"));
        assert!(templates.split_prompt.contains("index.ts"));
        assert!(templates.config.contains("npm"));
    }
}
