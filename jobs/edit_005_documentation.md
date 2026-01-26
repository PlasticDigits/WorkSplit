---
context_files:
  - jobs/_managerinstruction.md
output_dir: jobs/
output_file: _managerinstruction.md
---

# Update Manager Instructions with Edit Mode

## Overview
Document the edit mode feature in the manager instructions file so AI assistants know how to use it when creating jobs.

## Requirements

### Add New Section: Edit Mode
Add a new section after the "Sequential Multi-File" section:

```markdown
### 3. Edit Mode (Surgical Changes)

For making small, surgical changes to existing files without regenerating them entirely:

```markdown
---
mode: edit  # "edit" for surgical changes, "replace" (default) for full file generation
target_files:
  - src/main.rs
  - src/commands/run.rs
output_dir: src/
output_file: main.rs  # Fallback, not used in edit mode
---

# Add New CLI Flag

Add the `--verbose` flag to the run command.
```

**How It Works:**
1. Target files are read and passed to the LLM
2. LLM generates FIND/REPLACE/END edit instructions
3. Edits are applied surgically to each file
4. Verification checks the modified files

**Edit Instruction Format:**
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
```

**When to Use:**
- Adding a field to a struct (1-5 lines change)
- Modifying a function signature
- Adding an import or export
- Small bug fixes
- Any change where you're modifying <50 lines total

**When NOT to Use:**
- Creating new files (use replace mode)
- Large refactors (use replace or sequential)
- Changes spanning >50% of a file

**Tradeoffs:**

| Approach | Coherence | Token Efficiency | Error Recovery |
|----------|-----------|------------------|----------------|
| Replace (one file) | High | Medium | Good (redo one) |
| Replace (multi-file) | High | Low | Poor (redo all) |
| Sequential | Medium | High | Good (redo one) |
| Edit | Medium | Very High | Good (re-edit) |
```

### Update Guidance Section
Add to the "Context File Selection" or similar guidance section:

```markdown
## Choosing the Right Mode

1. **Replace mode (default)**: Use for new files or when >50% of file changes
2. **Multi-file replace**: Use for tightly coupled files that must be coherent
3. **Sequential mode**: Use for large features exceeding token limits
4. **Edit mode**: Use for surgical changes to existing, working code

Edit mode is particularly useful for:
- Adding CLI flags (changes to main.rs + commands/*.rs)
- Adding struct fields (changes to model + usages)
- Updating function signatures with minimal body changes
```

## Full Updated Content

Update _managerinstruction.md to include the edit mode section. Integrate it naturally with the existing structure while preserving all current content.
