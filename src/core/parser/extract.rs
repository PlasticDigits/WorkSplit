//! Code extraction and parsing functions for LLM responses.

use regex::Regex;
use std::path::PathBuf;
use tracing::debug;

use super::{ExtractedFile, ParsedReplacePatterns, ReplacePatternInstruction, StructLiteralMatch, VerificationResult};

/// Strip nested fence formats from content
/// 
/// Sometimes LLMs wrap code in both ~~~worksplit AND backticks with a path heading.
/// This function strips:
/// 1. Path-as-heading with backticks: "path.rs\n```lang\ncode\n```"
/// 2. Plain backtick fences: "```lang\ncode\n```"
fn strip_nested_fences(content: &str) -> String {
    let trimmed = content.trim();
    
    // Check for path-as-heading format: "path.ext\n```lang\ncode\n```"
    let path_heading_re = Regex::new(
        r"(?s)^[a-zA-Z0-9_./-]+\.[a-zA-Z]+\s*\n```\w*\n([\s\S]*?)\n?```\s*$"
    ).unwrap();
    
    if let Some(caps) = path_heading_re.captures(trimmed) {
        if let Some(inner) = caps.get(1) {
            debug!("Stripped path-as-heading wrapper from worksplit content");
            return inner.as_str().trim().to_string();
        }
    }
    
    // Check for plain backtick fence: "```lang\ncode\n```"
    let backtick_re = Regex::new(r"(?s)^```\w*\n([\s\S]*?)\n?```\s*$").unwrap();
    
    if let Some(caps) = backtick_re.captures(trimmed) {
        if let Some(inner) = caps.get(1) {
            debug!("Stripped backtick wrapper from worksplit content");
            return inner.as_str().trim().to_string();
        }
    }
    
    // No nested fences, return as-is
    trimmed.to_string()
}

/// Extract multiple files from LLM response
/// 
/// LLMs wrap code in markdown fences. This function:
/// 1. First looks for `~~~worksplit:path/to/file.ext` delimiters (multi-file output)
/// 2. Falls back to `~~~worksplit` without path (single file, uses job's output_path)
/// 3. Falls back to triple backticks for backward compatibility
/// 4. If no fences, uses entire response as single file
/// 
/// Returns a vector of ExtractedFile with optional paths.
/// Files with None path should use the job's default output_path.
pub fn extract_code_files(response: &str) -> Vec<ExtractedFile> {
    // Try worksplit delimiters with optional file path
    let worksplit_re = Regex::new(
        r"(?i)~~~worksplit(?::([^\s\n]+))?(?:\s+\w*)?\n([\s\S]*?)\n~~~worksplit"
    ).unwrap();
    
    let mut files: Vec<ExtractedFile> = Vec::new();
    
    for caps in worksplit_re.captures_iter(response) {
        let path = caps.get(1).map(|m| PathBuf::from(m.as_str().trim()));
        let raw_content = caps.get(2).map(|m| m.as_str().trim().to_string()).unwrap_or_default();
        let content = strip_nested_fences(&raw_content);
        
        if !content.is_empty() {
            if let Some(p) = path {
                debug!("Extracted file with path: {}", p.display());
                files.push(ExtractedFile::with_path(p, content));
            } else {
                debug!("Extracted file using default path");
                files.push(ExtractedFile::default_path(content));
            }
        }
    }
    
    if !files.is_empty() {
        debug!("Extracted {} files using worksplit delimiters", files.len());
        return files;
    }

    // Try path-as-heading format
    let path_heading_re = Regex::new(
        r"(?m)^([a-zA-Z0-9_./-]+\.[a-zA-Z]+)\s*\n```\w*\n([\s\S]*?)\n```"
    ).unwrap();
    
    for caps in path_heading_re.captures_iter(response) {
        let path = caps.get(1).map(|m| PathBuf::from(m.as_str().trim()));
        let content = caps.get(2).map(|m| m.as_str().trim().to_string()).unwrap_or_default();
        
        if !content.is_empty() {
            if let Some(p) = path {
                debug!("Extracted file with path-as-heading: {}", p.display());
                files.push(ExtractedFile::with_path(p, content));
            }
        }
    }
    
    if !files.is_empty() {
        debug!("Extracted {} files using path-as-heading format", files.len());
        return files;
    }

    // Fallback to generic markdown fences
    let re = Regex::new(r"```\w*\n?([\s\S]*?)```").unwrap();
    
    let blocks: Vec<&str> = re
        .captures_iter(response)
        .filter_map(|c| c.get(1).map(|m| m.as_str().trim()))
        .filter(|s| !s.is_empty())
        .collect();

    if blocks.is_empty() {
        debug!("No code fences found, using raw response");
        let cleaned = strip_worksplit_delimiters(response.trim());
        vec![ExtractedFile::default_path(cleaned)]
    } else {
        debug!("Extracted {} code blocks using generic delimiters", blocks.len());
        vec![ExtractedFile::default_path(blocks.join("\n\n"))]
    }
}

