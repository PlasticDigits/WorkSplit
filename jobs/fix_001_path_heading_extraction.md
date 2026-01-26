---
mode: edit
target_files:
  - src/core/parser.rs
output_dir: src/core/
output_file: parser.rs
---

# Fix Code Extraction for Path-as-Heading Format

## Overview

LLMs often output multi-file code in a "path as heading" format instead of the preferred `~~~worksplit:path` format:

```
src/user_service/delete.rs
```rust
use super::...
code here
```
```

The current `extract_code_files()` function doesn't handle this format - it falls back to simple triple backticks and loses the path information.

## Solution

Add a new extraction pattern between the `~~~worksplit` check and the simple backtick fallback.

## Formatting Notes

- Uses 4-space indentation
- File uses `Regex::new()` from the `regex` crate
- ExtractedFile has `with_path(PathBuf, String)` and `default_path(String)` constructors
- The function returns `Vec<ExtractedFile>`

## Exact Edit Location

The FIND text is exactly these lines (from line 98-104):

```
    if !files.is_empty() {
        debug!("Extracted {} files using worksplit delimiters", files.len());
        return files;
    }

    // Fallback to generic markdown fences (triple backticks)
```

## Required REPLACE

Replace with code that:
1. Keeps the original `if !files.is_empty()` block  
2. Adds a NEW block after it for "path-as-heading" format
3. Uses regex: `r"(?m)^([a-zA-Z0-9_./-]+\.[a-zA-Z]+)\s*\n```\w*\n([\s\S]*?)\n```"`
4. For each match, push `ExtractedFile::with_path(PathBuf::from(path), content)`
5. If any found, return them
6. Then the `// Fallback` comment

## Exact FIND/REPLACE

```
FILE: src/core/parser.rs
FIND:
    if !files.is_empty() {
        debug!("Extracted {} files using worksplit delimiters", files.len());
        return files;
    }

    // Fallback to generic markdown fences (triple backticks)
REPLACE:
    if !files.is_empty() {
        debug!("Extracted {} files using worksplit delimiters", files.len());
        return files;
    }

    // Try path-as-heading format: "path/to/file.ext\n```lang\ncode\n```"
    // Many LLMs use this format instead of ~~~worksplit
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

    // Fallback to generic markdown fences (triple backticks)
END
```
