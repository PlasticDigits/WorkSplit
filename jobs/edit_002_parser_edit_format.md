---
context_files: []
output_dir: src/core/
output_file: parser_edit.rs
---

# Add Edit Instruction Parser (New Module)

## Overview
Create a new module `parser_edit.rs` with parsing support for a custom edit format that allows surgical file modifications. This is a separate module to avoid making parser.rs too large.

## Problem
When LLMs generate diffs, they often get line numbers wrong or have whitespace issues. We need a more robust format that:
1. Uses exact text matching (like FIND/REPLACE)
2. Clearly delimits edit boundaries
3. Supports multiple edits per file
4. Is easy for LLMs to generate correctly

## Solution
Create a new module `parser_edit.rs` with a custom edit format parser:

```
FILE: src/main.rs
FIND:
        no_stream: bool,
    },
REPLACE:
        no_stream: bool,
        /// Stop processing when any job fails
        #[arg(long)]
        stop_on_fail: bool,
    },
END

FILE: src/commands/run.rs
FIND:
    pub no_stream: bool,
}
REPLACE:
    pub no_stream: bool,
    pub stop_on_fail: bool,
}
END
```

## Requirements

### 1. Define Edit Instruction Struct
```rust
/// A single edit instruction for a file
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditInstruction {
    /// File path to edit
    pub file_path: PathBuf,
    /// Text to find (exact match)
    pub find: String,
    /// Text to replace with
    pub replace: String,
}

/// Parsed edits for multiple files
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedEdits {
    /// List of edit instructions
    pub edits: Vec<EditInstruction>,
    /// Files that have edits
    pub affected_files: Vec<PathBuf>,
}

impl ParsedEdits {
    /// Get all edits for a specific file
    pub fn edits_for_file(&self, path: &PathBuf) -> Vec<&EditInstruction> {
        self.edits.iter().filter(|e| &e.file_path == path).collect()
    }
}
```

### 2. Implement Edit Parser Function
```rust
/// Parse edit instructions from LLM response
/// 
/// Format:
/// ```
/// FILE: path/to/file.rs
/// FIND:
/// <exact text to find>
/// REPLACE:
/// <replacement text>
/// END
/// ```
/// 
/// Multiple FILE blocks can be present for different files.
/// Multiple FIND/REPLACE/END blocks can be under a single FILE.
pub fn parse_edit_instructions(response: &str) -> ParsedEdits {
    // Implementation
}
```

### 3. Parser Implementation Details
The parser should:
1. Find `FILE:` markers to identify which file each edit applies to
2. Parse FIND/REPLACE/END blocks within each file section
3. Handle multiple edits per file
4. Preserve exact whitespace in FIND and REPLACE blocks
5. Support both `~~~worksplit` wrappers and raw format
6. Be case-insensitive for keywords (FILE, FIND, REPLACE, END)

### 4. Handle Edge Cases
- Empty FIND (should error or skip)
- Empty REPLACE (valid - deletes the found text)
- No FILE marker (use first/default target file)
- Nested or malformed blocks
- FIND text not found in target file (will be handled by runner, but note in result)

### 5. Add Utility Function
```rust
/// Apply a single edit to file content
/// Returns Ok(new_content) if successful, Err(reason) if FIND text not found
pub fn apply_edit(content: &str, edit: &EditInstruction) -> Result<String, String> {
    if !content.contains(&edit.find) {
        return Err(format!(
            "FIND text not found in {}: {:?}",
            edit.file_path.display(),
            edit.find.chars().take(50).collect::<String>()
        ));
    }
    Ok(content.replacen(&edit.find, &edit.replace, 1))
}

