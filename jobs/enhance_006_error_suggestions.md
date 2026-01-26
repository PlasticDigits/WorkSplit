---
context_files:
  - src/error.rs
output_dir: src/
output_file: error.rs
---

# Improve Error Messages with Actionable Suggestions

Enhance error types to include actionable suggestions that help users fix common issues.

## Requirements

### 1. Add EditSuggestion type

```rust
/// Actionable suggestion for fixing an error
#[derive(Debug, Clone)]
pub struct EditSuggestion {
    /// The suggestion text
    pub message: String,
    /// Priority (1 = most likely to help)
    pub priority: u8,
    /// Category of suggestion
    pub category: SuggestionCategory,
}

#[derive(Debug, Clone, Copy)]
pub enum SuggestionCategory {
    Whitespace,
    CaseSensitivity,
    ContextSize,
    ModeChoice,
    LineReference,
}
```

### 2. Add EditFailedWithSuggestions error variant

Update the `WorkSplitError` enum to add a new variant:

```rust
#[error("Edit failed: {message}")]
EditFailedWithSuggestions {
    message: String,
    suggestions: Vec<EditSuggestion>,
    fuzzy_matches: Vec<(String, usize, u8)>, // (file, line, similarity%)
},
```

### 3. Add helper function to generate suggestions

```rust
impl WorkSplitError {
    /// Generate suggestions from edit failure context
    pub fn edit_failed_with_context(
        message: String,
        file_path: &str,
        find_text: &str,
        edit_count: usize,
        has_fuzzy_match: bool,
        fuzzy_matches: Vec<(String, usize, u8)>,
    ) -> Self {
        let mut suggestions = Vec::new();
        
        // Check for whitespace hints
        if message.contains("whitespace") || has_fuzzy_match {
            suggestions.push(EditSuggestion {
                message: format!(
                    "Check whitespace: {} may use different indentation (spaces vs tabs, indent level)",
                    file_path
                ),
                priority: 1,
                category: SuggestionCategory::Whitespace,
            });
        }
        
        // Check for many edits
        if edit_count > 10 {
            suggestions.push(EditSuggestion {
                message: format!(
                    "Consider replace mode: this job has {} edits, replace mode is safer for 10+ edits",
                    edit_count
                ),
                priority: 2,
                category: SuggestionCategory::ModeChoice,
            });
        }
        
        // Check for small FIND context
        let find_lines = find_text.lines().count();
        if find_lines < 3 {
            suggestions.push(EditSuggestion {
                message: "Include more context: FIND text is too short, add surrounding lines for uniqueness".to_string(),
                priority: 1,
                category: SuggestionCategory::ContextSize,
            });
        }
        
        // Add line references from fuzzy matches
        for (file, line, similarity) in &fuzzy_matches {
            if *similarity >= 70 {
                suggestions.push(EditSuggestion {
                    message: format!(
                        "Possible match at {}:{} ({}% similar)",
                        file, line, similarity
                    ),
                    priority: 3,
                    category: SuggestionCategory::LineReference,
                });
            }
        }
        
        // Sort by priority
        suggestions.sort_by_key(|s| s.priority);
        
        WorkSplitError::EditFailedWithSuggestions {
            message,
            suggestions,
            fuzzy_matches,
        }
    }
}
```

### 4. Update Display impl for EditFailedWithSuggestions

The Display impl should show suggestions in a helpful format:

```rust
// In the Display derive or manual impl
// For EditFailedWithSuggestions:
// Error: Edit failed: FIND text not found
//
// Suggestions:
//   1. Check whitespace: file uses 4 spaces, your FIND may use tabs
//   2. Include more context: your FIND appears on lines 45, 78, 120
//   3. Consider replace mode: this job has 15+ edits
//
// See: worksplit help edit-troubleshooting
```

Since we use thiserror, add a method to format with suggestions:

```rust
impl WorkSplitError {
    /// Format error with suggestions for display
    pub fn display_with_suggestions(&self) -> String {
        match self {
            WorkSplitError::EditFailedWithSuggestions { message, suggestions, .. } => {
                let mut output = format!("Error: Edit failed: {}\n", message);
                if !suggestions.is_empty() {
                    output.push_str("\nSuggestions:\n");
                    for (i, s) in suggestions.iter().enumerate() {
                        output.push_str(&format!("  {}. {}\n", i + 1, s.message));
                    }
                }
                output
            }
            other => format!("Error: {}", other),
        }
    }
}
```

### 5. Add tests

Add tests for:
- `test_edit_suggestion_priority`
- `test_edit_failed_with_context_whitespace`
- `test_edit_failed_with_context_many_edits`
- `test_display_with_suggestions`

## Constraints

- Preserve all existing error variants
- EditFailedWithSuggestions should derive Debug and thiserror::Error
- Suggestions should be sorted by priority (1 = most important)
- Keep suggestion messages concise but actionable

## Formatting Notes

- Uses 4-space indentation
- Follow existing error.rs patterns
- Use thiserror derive macros

## Example Output

When an edit fails, the error should display like:

```
Error: Edit failed: FIND text not found in src/main.rs

Suggestions:
  1. Check whitespace: src/main.rs may use different indentation (spaces vs tabs, indent level)
  2. Include more context: FIND text is too short, add surrounding lines for uniqueness
  3. Possible match at src/main.rs:45 (85% similar)

See: worksplit help edit-troubleshooting
```
