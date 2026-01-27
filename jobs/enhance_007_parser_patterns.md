---
output_dir: src/core/parser/
output_file: extract.rs
mode: edit
target_files:
  - src/core/parser/extract.rs
verify: false
---

# Add Parser Support for Replace Pattern and Update Fixtures Modes

Add parsing logic for the new replace_pattern and update_fixtures modes.

## Requirements

### 1. Add ReplacePatternInstruction struct

Add after the existing parser structs (around line 30):

```rust
/// Instruction for replace_pattern mode (AFTER/INSERT)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplacePatternInstruction {
    /// Text to find (AFTER pattern)
    pub after_pattern: String,
    /// Text to insert after the pattern
    pub insert_text: String,
    /// Optional scope restriction (e.g., "#[cfg(test)]")
    pub scope: Option<String>,
}

/// Parsed replace pattern instructions
#[derive(Debug, Clone)]
pub struct ParsedReplacePatterns {
    pub instructions: Vec<ReplacePatternInstruction>,
    pub scope: Option<String>,
}
```

### 2. Add parse_replace_pattern_instructions function

Add a new function to parse AFTER/INSERT instructions from LLM response:

```rust
/// Parse replace pattern instructions from LLM response
/// 
/// Format:
/// ```text
/// AFTER:
/// <text to find>
/// INSERT:
/// <text to insert after>
/// 
/// SCOPE: #[cfg(test)]  (optional)
/// ```
pub fn parse_replace_pattern_instructions(response: &str) -> ParsedReplacePatterns {
    let mut instructions = Vec::new();
    let mut current_scope: Option<String> = None;
    let mut after_text = String::new();
    let mut insert_text = String::new();
    let mut in_after_block = false;
    let mut in_insert_block = false;

    for line in response.lines() {
        let trimmed = line.trim();
        
        // Check for SCOPE marker
        if trimmed.to_lowercase().starts_with("scope:") {
            current_scope = Some(trimmed[6..].trim().to_string());
            continue;
        }
        
        // Check for AFTER block start
        if trimmed.to_lowercase() == "after:" {
            // Save previous instruction if exists
            if !after_text.is_empty() && !insert_text.is_empty() {
                instructions.push(ReplacePatternInstruction {
                    after_pattern: after_text.trim().to_string(),
                    insert_text: insert_text.trim().to_string(),
                    scope: current_scope.clone(),
                });
            }
            in_after_block = true;
            in_insert_block = false;
            after_text.clear();
            insert_text.clear();
            continue;
        }
        
        // Check for INSERT block start
        if trimmed.to_lowercase() == "insert:" {
            in_after_block = false;
            in_insert_block = true;
            continue;
        }
        
        // Accumulate text
        if in_after_block {
            if !after_text.is_empty() {
                after_text.push('\n');
            }
            after_text.push_str(line);
        } else if in_insert_block {
            if !insert_text.is_empty() {
                insert_text.push('\n');
            }
            insert_text.push_str(line);
        }
    }
    
    // Don't forget the last instruction
    if !after_text.is_empty() && !insert_text.is_empty() {
        instructions.push(ReplacePatternInstruction {
            after_pattern: after_text.trim().to_string(),
            insert_text: insert_text.trim().to_string(),
            scope: current_scope.clone(),
        });
    }
    
    ParsedReplacePatterns {
        instructions,
        scope: current_scope,
    }
}
```

### 3. Add apply_replace_patterns function

```rust
/// Apply replace pattern instructions to file content
/// Returns the modified content
pub fn apply_replace_patterns(
    content: &str,
    patterns: &ParsedReplacePatterns,
) -> Result<String, String> {
    let mut result = content.to_string();
    
    for instruction in &patterns.instructions {
        // Find all occurrences of the AFTER pattern
        let mut last_pos = 0;
        let mut new_result = String::new();
        let mut found = false;
        
        while let Some(pos) = result[last_pos..].find(&instruction.after_pattern) {
            let absolute_pos = last_pos + pos;
            let end_pos = absolute_pos + instruction.after_pattern.len();
            
            // Check scope if specified
            if let Some(ref scope) = instruction.scope {
                // Simple scope check: see if we're inside the scope block
                let before_text = &result[..absolute_pos];
                if !is_in_scope(before_text, scope) {
                    new_result.push_str(&result[last_pos..end_pos]);
                    last_pos = end_pos;
                    continue;
                }
            }
            
            found = true;
            new_result.push_str(&result[last_pos..end_pos]);
            new_result.push_str(&instruction.insert_text);
            last_pos = end_pos;
        }
        
        new_result.push_str(&result[last_pos..]);
        
        if !found {
            return Err(format!(
                "AFTER pattern not found: {:?}",
                instruction.after_pattern.chars().take(50).collect::<String>()
            ));
        }
        
        result = new_result;
    }
    
    Ok(result)
}

