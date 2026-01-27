# Manager Instructions for Creating Job Files

This document explains how to create job files for WorkSplit when breaking down a feature into implementable chunks.

## CRITICAL: When to Use WorkSplit vs Direct Editing

**WorkSplit has overhead** (job creation, validation, verification, retries). Only use it when the cost savings outweigh this overhead.

### Cost Decision Matrix

| Task Size | Lines Changed | Recommendation | Reason |
|-----------|---------------|----------------|--------|
| Tiny | < 20 lines | **Direct edit** | Job overhead far exceeds savings |
| Small | 20-100 lines | **Direct edit** | Still faster to edit directly |
| Medium | 100-300 lines | **Evaluate** | Break-even zone; use WorkSplit for complex logic |
| Large | 300-500 lines | **WorkSplit** | Clear cost savings from free Ollama tokens |
| Very Large | 500+ lines | **WorkSplit strongly** | Significant savings; split into multiple jobs |

### Quick Decision Guide

```
STOP - Before creating a WorkSplit job, ask:

1. Is this < 100 lines of changes?
   → YES: Edit directly, don't use WorkSplit
   
2. Is this a simple, surgical change?
   → YES: Edit directly, WorkSplit overhead not worth it
   
3. Will this generate 300+ lines of NEW code?
   → YES: Use WorkSplit, clear savings
   
4. Is the logic complex enough to benefit from verification?
   → YES: Use WorkSplit
   → NO: Edit directly
```

---

## Quick Job Creation with Templates

**Preferred method**: Use `worksplit new-job` to scaffold job files quickly:

```bash
# Replace mode - generate a new file
worksplit new-job feature_001 --template replace -o src/services/ -f myService.ts

# Edit mode - modify existing files  
worksplit new-job fix_001 --template edit --targets src/main.ts

# With context files
worksplit new-job impl_001 --template replace -c src/types.ts -o src/ -f api.ts

# Split mode - break large file into modules
worksplit new-job split_001 --template split --targets src/largeFile.ts

# Sequential mode - multi-file with context accumulation
worksplit new-job big_001 --template sequential -o src/
```

After running, edit the generated `jobs/<name>.md` to add specific requirements.

### When to Use Each Template

| Template | Use When | Reliability |
|----------|----------|-------------|
| `replace` | Creating new files or completely rewriting existing ones | High |
| `edit` | Making 2-5 small, isolated changes to existing files | Medium |
| `split` | A file exceeds 900 lines and needs to be modularized | High |
| `sequential` | Generating multiple interdependent files | High |
| `tdd` | You want tests generated before implementation | High |

## Job File Format

Each job file uses YAML frontmatter followed by markdown instructions:

```markdown
---
context_files:
  - src/models/user.ts
  - src/db/connection.ts
output_dir: src/services/
output_file: userService.ts
---

# Create User Service

## Requirements
- Implement UserService class
- Add CRUD methods for User model

## Methods to Implement
- `constructor(db: DbConnection)`
- `createUser(user: NewUser): Promise<User>`
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

Standard mode that generates complete files.

### 2. Edit Mode (Surgical Changes)

For making small, surgical changes to existing files:

```markdown
---
mode: edit
target_files:
  - src/main.ts
output_dir: src/
output_file: main.ts
---

# Add New Config Option

Add the `verbose` option to the config interface.
```

### 3. Split Mode (Breaking Up Large Files)

For splitting a large file into a directory-based module structure:

```markdown
---
mode: split
target_file: src/services/userService.ts
output_dir: src/services/userService/
output_file: index.ts
output_files:
  - src/services/userService/index.ts
  - src/services/userService/create.ts
  - src/services/userService/query.ts
---
```

### 4. Sequential Multi-File

For bigger changes that exceed token limits:

```markdown
---
output_files:
  - src/main.ts
  - src/commands/run.ts
  - src/core/runner.ts
sequential: true
---
```

## Best Practices

### 1. Size Jobs Appropriately

Each job should generate **at most 900 lines of code**. If a feature requires more:
- Split into multiple jobs
- Each job handles one concern (model, service, API, etc.)
- Order jobs by dependency (use alphabetical naming)

### 2. Choose Context Files Wisely

Context files should:
- Define types/interfaces the generated code will use
- Show patterns to follow (error handling, naming conventions)
- Contain interfaces to implement

### 3. Write Clear Instructions

Good instructions include:
- **What** to create (classes, functions, interfaces)
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
