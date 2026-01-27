---
mode: edit
context_files:
  - src/models/config.rs
  - src/error.rs
target_files:
  - src/models/config.rs
  - src/error.rs
  - worksplit.toml
output_dir: src/
output_file: models/config.rs
---

# Plan3 Task 5 (Foundation): Config and Error for Build Verification

Add the necessary configuration structures and error variants for build and test verification.

## Requirements

1. Update `src/models/config.rs`:
   - Add `BuildConfig` struct with fields: `build_command`, `test_command`, `verify_build`, `verify_tests`.
   - Update `Config` struct to include `build: BuildConfig`.
   - Implement `Default` and necessary default functions for `BuildConfig`.
   - Update `Config::with_overrides` (if needed, though PLAN3 doesn't explicitly ask for CLI overrides for build yet, but it's good practice).
2. Update `src/error.rs`:
   - Add `BuildFailed` variant to `WorkSplitError`.
3. Update `worksplit.toml`:
   - Add a commented out `[build]` section as an example.

## Edit Instructions

FILE: src/models/config.rs
FIND:
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
REPLACE:
    #[serde(default)]
    pub behavior: BehaviorConfig,
    #[serde(default)]
    pub build: BuildConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ollama: OllamaConfig::default(),
            limits: LimitsConfig::default(),
            behavior: BehaviorConfig::default(),
            build: BuildConfig::default(),
        }
    }
}
END

FILE: src/models/config.rs
FIND:
fn default_create_output_dirs() -> bool {
    true
}
REPLACE:
fn default_create_output_dirs() -> bool {
    true
}

/// Build and test verification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
    /// Command to verify code compiles (optional)
    pub build_command: Option<String>,
    /// Command to run tests (optional)
    pub test_command: Option<String>,
    /// Whether to run build verification after generation
    #[serde(default = "default_verify_build")]
    pub verify_build: bool,
    /// Whether to run tests after generation
    #[serde(default = "default_verify_tests")]
    pub verify_tests: bool,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            build_command: None,
            test_command: None,
            verify_build: default_verify_build(),
            verify_tests: default_verify_tests(),
        }
    }
}

fn default_verify_build() -> bool {
    false
}

fn default_verify_tests() -> bool {
    false
}
END

FILE: src/error.rs
FIND:
    #[error("Ollama error: {0}")]
    OllamaError(String),
REPLACE:
    #[error("Ollama error: {0}")]
    OllamaError(String),

    #[error("Build failed for {command}:\n{output}")]
    BuildFailed {
        command: String,
        output: String,
    },
END

FILE: worksplit.toml
FIND:
[behavior]
REPLACE:
[build]
# build_command = "cargo check"
# test_command = "cargo test"
# verify_build = true
# verify_tests = false

[behavior]
END
