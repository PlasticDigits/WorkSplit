---
context_files:
  - src/core/parser/edit.rs
output_dir: src/core/parser/
output_file: edit.rs
---

# Add Fuzzy Matching with Similarity Scores

Enhance the edit parser to provide fuzzy matching with detailed similarity information when exact matches fail.

## Requirements

### 1. Add FuzzyMatchResult struct

Create a struct to represent fuzzy match results:

```rust
/// Result of a fuzzy match attempt
#[derive(Debug, Clone)]
pub struct FuzzyMatchResult {
    /// Line number where the closest match was found (1-indexed)
    pub line_number: usize,
    /// Similarity score as percentage (0-100)
    pub similarity: u8,
    /// The actual text found at that location (first 100 chars)
    pub matched_text_preview: String,
    /// Description of the difference
    pub difference_hint: String,
}
```

### 2. Add similarity calculation function

Create `calculate_similarity(text1: &str, text2: &str) -> u8` that:
- Normalizes both strings (trim, collapse whitespace)
- Calculates a simple similarity score based on:
  - Line-by-line comparison
  - Character overlap
- Returns a score from 0-100

### 3. Add find_fuzzy_matches function (plural)

Create `find_fuzzy_matches(content: &str, find_text: &str, max_results: usize) -> Vec<FuzzyMatchResult>` that:
- Scans the content for potential matches
- Returns up to `max_results` matches sorted by similarity (highest first)
- Each result includes line number, similarity score, and preview
- Detects common differences:
  - "differs in whitespace" (spaces vs tabs, extra/missing spaces)
  - "differs in case" (case sensitivity)
  - "similar structure" (same keywords, different values)

### 4. Update apply_edit to return fuzzy match hints on failure

Modify `apply_edit` to return an enhanced error type:

```rust
/// Error when an edit cannot be applied
#[derive(Debug, Clone)]
pub struct EditApplyError {
    pub file_path: PathBuf,
    pub find_preview: String,
    pub reason: String,
    pub fuzzy_matches: Vec<FuzzyMatchResult>,
}

impl std::fmt::Display for EditApplyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FIND text not found in {}", self.file_path.display())?;
        if !self.fuzzy_matches.is_empty() {
            write!(f, "\n  Possible matches:")?;
            for m in &self.fuzzy_matches {
                write!(f, "\n    - Line {} ({}% match): {}",
                    m.line_number, m.similarity, m.difference_hint)?;
            }
        }
        Ok(())
    }
}
```

Change `apply_edit` signature to return `Result<String, EditApplyError>`.

### 5. Update apply_edits to handle new error type

Update `apply_edits` to work with the new `EditApplyError` type.

### 6. Add tests for fuzzy matching

Add tests:
- `test_fuzzy_match_whitespace_difference`
- `test_fuzzy_match_case_difference`
- `test_fuzzy_match_multiple_candidates`
- `test_similarity_calculation`
- `test_edit_apply_error_display`

## Constraints

- Preserve all existing public API (backward compatible)
- The existing `apply_edit` function returns `Result<String, String>` - update call sites
- Keep fuzzy matching efficient (don't scan entire file if not needed)
- Limit preview text to 100 characters

## Formatting Notes

- Uses 4-space indentation
- Use `tracing::debug!` for logging fuzzy match attempts
- Keep existing test structure

## Implementation Notes

The similarity algorithm should be simple but useful:
1. Normalize whitespace on both texts
2. Split into lines
3. For each line in find_text, check if a normalized version exists in content
4. Count matches vs total lines
5. Return percentage