/// Strip worksplit delimiter lines from content
fn strip_worksplit_delimiters(content: &str) -> String {
    content
        .lines()
        .filter(|line| {
            let trimmed = line.trim().to_lowercase();
            if trimmed.starts_with("~~~worksplit") {
                return false;
            }
            if trimmed == "~~~" {
                return false;
            }
            true
        })
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

/// Extract code from LLM response (backward compatible single-file version)
pub fn extract_code(response: &str) -> String {
    let files = extract_code_files(response);
    files
        .into_iter()
        .map(|f| f.content)
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// Parse replace pattern instructions from LLM response
pub fn parse_replace_pattern_instructions(response: &str) -> ParsedReplacePatterns {
    let mut instructions = Vec::new();
    let mut current_scope: Option<String> = None;
    let mut after_text = String::new();
    let mut insert_text = String::new();
    let mut in_after_block = false;
    let mut in_insert_block = false;

    for line in response.lines() {
        let trimmed = line.trim();
        
        if trimmed.to_lowercase().starts_with("scope:") {
            current_scope = Some(trimmed[6..].trim().to_string());
            continue;
        }
        
        if trimmed.to_lowercase() == "after:" {
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
        
        if trimmed.to_lowercase() == "insert:" {
            in_after_block = false;
            in_insert_block = true;
            continue;
        }
        
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

/// Parse verification response for PASS/FAIL
pub fn parse_verification(response: &str) -> (VerificationResult, Option<String>) {
    let trimmed = response.trim();
    let lower = trimmed.to_lowercase();
    
    let normalized: String = lower
        .replace('_', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    
    if normalized.starts_with("pass with warnings") || normalized.starts_with("passwithwarnings") {
        debug!("Verification passed with warnings");
        let reason = extract_reason_after_pattern(trimmed, &["pass_with_warnings", "pass with warnings", "passwithwarnings"]);
        return (VerificationResult::PassWithWarnings, reason);
    }
    
    if normalized.starts_with("fail hard") || normalized.starts_with("failhard") {
        debug!("Verification failed hard");
        let reason = extract_reason_after_pattern(trimmed, &["fail_hard", "fail hard", "failhard"]);
        return (VerificationResult::FailHard, reason);
    }
    
    if normalized.starts_with("fail soft") || normalized.starts_with("failsoft") {
        debug!("Verification failed soft");
        let reason = extract_reason_after_pattern(trimmed, &["fail_soft", "fail soft", "failsoft"]);
        return (VerificationResult::FailSoft, reason);
    }
    
    let first_word = trimmed
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_lowercase();
    let first_word_clean: String = first_word.chars().filter(|c| c.is_alphabetic()).collect();
    
    match first_word_clean.as_str() {
        "pass" | "passed" => {
            debug!("Verification passed");
            (VerificationResult::Pass, None)
        }
        "fail" | "failed" => {
            debug!("Verification failed hard (default)");
            let reason = extract_failure_reason(trimmed);
            (VerificationResult::FailHard, reason)
        }
        _ => {
            if lower.contains("pass") && !lower.contains("fail") {
                debug!("Verification passed (found 'pass' in response)");
                (VerificationResult::Pass, None)
            } else if lower.contains("fail") {
                debug!("Verification failed hard (found 'fail' in response)");
                let reason = extract_failure_reason(trimmed);
                (VerificationResult::FailHard, reason)
            } else {
                debug!("Unclear verification response, treating as fail hard");
                (
                    VerificationResult::FailHard,
                    Some("Unclear verification response".to_string())
                )
            }
        }
    }
}

fn extract_reason_after_pattern(response: &str, patterns: &[&str]) -> Option<String> {
    let lower = response.to_lowercase();
    
    for pattern in patterns {
        if let Some(pos) = lower.find(pattern) {
            let after = &response[pos + pattern.len()..];
            let trimmed = after.trim_start_matches(|c: char| c == ':' || c == '-' || c.is_whitespace());
            if !trimmed.is_empty() {
                let first_line = trimmed.lines().next().unwrap_or(trimmed).trim();
                if !first_line.is_empty() {
                    return Some(first_line.to_string());
                }
            }
        }
    }
    None
}

fn extract_failure_reason(response: &str) -> Option<String> {
    let patterns = [
        r"(?i)fail[:\-\s]+(.+)",
        r"(?i)failed[:\-\s]+(.+)",
        r"(?i)reason[:\-\s]+(.+)",
    ];

    for pattern in patterns {
        if let Ok(re) = Regex::new(pattern) {
            if let Some(caps) = re.captures(response) {
                if let Some(m) = caps.get(1) {
                    let reason = m.as_str().trim();
                    if !reason.is_empty() {
                        let first_line = reason.lines().next().unwrap_or(reason);
                        return Some(first_line.to_string());
                    }
                }
            }
        }
    }

    let lines: Vec<&str> = response.lines().collect();
    if lines.len() > 1 {
        return Some(lines[1..].join(" ").trim().to_string());
    }

    None
}

/// Count lines in content
pub fn count_lines(content: &str) -> usize {
    content.lines().count()
}

/// Apply replace pattern instructions to file content
pub fn apply_replace_patterns(
    content: &str,
    patterns: &ParsedReplacePatterns,
) -> Result<String, String> {
    let mut result = content.to_string();
    
    for instruction in &patterns.instructions {
        let mut last_pos = 0;
        let mut new_result = String::new();
        let mut found = false;
        
        while let Some(pos) = result[last_pos..].find(&instruction.after_pattern) {
            let absolute_pos = last_pos + pos;
            let end_pos = absolute_pos + instruction.after_pattern.len();
            
            if let Some(ref scope) = instruction.scope {
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

fn is_in_scope(before_text: &str, scope: &str) -> bool {
    if let Some(scope_pos) = before_text.rfind(scope) {
        let after_scope = &before_text[scope_pos..];
        let opens = after_scope.matches('{').count();
        let closes = after_scope.matches('}').count();
        opens > closes
    } else {
        false
    }
}

/// Find all struct literals of a given type in source code
pub fn find_struct_literals(content: &str, struct_name: &str) -> Vec<StructLiteralMatch> {
    let mut matches = Vec::new();
    let pattern = format!("{} {{", struct_name);
    let mut search_pos = 0;
    
    while let Some(start) = content[search_pos..].find(&pattern) {
        let absolute_start = search_pos + start;
        let after_open = absolute_start + pattern.len();
        
        if let Some((end, last_field_end)) = find_matching_brace(&content[after_open..]) {
            let absolute_end = after_open + end;
            let absolute_last_field = after_open + last_field_end;
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

fn find_matching_brace(content: &str) -> Option<(usize, usize)> {
    let mut depth = 1;
    let mut last_comma_or_field = 0;
    let chars = content.char_indices();
    
    for (pos, ch) in chars {
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
    
    let mut result = content.to_string();
    for m in matches.into_iter().rev() {
        let insert_pos = m.end - 1;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::JobStatus;
    use crate::core::parser::prompts::assemble_sequential_creation_prompt;

    #[test]
    fn test_extract_code_with_worksplit_fences() {
        let response = "Here's the code:\n\n~~~worksplit rust\nfn main() {\n    println!(\"Hello\");\n}\n~~~worksplit\n\nThat's it!";

        let code = extract_code(response);
        assert!(code.contains("fn main()"));
        assert!(code.contains("println!"));
        assert!(!code.contains("Here's the code"));
        assert!(!code.contains("~~~worksplit"));
    }

    #[test]
    fn test_extract_code_with_worksplit_fences_no_language() {
        let response = "Code here:\n\n~~~worksplit\nfn test() {}\n~~~worksplit\n\nDone.";

        let code = extract_code(response);
        assert!(code.contains("fn test()"));
        assert!(!code.contains("Code here"));
    }

    #[test]
    fn test_extract_code_multiple_worksplit_blocks() {
        let response = "First block:\n\n~~~worksplit rust\nfn foo() {}\n~~~worksplit\n\nSecond block:\n\n~~~worksplit\nfn bar() {}\n~~~worksplit";

        let code = extract_code(response);
        assert!(code.contains("fn foo()"));
        assert!(code.contains("fn bar()"));
    }

    #[test]
    fn test_extract_code_fallback_to_generic() {
        let response = "Here's the code:\n\n```rust\nfn main() {\n    println!(\"Hello\");\n}\n```\n\nThat's it!";

        let code = extract_code(response);
        assert!(code.contains("fn main()"));
        assert!(code.contains("println!"));
        assert!(!code.contains("Here's the code"));
    }

    #[test]
    fn test_extract_code_no_fences() {
        let response = "fn main() {\n    println!(\"Hello\");\n}";
        let code = extract_code(response);
        assert_eq!(code, response);
    }

    #[test]
    fn test_parse_verification_pass() {
        let (result, msg) = parse_verification("PASS");
        assert_eq!(result, VerificationResult::Pass);
        assert!(result.is_pass());
        assert!(!result.is_hard_fail());
        assert_eq!(result.to_job_status(), JobStatus::Pass);
        assert_eq!(msg, None);
    }

    #[test]
    fn test_parse_verification_pass_with_warnings() {
        let (result, msg) = parse_verification("PASS_WITH_WARNINGS: Minor style issues");
        assert_eq!(result, VerificationResult::PassWithWarnings);
        assert!(result.is_pass());
        assert!(!result.is_hard_fail());
        assert_eq!(result.to_job_status(), JobStatus::Pass);
        assert_eq!(msg, Some("Minor style issues".to_string()));
    }

    #[test]
    fn test_parse_verification_fail_soft() {
        let (result, msg) = parse_verification("FAIL_SOFT: Potential memory leak");
        assert_eq!(result, VerificationResult::FailSoft);
        assert!(!result.is_pass());
        assert!(!result.is_hard_fail());
        assert_eq!(result.to_job_status(), JobStatus::Fail);
        assert_eq!(msg, Some("Potential memory leak".to_string()));
    }

    #[test]
    fn test_parse_verification_fail_hard() {
        let (result, msg) = parse_verification("FAIL_HARD: Syntax errors on line 42");
        assert_eq!(result, VerificationResult::FailHard);
        assert!(!result.is_pass());
        assert!(result.is_hard_fail());
        assert_eq!(result.to_job_status(), JobStatus::Fail);
        assert_eq!(msg, Some("Syntax errors on line 42".to_string()));
    }

    #[test]
    fn test_parse_verification_case_insensitive() {
        let (result, _) = parse_verification("pass_with_warnings");
        assert_eq!(result, VerificationResult::PassWithWarnings);

        let (result, _) = parse_verification("FAIL_SOFT");
        assert_eq!(result, VerificationResult::FailSoft);

        let (result, _) = parse_verification("fail_hard");
        assert_eq!(result, VerificationResult::FailHard);
    }

    #[test]
    fn test_parse_verification_spaces() {
        let (result, _) = parse_verification("PASS WITH WARNINGS: Minor style issues");
        assert_eq!(result, VerificationResult::PassWithWarnings);

        let (result, _) = parse_verification("FAIL SOFT: Potential memory leak");
        assert_eq!(result, VerificationResult::FailSoft);

        let (result, _) = parse_verification("FAIL HARD: Syntax errors on line 42");
        assert_eq!(result, VerificationResult::FailHard);
    }

    #[test]
    fn test_parse_verification_simple_fail_defaults_to_hard() {
        let (result, msg) = parse_verification("FAIL: Syntax errors");
        assert_eq!(result, VerificationResult::FailHard);
        assert_eq!(msg, Some("Syntax errors".to_string()));
    }

    #[test]
    fn test_parse_verification_unclear_defaults_to_hard_fail() {
        let (result, msg) = parse_verification("This is a confusing response");
        assert_eq!(result, VerificationResult::FailHard);
        assert_eq!(msg, Some("Unclear verification response".to_string()));
    }

    #[test]
    fn test_verification_result_is_pass() {
        assert!(VerificationResult::Pass.is_pass());
        assert!(VerificationResult::PassWithWarnings.is_pass());
        assert!(!VerificationResult::FailSoft.is_pass());
        assert!(!VerificationResult::FailHard.is_pass());
    }

    #[test]
    fn test_verification_result_is_hard_fail() {
        assert!(!VerificationResult::Pass.is_hard_fail());
        assert!(!VerificationResult::PassWithWarnings.is_hard_fail());
        assert!(!VerificationResult::FailSoft.is_hard_fail());
        assert!(VerificationResult::FailHard.is_hard_fail());
    }

    #[test]
    fn test_verification_result_to_job_status() {
        assert_eq!(VerificationResult::Pass.to_job_status(), JobStatus::Pass);
        assert_eq!(VerificationResult::PassWithWarnings.to_job_status(), JobStatus::Pass);
        assert_eq!(VerificationResult::FailSoft.to_job_status(), JobStatus::Fail);
        assert_eq!(VerificationResult::FailHard.to_job_status(), JobStatus::Fail);
    }

    #[test]
    fn test_extract_code_files_with_path() {
        let response = r#"Here's the code:

~~~worksplit:src/main.rs
fn main() {
    println!("Hello");
}
~~~worksplit

That's it!"#;

        let files = extract_code_files(response);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, Some(PathBuf::from("src/main.rs")));
        assert!(files[0].content.contains("fn main()"));
    }

    #[test]
    fn test_extract_code_files_multiple_files() {
        let response = r#"Here are the files:

~~~worksplit:src/lib.rs
pub mod models;
pub mod utils;
~~~worksplit

~~~worksplit:src/models.rs
pub struct User {
    pub name: String,
}
~~~worksplit

~~~worksplit:src/utils.rs
pub fn helper() -> bool {
    true
}
~~~worksplit

Done!"#;

        let files = extract_code_files(response);
        assert_eq!(files.len(), 3);
        
        assert_eq!(files[0].path, Some(PathBuf::from("src/lib.rs")));
        assert!(files[0].content.contains("pub mod models"));
        
        assert_eq!(files[1].path, Some(PathBuf::from("src/models.rs")));
        assert!(files[1].content.contains("pub struct User"));
        
        assert_eq!(files[2].path, Some(PathBuf::from("src/utils.rs")));
        assert!(files[2].content.contains("pub fn helper"));
    }

    #[test]
    fn test_extract_code_files_mixed_paths() {
        let response = r#"
~~~worksplit:src/specific.rs
fn specific() {}
~~~worksplit

~~~worksplit
fn default_file() {}
~~~worksplit
"#;

        let files = extract_code_files(response);
        assert_eq!(files.len(), 2);
        
        assert_eq!(files[0].path, Some(PathBuf::from("src/specific.rs")));
        assert!(files[0].content.contains("fn specific()"));
        
        assert_eq!(files[1].path, None);
        assert!(files[1].content.contains("fn default_file()"));
    }

    #[test]
    fn test_extract_code_files_with_path_and_language() {
        let response = r#"
~~~worksplit:src/main.rs rust
fn main() {
    println!("Hello");
}
~~~worksplit
"#;

        let files = extract_code_files(response);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, Some(PathBuf::from("src/main.rs")));
        assert!(files[0].content.contains("fn main()"));
    }

    #[test]
    fn test_extract_code_files_backward_compat_no_path() {
        let response = r#"
~~~worksplit rust
fn main() {
    println!("Hello");
}
~~~worksplit
"#;

        let files = extract_code_files(response);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, None);
        assert!(files[0].content.contains("fn main()"));
    }

    #[test]
    fn test_extract_code_files_fallback_to_backticks() {
        let response = r#"Here's the code:

```rust
fn main() {
    println!("Hello");
}
```

That's it!"#;

        let files = extract_code_files(response);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, None);
        assert!(files[0].content.contains("fn main()"));
    }

    #[test]
    fn test_extract_code_files_case_insensitive() {
        let response = r#"
~~~WORKSPLIT:src/main.rs
fn main() {}
~~~WORKSPLIT
"#;

        let files = extract_code_files(response);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, Some(PathBuf::from("src/main.rs")));
    }

    #[test]
    fn test_extracted_file_constructors() {
        let with_path = ExtractedFile::with_path(PathBuf::from("test.rs"), "content".to_string());
        assert_eq!(with_path.path, Some(PathBuf::from("test.rs")));
        assert_eq!(with_path.content, "content");

        let default = ExtractedFile::default_path("content".to_string());
        assert_eq!(default.path, None);
        assert_eq!(default.content, "content");
    }

    #[test]
    fn test_assemble_sequential_creation_prompt_basic() {
        let prompt = assemble_sequential_creation_prompt(
            "You are a helpful assistant.",
            &[],
            &[],
            "Create a main function",
            "src/main.rs",
            &[],
        );

        assert!(prompt.contains("[SYSTEM]"));
        assert!(prompt.contains("You are a helpful assistant."));
        assert!(prompt.contains("[INSTRUCTIONS]"));
        assert!(prompt.contains("Create a main function"));
        assert!(prompt.contains("[CURRENT OUTPUT FILE]"));
        assert!(prompt.contains("Generate: src/main.rs"));
        assert!(!prompt.contains("[CONTEXT]"));
        assert!(!prompt.contains("[PREVIOUSLY GENERATED"));
        assert!(!prompt.contains("[REMAINING FILES]"));
    }

    #[test]
    fn test_assemble_sequential_creation_prompt_with_context() {
        let context_files = vec![
            (PathBuf::from("src/lib.rs"), "pub mod models;".to_string()),
        ];

        let prompt = assemble_sequential_creation_prompt(
            "System prompt",
            &context_files,
            &[],
            "Create models",
            "src/models.rs",
            &[],
        );

        assert!(prompt.contains("[CONTEXT]"));
        assert!(prompt.contains("### File: src/lib.rs"));
        assert!(prompt.contains("pub mod models;"));
    }

    #[test]
    fn test_assemble_sequential_creation_prompt_with_previously_generated() {
        let previously_generated = vec![
            (PathBuf::from("src/main.rs"), "fn main() {}".to_string()),
            (PathBuf::from("src/lib.rs"), "pub mod utils;".to_string()),
        ];

        let prompt = assemble_sequential_creation_prompt(
            "System prompt",
            &[],
            &previously_generated,
            "Create utils",
            "src/utils.rs",
            &[],
        );

        assert!(prompt.contains("[PREVIOUSLY GENERATED IN THIS JOB]"));
        assert!(prompt.contains("### File: src/main.rs"));
        assert!(prompt.contains("fn main() {}"));
        assert!(prompt.contains("### File: src/lib.rs"));
        assert!(prompt.contains("pub mod utils;"));
        assert!(prompt.contains("Use them as reference for consistency"));
    }

    #[test]
    fn test_assemble_sequential_creation_prompt_with_remaining_files() {
        let remaining = vec![
            PathBuf::from("src/models.rs"),
            PathBuf::from("src/services.rs"),
        ];

        let prompt = assemble_sequential_creation_prompt(
            "System prompt",
            &[],
            &[],
            "Create main",
            "src/main.rs",
            &remaining,
        );

        assert!(prompt.contains("[REMAINING FILES]"));
        assert!(prompt.contains("These files will be generated after this one:"));
        assert!(prompt.contains("- src/models.rs"));
        assert!(prompt.contains("- src/services.rs"));
        assert!(prompt.contains("Consider their requirements when designing interfaces"));
    }

    #[test]
    fn test_assemble_sequential_creation_prompt_full() {
        let context = vec![
            (PathBuf::from("src/types.rs"), "pub struct Config {}".to_string()),
        ];
        let previously_generated = vec![
            (PathBuf::from("src/main.rs"), "fn main() {}".to_string()),
        ];
        let remaining = vec![
            PathBuf::from("src/utils.rs"),
        ];

        let prompt = assemble_sequential_creation_prompt(
            "System prompt",
            &context,
            &previously_generated,
            "Create the runner module",
            "src/runner.rs",
            &remaining,
        );

        assert!(prompt.contains("[SYSTEM]"));
        assert!(prompt.contains("[CONTEXT]"));
        assert!(prompt.contains("[PREVIOUSLY GENERATED IN THIS JOB]"));
        assert!(prompt.contains("[INSTRUCTIONS]"));
        assert!(prompt.contains("[CURRENT OUTPUT FILE]"));
        assert!(prompt.contains("[REMAINING FILES]"));
        
        let system_pos = prompt.find("[SYSTEM]").unwrap();
        let context_pos = prompt.find("[CONTEXT]").unwrap();
        let prev_pos = prompt.find("[PREVIOUSLY GENERATED").unwrap();
        let instructions_pos = prompt.find("[INSTRUCTIONS]").unwrap();
        let current_pos = prompt.find("[CURRENT OUTPUT FILE]").unwrap();
        let remaining_pos = prompt.find("[REMAINING FILES]").unwrap();

        assert!(system_pos < context_pos);
        assert!(context_pos < prev_pos);
        assert!(prev_pos < instructions_pos);
        assert!(instructions_pos < current_pos);
        assert!(current_pos < remaining_pos);
    }

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

    #[test]
    fn test_worksplit_preferred_over_path_heading() {
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
}
