use std::path::PathBuf;

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

/// Parse edit instructions from LLM response
/// 
/// Format:
/// ```text
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
    let mut edits = Vec::new();
    let mut affected_files = Vec::new();
    let mut current_file_path: Option<PathBuf> = None;
    let mut find_text = String::new();
    let mut replace_text = String::new();
    let mut in_find_block = false;
    let mut in_replace_block = false;

    for line in response.lines() {
        let trimmed_line = line.trim();
        
        // Check for FILE marker
        if trimmed_line.to_lowercase().starts_with("file:") {
            // Start new file context
            let file_path_str = trimmed_line[5..].trim().to_string();
            current_file_path = Some(PathBuf::from(file_path_str));
            continue;
        }

        // Handle FIND block start
        if trimmed_line.to_lowercase() == "find:" {
            in_find_block = true;
            find_text.clear();
            continue;
        }

        // Handle REPLACE block start
        if trimmed_line.to_lowercase() == "replace:" {
            in_find_block = false;
            in_replace_block = true;
            replace_text.clear();
            continue;
        }

        // Handle END marker
        if trimmed_line.to_lowercase() == "end" {
            in_replace_block = false;
            // Save this edit
            if let Some(ref file_path) = current_file_path {
                if !find_text.is_empty() {
                    edits.push(EditInstruction {
                        file_path: file_path.clone(),
                        find: find_text.trim().to_string(),
                        replace: replace_text.trim().to_string(),
                    });
                    if !affected_files.contains(file_path) {
                        affected_files.push(file_path.clone());
                    }
                }
            }
            find_text.clear();
            replace_text.clear();
            continue;
        }

        // Accumulate text in blocks
        if in_find_block {
            if !find_text.is_empty() {
                find_text.push('\n');
            }
            find_text.push_str(line);
        } else if in_replace_block {
            if !replace_text.is_empty() {
                replace_text.push('\n');
            }
            replace_text.push_str(line);
        }
    }

    ParsedEdits {
        edits,
        affected_files,
    }
}

