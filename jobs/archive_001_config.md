---
context_files:
  - src/models/config.rs
output_dir: src/models/
output_file: config.rs
---

# Add Archive and Cleanup Configuration

Regenerate config.rs with new ArchiveConfig and CleanupConfig structs added.

## Current File Context

The current config.rs defines Config, OllamaConfig, LimitsConfig, BehaviorConfig, and BuildConfig.
You must preserve ALL existing code exactly and ADD the new configuration structs.

## New Structs to Add

### 1. ArchiveConfig (add after BuildConfig)

```rust
/// Archive configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveConfig {
    /// Whether auto-archive is enabled
    #[serde(default = "default_archive_enabled")]
    pub enabled: bool,
    /// Archive jobs older than this many days
    #[serde(default = "default_archive_days")]
    pub days: u32,
}

impl Default for ArchiveConfig {
    fn default() -> Self {
        Self {
            enabled: default_archive_enabled(),
            days: default_archive_days(),
        }
    }
}

fn default_archive_enabled() -> bool {
    true
}

fn default_archive_days() -> u32 {
    3
}
```

### 2. CleanupConfig (add after ArchiveConfig)

```rust
/// Cleanup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupConfig {
    /// Whether auto-cleanup is enabled
    #[serde(default = "default_cleanup_enabled")]
    pub enabled: bool,
    /// Delete archived jobs older than this many days
    #[serde(default = "default_cleanup_days")]
    pub days: u32,
}

impl Default for CleanupConfig {
    fn default() -> Self {
        Self {
            enabled: default_cleanup_enabled(),
            days: default_cleanup_days(),
        }
    }
}

fn default_cleanup_enabled() -> bool {
    true
}

fn default_cleanup_days() -> u32 {
    30
}
```

## Changes to Existing Code

### Update Config struct

Add these fields:
```rust
#[serde(default)]
pub archive: ArchiveConfig,
#[serde(default)]
pub cleanup: CleanupConfig,
```

### Update Config::default()

Add:
```rust
archive: ArchiveConfig::default(),
cleanup: CleanupConfig::default(),
```

### Add New Tests

Add these tests to the existing test module:

```rust
#[test]
fn test_default_archive_config() {
    let config = Config::default();
    assert!(config.archive.enabled);
    assert_eq!(config.archive.days, 3);
}

#[test]
fn test_default_cleanup_config() {
    let config = Config::default();
    assert!(config.cleanup.enabled);
    assert_eq!(config.cleanup.days, 30);
}

#[test]
fn test_parse_toml_with_archive_cleanup() {
    let toml_str = r#"
[project]
language = "rust"

[archive]
enabled = false
days = 7

[cleanup]
enabled = false
days = 60
"#;
    let config: Config = toml::from_str(toml_str).unwrap();
    assert!(!config.archive.enabled);
    assert_eq!(config.archive.days, 7);
    assert!(!config.cleanup.enabled);
    assert_eq!(config.cleanup.days, 60);
}
```

## Critical Requirements

1. Preserve ALL existing code exactly - do not remove or modify existing structs/functions/tests
2. Add the new structs BEFORE the `impl Config` block
3. Add the new default functions BEFORE the `impl Config` block
4. The file must compile with `cargo check`
