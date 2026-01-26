---
context_files:
  - src/error.rs
output_dir: src/
output_file: error.rs
---

# Add Edit Mode Error Type

## Overview
Add a new error variant to handle failures when applying edit instructions.

## Problem
When edit mode fails (e.g., FIND text not found in target file), we need a specific error type to communicate this clearly.

## Requirements

### Add Error Variant
Add to `WorkSplitError` enum:

```rust
#[error("Edit failed: {0}")]
EditFailed(String),
```

This error is used when:
- FIND text is not found in target file
- Target file cannot be read
- Edit instructions are malformed

## Implementation
Add the variant after the existing variants in the `WorkSplitError` enum. Keep the alphabetical-ish ordering if there is one, or add near related errors like `OutputTooLarge`.

## Full File
Regenerate the complete error.rs file with the new variant added.
