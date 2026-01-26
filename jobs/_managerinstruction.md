# Manager Instructions for Creating Job Files

This document explains how to create job files for WorkSplit when breaking down a feature into implementable chunks.

## Job File Format

Each job file uses YAML frontmatter followed by markdown instructions:

```markdown
---
context_files:
  - src/models/user.rs
  - src/db/connection.rs
output_dir: src/services/
output_file: user_service.rs
---

# Create User Service

## Requirements
- Implement UserService struct
- Add CRUD methods for User model

## Methods to Implement
- `new(db: DbConnection) -> Self`
- `create_user(user: NewUser) -> Result<User, ServiceError>`
```

## Frontmatter Fields

| Field | Required | Description |
|-------|----------|-------------|
| `context_files` | No | List of files to include as context (max 2, each under 1000 lines) |
| `output_dir` | Yes | Directory where the output file will be created |
| `output_file` | Yes | Name of the generated file (default if multi-file output is used) |
| `output_files` | No | List of files to generate in sequential mode |
| `sequential` | No | Enable sequential mode (one LLM call per file) |
| `mode` | No | Output mode: "replace" (default) or "edit" for surgical changes |
| `target_files` | No | Files to edit when using edit mode |

## Output Modes

### 1. Replace Mode (Default)

Standard mode that generates complete files. Use `~~~worksplit:path/to/file` delimiter for multi-file output.

### 2. Edit Mode (Surgical Changes)

For making small, surgical changes to existing files without regenerating them entirely:

```markdown
---
mode: edit
target_files:
  - src/main.rs
  - src/commands/run.rs
output_dir: src/
output_file: main.rs
---

# Add New CLI Flag

Add the `--verbose` flag to the run command.
```

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

**CRITICAL: Edit Mode Best Practices**

Edit mode is sensitive to exact text matching. To ensure successful edits:

1. **Always include target files as context_files** - The LLM needs to see the exact formatting:
   ```yaml
   mode: edit
   context_files:
     - src/main.rs  # Same as target!
   target_files:
     - src/main.rs
   ```

2. **Describe the exact formatting in instructions** - Be explicit:
   ```markdown
   ## Formatting Notes
   - Uses 4-space indentation (not tabs)
   - Struct fields have no trailing commas
   - Uses `#[arg(...)]` attribute style, not `#[clap(...)]`
   ```

3. **Quote the exact surrounding context** - In instructions, show what's nearby:
   ```markdown
   ## Edit Location
   After the existing imports:
   ```rust
   use clap::{Parser, Subcommand};
   use std::path::PathBuf;
   ```
   Add the new import here.
   ```

4. **Specify line numbers when helpful** - Reference locations:
   ```markdown
   Around line 45-50, in the `Commands` enum, after the `Validate` variant...
   ```

5. **Keep FIND blocks minimal but unique** - Include just enough context to be unambiguous.

**Common Edit Mode Failures:**
- Wrong indentation (tabs vs spaces, 2 vs 4 spaces)
- Extra/missing newlines
- Trailing whitespace differences
- Case sensitivity mismatches

**When to Use Edit Mode:**
- Adding a field to a struct (1-5 lines change)
- Modifying a function signature
- Adding an import or export
- Small bug fixes
- Any change where you're modifying <50 lines total

**When NOT to Use Edit Mode:**
- Creating new files (use replace mode)
- Large refactors (use replace or sequential)
- Changes spanning >50% of a file

### 3. Multi-File Replace (Single LLM Call)

A single job can generate multiple related files using the `~~~worksplit:path/to/file` delimiter syntax.

```
~~~worksplit:src/models/user.rs
pub struct User {
    pub id: i32,
    pub name: String,
}
~~~worksplit

~~~worksplit:src/models/mod.rs
pub mod user;
pub use user::User;
~~~worksplit
```

**When to Use:**
- Creating a module with its mod.rs exports
- Generating a struct and its associated tests
- Files that must be consistent with each other
- Related changes that fit within the 900-line total limit

### 4. Split Mode (Breaking Up Large Files)

For splitting a large file into a directory-based module structure:

```markdown
---
mode: split
target_file: src/services/user_service.rs
output_dir: src/services/user_service/
output_file: mod.rs
output_files:
  - src/services/user_service/mod.rs
  - src/services/user_service/create.rs
  - src/services/user_service/query.rs
