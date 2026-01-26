---
mode: edit
target_files:
  - src/core/parser.rs
output_dir: src/core/
output_file: parser.rs
---

# Add Tests for Path-as-Heading Extraction

## Overview

Add unit tests to verify the new path-as-heading extraction pattern works correctly.

## Formatting Notes

- Uses 4-space indentation
- Tests are in `mod tests` at the bottom of the file
- Test functions use `#[test]` attribute
- Use `assert_eq!` and `assert!` macros

## Edit Location

In the `mod tests` section, after the existing `extract_code` tests. Look for tests like `test_extract_code_files_with_path` or similar.

Find the last test function in the module and add new tests after it.

## Tests to Add

### test_extract_path_as_heading_single

```rust
#[test]
fn test_extract_path_as_heading_single() {
    let response = r#"Here is the code:

src/main.rs
```rust
fn main() {
    println!("Hello");
}
```

Done."#;
    let files = extract_code_files(response);
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].path, Some(PathBuf::from("src/main.rs")));
    assert!(files[0].content.contains("fn main()"));
}
```

### test_extract_path_as_heading_multiple

```rust
#[test]
fn test_extract_path_as_heading_multiple() {
    let response = r#"Generated files:

src/lib.rs
```rust
pub mod utils;
```

src/utils.rs
```rust
pub fn helper() -> i32 {
    42
}
```

All done."#;
    let files = extract_code_files(response);
    assert_eq!(files.len(), 2);
    assert_eq!(files[0].path, Some(PathBuf::from("src/lib.rs")));
    assert_eq!(files[1].path, Some(PathBuf::from("src/utils.rs")));
}
```

### test_extract_path_as_heading_without_language

```rust
#[test]
fn test_extract_path_as_heading_without_language() {
    let response = r#"
config.toml
```
[package]
name = "test"
```
"#;
    let files = extract_code_files(response);
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].path, Some(PathBuf::from("config.toml")));
}
```

### test_worksplit_preferred_over_path_heading

```rust
#[test]
fn test_worksplit_preferred_over_path_heading() {
    // When both formats are present, worksplit should win
    let response = r#"
~~~worksplit:src/preferred.rs
fn preferred() {}
~~~worksplit

src/ignored.rs
```rust
fn ignored() {}
```
"#;
    let files = extract_code_files(response);
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].path, Some(PathBuf::from("src/preferred.rs")));
    assert!(files[0].content.contains("preferred"));
}
```

## FIND/REPLACE Approach

Find the last test function in the tests module (likely ends with a closing brace `}` followed by the module closing brace), then add the new tests before the final module closing brace.
