---
context_files:
  - jobs/_systemprompt_create.md
output_dir: jobs/
output_file: _systemprompt_create.md
---

# Update System Prompt for Edit Mode

## Overview
Update the creation system prompt to include instructions for edit mode output format.

## Problem
The current system prompt only covers full file replacement. When jobs use edit mode, the LLM needs guidance on the FIND/REPLACE/END format.

## Requirements

### Add Edit Mode Section
Add a new section after the existing output format instructions:

```markdown
## Edit Mode Output

When the job specifies `mode: edit`, generate surgical edits instead of full files.

### Edit Format

```
FILE: path/to/file.rs
FIND:
<exact text to find in the file>
REPLACE:
<text to replace it with>
END
```

### Rules for Edit Mode

1. **FIND must be exact**: The text in FIND must match exactly what's in the target file, including whitespace and indentation

2. **Include enough context**: Make FIND unique - include surrounding lines if needed:
   ```
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

3. **Multiple edits per file**: You can include multiple FIND/REPLACE/END blocks for the same file

4. **Multiple files**: Include a new FILE: line for each different file

5. **Order matters**: Edits are applied in order - if one edit changes text that a later edit needs to find, account for this

6. **Deletions**: To delete code, use empty REPLACE:
   ```
   FIND:
   // old comment
   REPLACE:
   END
   ```

7. **Insertions**: To insert new code, find a unique anchor point and include it in both FIND and REPLACE:
   ```
   FIND:
   fn existing() {}
   REPLACE:
   fn existing() {}
   
   fn new_function() {}
   END
   ```
```

### Update Output Guidelines
In the existing guidelines section, add:
- For edit mode, focus on minimal, surgical changes
- Don't regenerate entire functions if only changing a few lines
- Prefer smaller, focused edits over large replacements

## Full Updated Content

The _systemprompt_create.md should include the edit mode section integrated with existing content. Preserve all existing content and add the edit mode section in the appropriate location (after the standard output format section).