/// Check if a position is within a scope block
fn is_in_scope(before_text: &str, scope: &str) -> bool {
    // Simple heuristic: count opening and closing braces after the scope marker
    if let Some(scope_pos) = before_text.rfind(scope) {
        let after_scope = &before_text[scope_pos..];
        let opens = after_scope.matches('{').count();
        let closes = after_scope.matches('}').count();
        opens > closes
    } else {
        false
    }
}
```

### 4. Add StructLiteralFinder for update_fixtures mode

```rust
/// Find struct literals in source code for update_fixtures mode
#[derive(Debug, Clone)]
pub struct StructLiteralMatch {
    /// Start position of the struct literal
    pub start: usize,
    /// End position (after closing brace)
    pub end: usize,
    /// The last field before the closing brace
    pub last_field_end: usize,
    /// Line number (1-indexed)
    pub line_number: usize,
}

/// Find all struct literals of a given type in source code
pub fn find_struct_literals(content: &str, struct_name: &str) -> Vec<StructLiteralMatch> {
    let mut matches = Vec::new();
    let pattern = format!("{} {{", struct_name);
    let mut search_pos = 0;
    
    while let Some(start) = content[search_pos..].find(&pattern) {
        let absolute_start = search_pos + start;
        
        // Find matching closing brace
        let after_open = absolute_start + pattern.len();
        if let Some((end, last_field_end)) = find_matching_brace(&content[after_open..]) {
            let absolute_end = after_open + end;
            let absolute_last_field = after_open + last_field_end;
            
            // Calculate line number
            let line_number = content[..absolute_start].lines().count() + 1;
            
            matches.push(StructLiteralMatch {
                start: absolute_start,
                end: absolute_end,
                last_field_end: absolute_last_field,
                line_number,
            });
            
            search_pos = absolute_end;
        } else {
            search_pos = after_open;
        }
    }
    
    matches
}

/// Find matching closing brace and position of last field
/// Returns (end_pos, last_field_end_pos) relative to input
fn find_matching_brace(content: &str) -> Option<(usize, usize)> {
    let mut depth = 1;
    let mut last_comma_or_field = 0;
    let mut chars = content.char_indices().peekable();
    
    while let Some((pos, ch)) = chars.next() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some((pos + 1, last_comma_or_field));
                }
            }
            ',' => {
                if depth == 1 {
                    last_comma_or_field = pos + 1;
                }
            }
            ':' => {
                if depth == 1 {
                    // Find end of this field value
                    // This is a simplification - real implementation would need proper parsing
                }
            }
            _ => {}
        }
    }
    
    None
}

/// Insert a new field into struct literals
pub fn insert_field_into_struct_literals(
    content: &str,
    struct_name: &str,
    new_field: &str,
) -> Result<String, String> {
    let matches = find_struct_literals(content, struct_name);
    
    if matches.is_empty() {
        return Err(format!("No {} struct literals found", struct_name));
    }
    
    // Apply insertions from end to start to preserve positions
    let mut result = content.to_string();
    for m in matches.into_iter().rev() {
        // Find the position just before the closing brace
        // Insert the new field there
        let insert_pos = m.end - 1; // Before the }
        
        // Check if we need a comma
        let before_insert = result[..insert_pos].trim_end();
        let needs_comma = !before_insert.ends_with(',') && !before_insert.ends_with('{');
        
        let insertion = if needs_comma {
            format!(",\n            {}", new_field)
        } else {
            format!("\n            {}", new_field)
        };
        
        result.insert_str(insert_pos, &insertion);
    }
    
    Ok(result)
}
```

### 5. Add tests

Add tests at the end of the tests module:

```rust
#[test]
fn test_parse_replace_pattern_simple() {
    let response = r#"
AFTER:
    target_file: None,
INSERT:
    verify: true,
"#;
    let parsed = parse_replace_pattern_instructions(response);
    assert_eq!(parsed.instructions.len(), 1);
    assert_eq!(parsed.instructions[0].after_pattern, "target_file: None,");
    assert_eq!(parsed.instructions[0].insert_text, "verify: true,");
}

#[test]
fn test_parse_replace_pattern_with_scope() {
    let response = r#"
SCOPE: #[cfg(test)]
AFTER:
    field: value,
INSERT:
    new_field: true,
"#;
    let parsed = parse_replace_pattern_instructions(response);
    assert_eq!(parsed.scope, Some("#[cfg(test)]".to_string()));
    assert_eq!(parsed.instructions[0].scope, Some("#[cfg(test)]".to_string()));
}

#[test]
fn test_find_struct_literals() {
    let content = r#"
let a = MyStruct {
    field1: 1,
    field2: 2,
};

let b = MyStruct {
    field1: 3,
};
"#;
    let matches = find_struct_literals(content, "MyStruct");
    assert_eq!(matches.len(), 2);
}

#[test]
fn test_insert_field_into_struct() {
    let content = r#"let a = MyStruct {
    field1: 1,
};"#;
    let result = insert_field_into_struct_literals(content, "MyStruct", "new_field: true").unwrap();
    assert!(result.contains("new_field: true"));
}
```

## Constraints

- Preserve all existing parser functionality
- Handle multi-line AFTER and INSERT patterns
- Scope checking is optional (only applies if SCOPE is specified)
- Struct literal detection should handle nested braces

## Formatting Notes

- Uses 4-space indentation
- Follow existing function naming patterns
- Use tracing::debug! for logging
