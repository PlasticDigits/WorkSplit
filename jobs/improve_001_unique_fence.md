---
context_files:
  - src/core/parser/extract.rs
output_dir: src/core/parser/
output_file: extract.rs
---

# Unique Fence Delimiter for Code Extraction

## Overview
Change the code extraction to use a unique fence delimiter `~~~worksplit` instead of the generic triple backtick (```) pattern. This avoids conflicts when the LLM generates code that itself contains markdown code fences.

## Problem
Currently, `extract_code()` uses regex to find ` ```language ... ``` ` blocks. When the LLM generates code that contains markdown fences (common when generating documentation, README files, or this very codebase), the extraction can fail or extract incorrect content.

## Solution
Change the system prompt instructions to ask the LLM to wrap its output in `~~~worksplit\n...\n~~~worksplit` delimiters, and update the parser to look for this unique delimiter first, falling back to the generic ``` pattern for backward compatibility.

## Requirements

### 1. Update `extract_code()` function
- First, look for `~~~worksplit` delimiters (with or without language tag after worksplit)
- Pattern: `~~~worksplit\n(content)\n~~~worksplit` or `~~~worksplit rust\n(content)\n~~~worksplit`
- If found, extract the content between the delimiters
- If NOT found, fall back to the existing ``` pattern for backward compatibility
- Log which extraction method was used (debug level)

### 2. Regex patterns
- Primary: `~~~worksplit\s*\w*\n([\s\S]*?)\n~~~worksplit`
- Fallback: existing pattern ```` ```\w*\n?([\s\S]*?)``` ````

### 3. Add tests
- Test extraction with `~~~worksplit` delimiter (no language)
- Test extraction with `~~~worksplit rust` delimiter (with language)
- Test fallback to ``` when `~~~worksplit` not found
- Test that nested ``` fences inside `~~~worksplit` are preserved
- Test multiple `~~~worksplit` blocks

## Implementation Notes
- Keep all existing tests passing
- The `~~~worksplit` pattern should be case-insensitive for the word "worksplit"
- Preserve the trimming behavior of the existing implementation
- All other functions in parser.rs should remain unchanged

## Expected Behavior
```rust
// Using ~~~worksplit delimiter
let response = r#"Here's the code:

~~~worksplit rust
fn main() {
    println!("Hello");
}
~~~worksplit

Done!"#;
let code = extract_code(response);
assert!(code.contains("fn main()"));
assert!(!code.contains("~~~worksplit"));
```
