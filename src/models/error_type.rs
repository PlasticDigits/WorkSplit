use serde::{Deserialize, Serialize};
use std::fmt;

/// Type of error that triggered auto-fix
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorType {
    /// Build/compilation error
    Build,
    /// Test failure
    Test,
    /// Linter error
    Lint,
}

impl ErrorType {
    /// Get the human-readable name for this error type
    pub fn name(&self) -> &'static str {
        match self {
            ErrorType::Build => "Build",
            ErrorType::Test => "Test",
            ErrorType::Lint => "Lint",
        }
    }

    /// Get the lowercase name for this error type
    pub fn lowercase_name(&self) -> &'static str {
        match self {
            ErrorType::Build => "build",
            ErrorType::Test => "test",
            ErrorType::Lint => "lint",
        }
    }

    /// Format the error section header for the LLM prompt
    pub fn prompt_header(&self) -> &'static str {
        match self {
            ErrorType::Build => "## Build/Compilation Errors",
            ErrorType::Test => "## Test Failures",
            ErrorType::Lint => "## Linter Errors",
        }
    }

    /// Get instructions specific to this error type
    pub fn fix_instructions(&self) -> &'static str {
        match self {
            ErrorType::Build => "Fix the compilation errors. Focus on type mismatches, missing imports, and syntax errors.",
            ErrorType::Test => "Fix the test failures. Focus on assertion logic, test setup, and expected values.",
            ErrorType::Lint => "Fix the linter errors. Focus on unused variables, missing type annotations, and style issues.",
        }
    }
}

impl fmt::Display for ErrorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_type_names() {
        assert_eq!(ErrorType::Build.name(), "Build");
        assert_eq!(ErrorType::Test.name(), "Test");
        assert_eq!(ErrorType::Lint.name(), "Lint");
    }

    #[test]
    fn test_error_type_lowercase_names() {
        assert_eq!(ErrorType::Build.lowercase_name(), "build");
        assert_eq!(ErrorType::Test.lowercase_name(), "test");
        assert_eq!(ErrorType::Lint.lowercase_name(), "lint");
    }

    #[test]
    fn test_error_type_display() {
        assert_eq!(format!("{}", ErrorType::Build), "Build");
        assert_eq!(format!("{}", ErrorType::Test), "Test");
        assert_eq!(format!("{}", ErrorType::Lint), "Lint");
    }
}
