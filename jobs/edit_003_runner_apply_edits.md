---
context_files:
  - src/core/runner/mod.rs
output_dir: src/core/
output_file: runner.rs
---

# Add Edit Mode Support to Runner

## Overview
Extend the Runner to handle edit mode jobs, applying surgical edits to existing files rather than replacing them entirely.

## Problem
Currently, the runner only supports full file replacement. When a job uses `mode: edit`, the runner needs to:
1. Read target files
2. Parse edit instructions from LLM response
3. Apply edits to file contents
4. Write updated files
5. Verify the edits

## Requirements

### 1. Import New Parser Functions
Add imports for edit-related functions from the new `parser_edit` module:
```rust
use crate::core::{
    // ... existing imports ...
    parse_edit_instructions, apply_edits, EditInstruction, ParsedEdits,
    assemble_edit_prompt,
};
```

Note: These are exported from `src/core/parser_edit.rs` via `mod.rs`.

### 2. Update run_job Method
In `run_job()`, add a branch for edit mode before the creation phase:

```rust
// === CREATION PHASE ===
if job.metadata.is_edit_mode() {
    // === EDIT MODE ===
    let target_files = job.metadata.get_target_files();
    info!("Edit mode: applying edits to {} file(s)", target_files.len());
    
    // Load target files
    let mut target_file_contents: Vec<(PathBuf, String)> = Vec::new();
    for path in &target_files {
        let full_path = self.project_root.join(path);
        let content = fs::read_to_string(&full_path).map_err(|e| {
            WorkSplitError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Target file not found: {}", full_path.display()),
            ))
        })?;
        target_file_contents.push((path.clone(), content));
    }
    
    // Assemble edit prompt
    let edit_prompt = assemble_edit_prompt(
        create_prompt,
        &target_file_contents,
        &context_files,
        &job.instructions,
    );
    
    info!("Sending to Ollama for edit generation...");
    let response = self
        .ollama
        .generate(&edit_prompt, self.config.behavior.stream_output)
        .await
        .map_err(|e| {
            let _ = self.status_manager.set_failed(job_id, e.to_string());
            WorkSplitError::Ollama(e)
        })?;
    
    // Parse edit instructions
    let parsed_edits = parse_edit_instructions(&response);
    info!("Parsed {} edit(s) for {} file(s)", 
          parsed_edits.edits.len(), 
          parsed_edits.affected_files.len());
    
    // Apply edits to each target file
    for (path, original_content) in &target_file_contents {
        let file_edits: Vec<&EditInstruction> = parsed_edits.edits_for_file(path);
        
        if file_edits.is_empty() {
            info!("No edits for {}", path.display());
            continue;
        }
        
        let edited_content = apply_edits(original_content, &file_edits).map_err(|e| {
            let _ = self.status_manager.set_failed(job_id, e.clone());
            WorkSplitError::EditFailed(e)
        })?;
        
        let line_count = count_lines(&edited_content);
        total_lines += line_count;
        
        // Write the edited file
        let full_path = self.project_root.join(path);
        fs::write(&full_path, &edited_content)?;
        info!("Applied {} edit(s) to: {}", file_edits.len(), full_path.display());
        
        generated_files.push((path.clone(), edited_content));
        self.modified_files.push(full_path.clone());
        full_output_paths.push(full_path);
    }
    
    info!("Edit mode complete: {} file(s) modified, {} edit(s) applied",
          generated_files.len(),
          parsed_edits.edits.len());
          
} else if job.metadata.is_sequential() {
    // ... existing sequential mode code ...
} else {
    // ... existing standard mode code ...
}
```

### 3. Add Error Type
Add to `WorkSplitError` in error.rs:
```rust
#[error("Edit failed: {0}")]
EditFailed(String),
```

### 4. Update Verification for Edit Mode
The existing verification should work, but ensure the generated_files contain the full edited content (not just the edits).

### 5. Handle Retry for Edit Mode
When verification fails in edit mode:
1. Include the original files and the failed edits
2. Ask LLM to generate corrected edits
3. Apply from scratch (reset to original, apply new edits)

### 6. Note on Imports
Ensure the new parser functions are exported:
```rust
pub use parser::{
    // ... existing exports ...
    parse_edit_instructions, apply_edit, apply_edits,
    EditInstruction, ParsedEdits,
};
```

## Expected Behavior

Given a job:
```yaml
---
mode: edit
target_files:
  - src/main.rs
  - src/commands/run.rs
output_dir: src/
output_file: main.rs
---

# Add new CLI flag

Add a --verbose flag to the run command.
```

The LLM generates:
```
FILE: src/main.rs
FIND:
        no_stream: bool,
    },
REPLACE:
        no_stream: bool,
        #[arg(long)]
        verbose: bool,
    },
END

FILE: src/commands/run.rs
FIND:
    pub no_stream: bool,
}
REPLACE:
    pub no_stream: bool,
    pub verbose: bool,
}
END
```

The runner:
1. Reads both target files
2. Parses the edit instructions
3. Applies edits to each file
4. Writes updated files
5. Verifies the changes

## Implementation Notes
- Edit mode is more token-efficient for small changes
- If FIND text is not found, the edit fails (no fuzzy matching)
- Edits are applied in order (first edit, then second, etc.)
- For verification, pass the fully edited file contents, not the edit instructions
- The retry mechanism should regenerate edits, not try to fix failed edits
