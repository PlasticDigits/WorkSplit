---
context_files:
  - src/core/parser.rs
output_dir: src/core/
output_file: parser.rs
---

# Add Test Prompt Assembly Function to Parser

## Overview
Add a new function to the parser module for assembling test generation prompts.

## Requirements

### New Function

Add this function:

**`assemble_test_prompt(system_prompt: &str, context_files: &[(PathBuf, String)], instructions: &str, test_path: &str) -> String`**

This function assembles a prompt for generating tests BEFORE the implementation exists.

### Prompt Structure

The test generation prompt should have this structure:

```
[SYSTEM]
{system_prompt}

[CONTEXT]
### File: {context_file_1_path}
```
{context_file_1_content}
```

### File: {context_file_2_path}
```
{context_file_2_content}
```

[REQUIREMENTS]
{instructions}

[TEST OUTPUT]
Generate tests for: {test_path}

The implementation does not exist yet. Generate tests that will:
1. Verify the requirements are met
2. Cover edge cases
3. Be immediately runnable once implementation exists
```

### Key Differences from Creation Prompt

The test prompt differs from `assemble_creation_prompt` in:

1. Uses `[REQUIREMENTS]` instead of `[INSTRUCTIONS]` (emphasizes what to test)
2. Uses `[TEST OUTPUT]` instead of just "Output to:" 
3. Includes explicit TDD context reminding the LLM that implementation doesn't exist yet

### Implementation

Follow the same pattern as existing assemble functions:

```rust
pub fn assemble_test_prompt(
    system_prompt: &str,
    context_files: &[(std::path::PathBuf, String)],
    instructions: &str,
    test_path: &str,
) -> String {
    let mut prompt = String::new();

    // System prompt
    prompt.push_str("[SYSTEM]\n");
    prompt.push_str(system_prompt);
    prompt.push_str("\n\n");

    // Context files (same as creation prompt)
    if !context_files.is_empty() {
        prompt.push_str("[CONTEXT]\n");
        for (path, content) in context_files {
            prompt.push_str(&format!("### File: {}\n", path.display()));
            prompt.push_str("```\n");
            prompt.push_str(content);
            if !content.ends_with('\n') {
                prompt.push('\n');
            }
            prompt.push_str("```\n\n");
        }
    }

    // Requirements (what to test)
    prompt.push_str("[REQUIREMENTS]\n");
    prompt.push_str(instructions);
    prompt.push_str("\n\n");

    // Test output info with TDD context
    prompt.push_str("[TEST OUTPUT]\n");
    prompt.push_str(&format!("Generate tests for: {}\n\n", test_path));
    prompt.push_str("The implementation does not exist yet. Generate tests that will:\n");
    prompt.push_str("1. Verify the requirements are met when implementation exists\n");
    prompt.push_str("2. Cover edge cases and error conditions\n");
    prompt.push_str("3. Be immediately runnable once implementation is created\n");

    prompt
}
```

### Export

Ensure the new function is exported from the module. Check `src/core/mod.rs` to see how other parser functions are exported.

### Tests

Add tests for the new function:

1. **`test_assemble_test_prompt_basic`** - Basic prompt assembly with context files
2. **`test_assemble_test_prompt_no_context`** - Prompt without context files
3. **`test_assemble_test_prompt_sections`** - Verify all sections are present

```rust
#[test]
fn test_assemble_test_prompt_basic() {
    use std::path::PathBuf;
    
    let prompt = assemble_test_prompt(
        "You are a test generator.",
        &[(PathBuf::from("src/user.rs"), "pub struct User {}".to_string())],
        "Create a User struct with name and email fields.",
        "src/user_test.rs",
    );

    assert!(prompt.contains("[SYSTEM]"));
    assert!(prompt.contains("You are a test generator."));
    assert!(prompt.contains("[CONTEXT]"));
    assert!(prompt.contains("src/user.rs"));
    assert!(prompt.contains("[REQUIREMENTS]"));
    assert!(prompt.contains("Create a User struct"));
    assert!(prompt.contains("[TEST OUTPUT]"));
    assert!(prompt.contains("Generate tests for: src/user_test.rs"));
    assert!(prompt.contains("implementation does not exist"));
}

#[test]
fn test_assemble_test_prompt_no_context() {
    let prompt = assemble_test_prompt(
        "System prompt.",
        &[],
        "Requirements here.",
        "test.rs",
    );

    assert!(prompt.contains("[SYSTEM]"));
    assert!(!prompt.contains("[CONTEXT]")); // No context section when empty
    assert!(prompt.contains("[REQUIREMENTS]"));
    assert!(prompt.contains("[TEST OUTPUT]"));
}
```

## Implementation Notes

- Keep all existing functions unchanged
- Follow the same code style as existing `assemble_*` functions
- The prompt structure is designed to give the LLM clear context that this is TDD