/// Normalize whitespace for fuzzy matching
/// - Trims each line
/// - Collapses multiple spaces/tabs to single space
/// - Preserves newlines
fn normalize_whitespace(text: &str) -> String {
    text.lines()
        .map(|line| {
            line.split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Find fuzzy match location in content
/// Returns (start_idx, end_idx, matched_text) if found
pub fn find_fuzzy_match(content: &str, find_text: &str) -> Option<(usize, usize, String)> {
    let normalized_find = normalize_whitespace(find_text);
    let find_lines: Vec<&str> = normalized_find.lines().collect();
    
    if find_lines.is_empty() {
        return None;
    }
    
    let content_lines: Vec<&str> = content.lines().collect();
    
    // Slide through content looking for a normalized match
    for start_line in 0..content_lines.len() {
        if start_line + find_lines.len() > content_lines.len() {
            break;
        }
        
        let mut matches = true;
        for (i, find_line) in find_lines.iter().enumerate() {
            let content_line_normalized = normalize_whitespace(content_lines[start_line + i]);
            if content_line_normalized != *find_line {
                matches = false;
                break;
            }
        }
        
        if matches {
            // Found a match - calculate byte positions in original content
            let mut start_byte = 0;
            for line in content_lines.iter().take(start_line) {
                start_byte += line.len() + 1; // +1 for newline
            }
            
            let mut end_byte = start_byte;
            for i in 0..find_lines.len() {
                end_byte += content_lines[start_line + i].len();
                if start_line + i + 1 < content_lines.len() {
                    end_byte += 1; // newline
                }
            }
            
            // Extract the actual matched text from original content
            let matched_text = content_lines[start_line..start_line + find_lines.len()]
                .join("\n");
            
            return Some((start_byte, end_byte, matched_text));
        }
    }
    
    None
}

/// Apply a single edit to file content
/// Returns Ok(new_content) if successful, Err(reason) if FIND text not found
/// 
/// Matching strategy:
/// 1. Try exact match first
/// 2. If exact fails, try fuzzy match (normalized whitespace)
/// 3. Fuzzy match auto-applies with the actual matched text
pub fn apply_edit(content: &str, edit: &EditInstruction) -> Result<String, String> {
    // Strategy 1: Exact match
    if content.contains(&edit.find) {
        return Ok(content.replacen(&edit.find, &edit.replace, 1));
    }
    
    // Strategy 2: Fuzzy match with normalized whitespace
    if let Some((_start, _end, matched_text)) = find_fuzzy_match(content, &edit.find) {
        // Apply edit using the actual text found in the file
        let result = content.replacen(&matched_text, &edit.replace, 1);
        tracing::info!(
            "Fuzzy match applied for {} (whitespace normalized)",
            edit.file_path.display()
        );
        return Ok(result);
    }
    
    // No match found - provide detailed feedback
    let find_preview: String = edit.find.chars().take(100).collect();
    let find_first_line = edit.find.lines().next().unwrap_or("").trim();
    
    // Try to find a close match for error message
    let mut suggestion = String::new();
    if !find_first_line.is_empty() && find_first_line.len() > 5 {
        // Look for the first line of FIND in the content
        let normalized_first = normalize_whitespace(find_first_line);
        for (line_num, line) in content.lines().enumerate() {
            let normalized_line = normalize_whitespace(line);
            if normalized_line.contains(&normalized_first) {
                suggestion = format!(
                    "\n\nPossible match at line {}: {:?}",
                    line_num + 1,
                    line.chars().take(80).collect::<String>()
                );
                break;
            }
        }
        
        // If no normalized match, try case-insensitive
        if suggestion.is_empty() {
            let find_lower = normalized_first.to_lowercase();
            for (line_num, line) in content.lines().enumerate() {
                if normalize_whitespace(line).to_lowercase().contains(&find_lower) {
                    suggestion = format!(
                        "\n\nSimilar text at line {} (case mismatch?): {:?}",
                        line_num + 1,
                        line.chars().take(80).collect::<String>()
                    );
                    break;
                }
            }
        }
    }
    
    Err(format!(
        "FIND text not found in {}.\nSearched for: {:?}{}",
        edit.file_path.display(),
        find_preview,
        suggestion
    ))
}

/// Apply multiple edits to file content in order
pub fn apply_edits(content: &str, edits: &[&EditInstruction]) -> Result<String, String> {
    let mut result = content.to_string();
    for edit in edits {
        result = apply_edit(&result, edit)?;
    }
    Ok(result)
}

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
    prompt.push_str("- Multiple edits can be made to the same file\n");
    prompt.push_str("- Use line number hints like 'FIND (near line 50):' to reference locations\n\n");

    // Target files to be edited (with line number hints)
    prompt.push_str("[TARGET FILES]\n");
    prompt.push_str("These are the files you will be editing (line numbers shown every 10 lines):\n\n");
    for (path, content) in target_files {
        prompt.push_str(&format!("### File: {} ({} lines)\n", path.display(), content.lines().count()));
        prompt.push_str("```\n");
        for (idx, line) in content.lines().enumerate() {
            let line_num = idx + 1;
            // Add line number marker every 10 lines or at first line
            if line_num == 1 || line_num % 10 == 0 {
                prompt.push_str(&format!("[Line {:>4}] ", line_num));
            } else {
                prompt.push_str("            "); // Padding to align code
            }
            prompt.push_str(line);
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

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_parse_edit_multiple_edits_same_file() {
        let response = r#"
FILE: src/main.rs
FIND:
fn old1() {}
REPLACE:
fn new1() {}
END

FILE: src/main.rs
FIND:
fn old2() {}
REPLACE:
fn new2() {}
END
"#;
        let parsed = parse_edit_instructions(response);
        assert_eq!(parsed.edits.len(), 2);
        // Note: affected_files will have duplicates removed
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

    #[test]
    fn test_apply_edit_deletion() {
        let content = "fn old() {}\nfn other() {}";
        let edit = EditInstruction {
            file_path: PathBuf::from("test.rs"),
            find: "fn old() {}".to_string(),
            replace: "".to_string(),
        };
        let result = apply_edit(content, &edit).unwrap();
        assert_eq!(result, "\nfn other() {}");
    }

    #[test]
    fn test_apply_edits_multiple() {
        let content = "fn old1() {}\nfn old2() {}\nfn other() {}";
        let edit1 = EditInstruction {
            file_path: PathBuf::from("test.rs"),
            find: "fn old1() {}".to_string(),
            replace: "fn new1() {}".to_string(),
        };
        let edit2 = EditInstruction {
            file_path: PathBuf::from("test.rs"),
            find: "fn old2() {}".to_string(),
            replace: "fn new2() {}".to_string(),
        };
        let result = apply_edits(content, &[&edit1, &edit2]).unwrap();
        assert_eq!(result, "fn new1() {}\nfn new2() {}\nfn other() {}");
    }

    #[test]
    fn test_parse_edit_case_insensitive() {
        let response = r#"
file: src/main.rs
find:
fn old() {}
replace:
fn new() {}
end
"#;
        let parsed = parse_edit_instructions(response);
        assert_eq!(parsed.edits.len(), 1);
        assert_eq!(parsed.edits[0].file_path, PathBuf::from("src/main.rs"));
    }

    #[test]
    fn test_normalize_whitespace() {
        assert_eq!(normalize_whitespace("  hello   world  "), "hello world");
        assert_eq!(normalize_whitespace("fn foo() {}"), "fn foo() {}");
        assert_eq!(
            normalize_whitespace("    pub fn bar(\n        x: i32,\n    )"),
            "pub fn bar(\nx: i32,\n)"
        );
    }

    #[test]
    fn test_fuzzy_match_extra_indentation() {
        // Content has 4-space indent, FIND has 8-space indent
        let content = "fn main() {\n    let x = 1;\n}";
        let edit = EditInstruction {
            file_path: PathBuf::from("test.rs"),
            find: "        let x = 1;".to_string(),  // Wrong indent
            replace: "    let y = 2;".to_string(),
        };
        let result = apply_edit(content, &edit).unwrap();
        assert_eq!(result, "fn main() {\n    let y = 2;\n}");
    }

    #[test]
    fn test_fuzzy_match_tabs_vs_spaces() {
        // Content has spaces, FIND has tabs
        let content = "struct Foo {\n    field: i32,\n}";
        let edit = EditInstruction {
            file_path: PathBuf::from("test.rs"),
            find: "\tfield: i32,".to_string(),  // Tab instead of spaces
            replace: "    new_field: String,".to_string(),
        };
        let result = apply_edit(content, &edit).unwrap();
        assert_eq!(result, "struct Foo {\n    new_field: String,\n}");
    }

    #[test]
    fn test_fuzzy_match_multiline() {
        let content = "impl Foo {\n    fn bar() {\n        println!(\"hello\");\n    }\n}";
        let edit = EditInstruction {
            file_path: PathBuf::from("test.rs"),
            find: "  fn bar() {\n      println!(\"hello\");\n  }".to_string(),  // Different indent
            replace: "    fn baz() {\n        println!(\"world\");\n    }".to_string(),
        };
        let result = apply_edit(content, &edit).unwrap();
        assert!(result.contains("fn baz()"));
        assert!(result.contains("println!(\"world\")"));
    }

    #[test]
    fn test_exact_match_preferred() {
        // Exact match should be used when available
        let content = "fn old() {}";
        let edit = EditInstruction {
            file_path: PathBuf::from("test.rs"),
            find: "fn old() {}".to_string(),
            replace: "fn new() {}".to_string(),
        };
        let result = apply_edit(content, &edit).unwrap();
        assert_eq!(result, "fn new() {}");
    }

    #[test]
    fn test_exact_substring_match() {
        // Exact substring match works even with different leading whitespace
        // The find text "let x = 1;" is found inside "    let x = 1;"
        // Only the matched portion is replaced, preserving leading "    "
        let content = "    let x = 1;";
        let edit = EditInstruction {
            file_path: PathBuf::from("test.rs"),
            find: "let x = 1;".to_string(),  // No indent in find
            replace: "let x = 2;".to_string(),  // No indent in replace
        };
        let result = apply_edit(content, &edit).unwrap();
        assert_eq!(result, "    let x = 2;");  // Original 4-space indent preserved
    }

    #[test]
    fn test_fuzzy_match_when_exact_fails() {
        // Fuzzy match kicks in when exact match fails due to whitespace
        // Here: content has 4-space indent, but FIND has extra trailing spaces that don't exist
        let content = "fn foo() {\n    let x = 1;\n}";
        let edit = EditInstruction {
            file_path: PathBuf::from("test.rs"),
            find: "    let x = 1;   ".to_string(),  // Extra trailing spaces - won't exact match
            replace: "    let x = 2;".to_string(),
        };
        let result = apply_edit(content, &edit).unwrap();
        assert!(result.contains("let x = 2;"));
    }
}