---

# Split user_service.rs into Directory Module

Split into standalone helper functions (not impl blocks in submodules).

## File Structure

- `mod.rs`: UserService struct, public API, calls helper functions
- `create.rs`: User creation logic
- `query.rs`: User query/search logic

## Function Signatures (REQUIRED)

Provide exact signatures for each submodule function:

### create.rs
```rust
pub(crate) async fn create_user(
    db: &DbConnection,
    user_data: &CreateUserRequest,
) -> Result<User, ServiceError>

pub(crate) async fn create_users_batch(
    db: &DbConnection,
    users: &[CreateUserRequest],
) -> Result<Vec<User>, ServiceError>
```

### query.rs
```rust
pub(crate) async fn find_user_by_id(
    db: &DbConnection,
    id: i64,
) -> Result<Option<User>, ServiceError>

pub(crate) async fn search_users(
    db: &DbConnection,
    query: &UserSearchQuery,
) -> Result<Vec<User>, ServiceError>
```
```

**Split Mode Frontmatter:**

| Field | Required | Description |
|-------|----------|-------------|
| `mode` | Yes | Must be `split` |
| `target_file` | Yes | The large file to split |
| `output_dir` | Yes | Directory for output files |
| `output_files` | Yes | List of files to generate (one LLM call each) |

**Job Instructions Must Include:**
- **Function signatures**: Exact signatures for each submodule function
  - Include `async` keyword if function uses `.await`
  - List all parameters with types
  - Specify return type
- **File structure**: What goes in each file
- **Extraction plan**: Which functions/logic move where

**When to Use:**
- File exceeds 500+ lines
- Clear logical separation exists (CRUD operations, modes, phases)
- Directory module structure is appropriate for the codebase

### 5. Sequential Multi-File (One LLM Call Per File)

For bigger changes that exceed token limits, use sequential mode:

```markdown
---
output_files:
  - src/main.rs
  - src/commands/run.rs
  - src/core/runner.rs
sequential: true
---

# Large Feature Implementation
```

**How Sequential Mode Works:**
1. Each file in `output_files` gets its own Ollama call
2. Previously modified files in this job become automatic context
3. Final verification sees all files together

**When to Use:**
- Total output would exceed 900 lines
- Files have dependencies on each other
- Context window limits would be exceeded

## Mode Comparison

| Approach | Coherence | Token Efficiency | Error Recovery |
|----------|-----------|------------------|----------------|
| Replace (one file) | High | Medium | Good (redo one) |
| Replace (multi-file) | High | Low | Poor (redo all) |
| Sequential | Medium | High | Good (redo one) |
| Edit | Medium | Very High | Good (re-edit) |

## Best Practices

### 1. Size Jobs Appropriately

Each job should generate **at most 900 lines of code**. If a feature requires more:
- Split into multiple jobs
- Each job handles one concern (model, service, API, etc.)
- Order jobs by dependency (use alphabetical naming)

### 2. Choose Context Files Wisely

Context files should:
- Define types the generated code will use
- Show patterns to follow (error handling, naming conventions)
- Contain interfaces to implement

Context files should NOT:
- Be unrelated to the task
- Exceed 1000 lines each

### 3. Write Clear Instructions

Good instructions include:
- **What** to create (structs, functions, traits)
- **How** it should behave (expected logic, edge cases)
- **Why** (context helps the LLM make good decisions)

### 4. Naming Convention

```
feature_order_component.md

Examples:
- auth_001_user_model.md
- auth_002_password_hasher.md
- auth_003_session_service.md
```

This ensures jobs run in dependency order (alphabetically).

### 5. Handle Dependencies

If Job B depends on Job A's output:
1. Name Job A alphabetically before Job B
2. Include Job A's output file in Job B's context_files
3. Run `worksplit run` - jobs process in order