/// Apply multiple edits to file content in order
pub fn apply_edits(content: &str, edits: &[&EditInstruction]) -> Result<String, String> {
    let mut result = content.to_string();
    for edit in edits {
        result = apply_edit(&result, edit)?;
    }
    Ok(result)
}
```

### 6. Add Tests
Test cases:
- Single file, single edit
- Single file, multiple edits
- Multiple files
- Empty REPLACE (deletion)
- Whitespace preservation
- Case-insensitive keywords
- Missing FIND text error
- Malformed blocks
- Edit within `~~~worksplit` wrapper

## Example Test Cases

```rust
#[test]
fn test_parse_edit_single_file() {
    let response = r#"
FILE: src/main.rs
FIND:
fn old() {}
REPLACE:
fn new() {}
END
"#;
    let parsed = parse_edit_instructions(response);
    assert_eq!(parsed.edits.len(), 1);
    assert_eq!(parsed.edits[0].file_path, PathBuf::from("src/main.rs"));
    assert_eq!(parsed.edits[0].find, "fn old() {}");
    assert_eq!(parsed.edits[0].replace, "fn new() {}");
}

#[test]
fn test_parse_edit_multiple_files() {
    let response = r#"
FILE: src/main.rs
FIND:
line1
REPLACE:
line1_new
END

FILE: src/lib.rs
FIND:
line2
REPLACE:
line2_new
END
"#;
    let parsed = parse_edit_instructions(response);
    assert_eq!(parsed.edits.len(), 2);
    assert_eq!(parsed.affected_files.len(), 2);
}

#[test]
fn test_apply_edit_success() {
    let content = "fn old() {}\nfn other() {}";
    let edit = EditInstruction {
        file_path: PathBuf::from("test.rs"),
        find: "fn old() {}".to_string(),
        replace: "fn new() {}".to_string(),
    };
    let result = apply_edit(content, &edit).unwrap();
    assert_eq!(result, "fn new() {}\nfn other() {}");
}

#[test]
fn test_apply_edit_not_found() {
    let content = "fn other() {}";
    let edit = EditInstruction {
        file_path: PathBuf::from("test.rs"),
        find: "fn old() {}".to_string(),
        replace: "fn new() {}".to_string(),
    };
    let result = apply_edit(content, &edit);
    assert!(result.is_err());
}
```

## Additional Function: Edit Prompt Assembly

Also include a function to assemble the prompt for edit mode:

```rust
use std::path::PathBuf;

/// Assemble a creation prompt for edit mode
pub fn assemble_edit_prompt(
    system_prompt: &str,
    target_files: &[(PathBuf, String)],  // Files to be edited with their current content
    context_files: &[(PathBuf, String)], // Additional context
    instructions: &str,
) -> String {
    let mut prompt = String::new();

    // System prompt
    prompt.push_str("[SYSTEM]\n");
    prompt.push_str(system_prompt);
    prompt.push_str("\n\n");
    
    // Edit mode instructions
    prompt.push_str("[EDIT MODE]\n");
    prompt.push_str("You are making surgical edits to existing files. ");
    prompt.push_str("Use the following format for each edit:\n\n");
    prompt.push_str("FILE: path/to/file.rs\n");
    prompt.push_str("FIND:\n<exact text to find>\n");
    prompt.push_str("REPLACE:\n<replacement text>\n");
    prompt.push_str("END\n\n");
    prompt.push_str("Important:\n");
    prompt.push_str("- FIND text must match exactly (including whitespace)\n");
    prompt.push_str("- Include enough context in FIND to be unique\n");
    prompt.push_str("- Multiple edits can be made to the same file\n\n");

    // Target files to be edited
    prompt.push_str("[TARGET FILES]\n");
    prompt.push_str("These are the files you will be editing:\n\n");
    for (path, content) in target_files {
        prompt.push_str(&format!("### File: {}\n", path.display()));
        prompt.push_str("```\n");
        prompt.push_str(content);
        if !content.ends_with('\n') {
            prompt.push('\n');
        }
        prompt.push_str("```\n\n");
    }

    // Additional context files
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

    // Instructions
    prompt.push_str("[INSTRUCTIONS]\n");
    prompt.push_str(instructions);
    prompt.push_str("\n\n");

    prompt
}
```

## Implementation Notes
- FIND text matching should be exact (no fuzzy matching)
- Whitespace at the start/end of FIND/REPLACE blocks should be trimmed line-by-line but internal whitespace preserved
- The first occurrence of FIND is replaced (use `replacen(..., 1)`)
- For multiple replacements of the same text, include multiple FIND/REPLACE blocks
