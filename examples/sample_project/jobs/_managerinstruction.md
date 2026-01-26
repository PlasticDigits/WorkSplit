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

## Multi-File Output Modes

### 1. Standard Multi-File (Single LLM Call)

A single job can generate multiple related files using the `~~~worksplit:path/to/file` delimiter syntax. All files are generated in one LLM call.

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

### 2. Sequential Multi-File (One LLM Call Per File)

For bigger changes that exceed token limits, use sequential mode:

```markdown
---
context_files:
  - src/lib.rs
output_dir: src/
output_file: main.rs
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
- The LLM needs to reference previously generated code

### Example Sequential Job

```markdown
---
output_files:
  - src/models/user.rs      # Generated first
  - src/services/user.rs    # Sees user.rs as context
  - src/handlers/user.rs    # Sees both previous files
  - src/routes/mod.rs       # Sees all three previous files
sequential: true
---

# User Module Implementation

## Files to Generate

### src/models/user.rs
- Define User struct with id, name, email fields

### src/services/user.rs  
- UserService with CRUD operations using User model

### src/handlers/user.rs
- HTTP handlers using the service

### src/routes/mod.rs
- Route registration
```

### Example Standard Multi-File Job

```markdown
---
context_files:
  - src/lib.rs
output_dir: src/
output_file: models/mod.rs
---

# Create User and Role Models

Generate the user and role model files with proper module structure.

## Files to Generate
- src/models/user.rs - User struct
- src/models/role.rs - Role enum  
- src/models/mod.rs - Module exports

## Requirements
- User should have id, name, email, and role fields
- Role should be an enum with Admin, User, Guest variants
- mod.rs should re-export all public types
```

## Best Practices

### 1. Size Your Jobs Appropriately

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
- Duplicate information already in instructions
- Exceed 1000 lines each

### 3. Write Clear Instructions

Good instructions include:
- **What** to create (structs, functions, traits)
- **How** it should behave (expected logic, edge cases)
- **Why** (context helps the LLM make good decisions)

Be specific about:
- Required method signatures
- Error handling expectations
- Expected behavior with examples

### 4. Naming Convention

Use descriptive job IDs that indicate feature and order:

```
feature_order_component.md

Examples:
- auth_001_user_model.md
- auth_002_password_hasher.md
- auth_003_session_service.md
- auth_004_auth_middleware.md
```

This ensures jobs run in dependency order (alphabetically).

### 5. Handle Dependencies

If Job B depends on Job A's output:
1. Name Job A alphabetically before Job B
2. Include Job A's output file in Job B's context_files
3. Run `worksplit run` - jobs process in order

## Example Workflow

1. **Analyze the feature**: What files need to be generated?

2. **Map dependencies**: Which files depend on others?

3. **Create jobs in order**:
   ```
   user_001_model.md      -> generates src/models/user.rs
   user_002_repository.md -> generates src/db/user_repo.rs (needs user.rs)
   user_003_service.md    -> generates src/services/user.rs (needs both)
   ```

4. **Run and iterate**:
   ```bash
   worksplit run           # Process all jobs
   worksplit status -v     # Check results
   worksplit run --reset user_002_repository  # Reset if needed
   ```

## Template (Single File)

```markdown
---
context_files:
  - path/to/relevant_file.rs
output_dir: src/path/to/output/
output_file: generated_file.rs
---

# [Clear Title Describing What to Generate]

## Overview
[1-2 sentences explaining the purpose of this code]

## Requirements
- [Requirement 1]
- [Requirement 2]
- [Requirement 3]

## Types/Structs to Define
- `StructName` - [purpose]

## Functions/Methods to Implement
- `function_name(params) -> ReturnType` - [what it does]

## Error Handling
- [How errors should be handled]
- [What error types to use]

## Example Usage (optional)
```rust
// Show how the generated code should be used
```

## Notes (optional)
- [Any additional context or constraints]
```

## Template (Multi-File)

```markdown
---
context_files:
  - path/to/relevant_file.rs
output_dir: src/
output_file: default_output.rs
---

# [Clear Title Describing What to Generate]

## Overview
[1-2 sentences explaining the purpose]

## Files to Generate
- path/to/file1.rs - [purpose]
- path/to/file2.rs - [purpose]
- path/to/mod.rs - [purpose]

## Requirements
- [Requirements that apply to all files]

## File-Specific Details

### file1.rs
- [Specific requirements for file1]

### file2.rs  
- [Specific requirements for file2]

## Notes
- Total output should stay under 900 lines across all files
- Use ~~~worksplit:path/to/file.rs delimiters for each file
```
