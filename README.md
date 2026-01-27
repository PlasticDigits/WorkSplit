# WorkSplit

A Rust CLI tool that delegates code generation to a local Ollama LLM, minimizing the work required from the manager (human or AI) running it.

## The Problem WorkSplit Solves

When using AI assistants for code generation, the **manager** (you, or an AI like Claude/Opus orchestrating the work) pays the cost:

- **Human managers**: Time spent writing prompts, reading output, iterating on failures
- **AI managers**: Input/output tokens consumed reading context and crafting instructions

WorkSplit shifts the expensive work to a **free local LLM** (Ollama), so the manager only needs to:

1. Write a brief job file (once)
2. Run `worksplit run`
3. Check `worksplit status` for pass/fail

The local LLM handles all the verbose code generation and verification internally.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        MANAGER (expensive)                       │
│              Human developer OR AI assistant (Opus)              │
│                                                                  │
│  Work required:                                                  │
│  • Write job files (brief markdown)                              │
│  • Run: worksplit run                                            │
│  • Read: worksplit status (one-line per job)                     │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      WORKSPLIT (orchestrator)                    │
│                                                                  │
│  Handles automatically:                                          │
│  • Loading context files                                         │
│  • Constructing detailed prompts                                 │
│  • Parsing LLM output                                            │
│  • Running verification                                          │
│  • Retry on failure (one automatic retry)                        │
│  • Writing output files                                          │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                       OLLAMA (free/cheap)                        │
│                    Local LLM (qwen3, etc.)                       │
│                                                                  │
│  Does the heavy lifting:                                         │
│  • Reads full context files                                      │
│  • Generates hundreds of lines of code                           │
│  • Verifies against requirements                                 │
│  • Produces verbose output (costs nothing)                       │
└─────────────────────────────────────────────────────────────────┘
```

## Key Principle: Minimize Manager Overhead

Every feature in WorkSplit is designed to reduce what the manager must read, write, or decide:

| Without WorkSplit | With WorkSplit |
|-------------------|----------------|
| Manager writes detailed prompt with full context | Manager writes brief job file, WorkSplit loads context |
| Manager reads verbose LLM output | Manager reads "PASS" or "FAIL: reason" |
| Manager manually verifies code quality | Ollama verifies automatically |
| Manager debugs and re-prompts on failure | WorkSplit retries automatically once |
| Manager tracks which files were generated | WorkSplit tracks status in `_jobstatus.json` |

## Features

- **Minimal job files**: Just specify context files, output path, and requirements
- **Automatic context loading**: WorkSplit reads and formats context files for you
- **Built-in verification**: Ollama validates its own output before marking complete
- **Automatic retry**: One retry attempt on verification failure (no manager intervention)
- **Concise status**: `worksplit status` shows one line per job
- **Summary/status JSON**: `worksplit status --summary` or `--json` for quick checks
- **Dry run**: `worksplit run --dry-run` to preview what would run
- **Reset command**: `worksplit reset <job_id>` for failed/stuck jobs
- **Cancel command**: `worksplit cancel <job_id|all>` to stop running jobs
- **Retry command**: `worksplit retry <job_id>` to retry failed jobs
- **Dependency-aware ordering**: `depends_on` support and `worksplit deps`
- **Build verification**: Optional build/test commands via `worksplit.toml`
- **Batch processing**: Run all jobs with `worksplit run`, check results once at the end
- **Watch mode**: `worksplit status --watch` for real-time progress monitoring

## Installation

```bash
# Clone and build
git clone https://github.com/worksplit/worksplit
cd worksplit
cargo build --release

# Or install directly
cargo install --path .
```

## Quick Start

```bash
# Initialize a new project
worksplit init

# Create a job from template
worksplit new-job my_feature --template replace -o src/services/ -f my_service.rs

# Or create job files manually in the jobs/ directory
# See examples/sample_project for reference

# Run all pending jobs
worksplit run

# Check status
worksplit status -v

# Validate project structure
worksplit validate
```

## Language Support

WorkSplit works with **any programming language**, not just Rust. The core workflow is language-agnostic:

1. Job files describe what code to generate (in any language)
2. Ollama generates the code
3. WorkSplit writes the output files

**However**, you must customize the system prompts for your language:

| File | What to Customize |
|------|-------------------|
| `jobs/_systemprompt_create.md` | Change Rust references to your language, update code style guidelines |
| `jobs/_systemprompt_verify.md` | Update syntax checking guidance, remove Rust-specific checks |
| `jobs/_systemprompt_edit.md` | Adjust FIND/REPLACE examples for your language |
| `worksplit.toml` | Add `build_command` and `test_command` for your build system |

Example for Python:
```toml
# worksplit.toml
[build]
build_command = "python -m py_compile"
test_command = "pytest"
```

Example for TypeScript:
```toml
# worksplit.toml
[build]
build_command = "tsc --noEmit"
test_command = "npm test"
```

## Creating Jobs with Templates

Use `worksplit new-job` to quickly scaffold job files:

```bash
# Basic replace mode (generate new file)
worksplit new-job my_service --template replace -o src/services/ -f service.rs

# Edit mode (modify existing files)
worksplit new-job fix_bug --template edit --targets src/main.rs,src/lib.rs

# Sequential mode (multi-file generation)
worksplit new-job big_feature --template sequential -o src/

# Split mode (break large file into modules)
worksplit new-job refactor --template split --targets src/monolith.rs

# With context files
worksplit new-job impl_api --template replace -c src/types.rs,src/db.rs -o src/ -f api.rs
```

### Available Templates

| Template | Use Case |
|----------|----------|
| `replace` | Generate complete new files (default) |
| `edit` | Make surgical changes to existing files |
| `split` | Break a large file into smaller modules |
| `sequential` | Generate multiple files with accumulated context |
| `tdd` | Test-driven development (tests first, then implementation) |

After creating a job, edit the generated `.md` file to add your specific requirements.

## Project Structure

After running `worksplit init`, your project will have:

```
your_project/
├── worksplit.toml              # Configuration
└── jobs/
    ├── _systemprompt_create.md # Instructions for code generation
    ├── _systemprompt_verify.md # Instructions for verification
    ├── _managerinstruction.md  # How to create job files (for AI assistants)
    ├── _jobstatus.json         # Job status tracking (managed by WorkSplit)
    └── example_001.md          # Example job file
```

## Job File Format

Job files use YAML frontmatter with markdown instructions:

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
- Implement UserService struct with CRUD methods
- Use the User model from context
- Handle database errors gracefully

## Methods to implement
- `new(db: DbConnection) -> Self`
- `create_user(user: NewUser) -> Result<User, ServiceError>`
- `get_user(id: i32) -> Result<Option<User>, ServiceError>`
```

## Multi-File Output

A single job can generate multiple files using the `~~~worksplit:path/to/file.ext` delimiter format. This is useful when related files need to be generated together (e.g., a module and its types, or a struct and its tests).

### Syntax

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

### How It Works

1. **One job, one Ollama call, multiple files**: All files are generated in a single LLM call
2. **Path after colon**: The file path follows the delimiter, e.g., `~~~worksplit:src/main.rs`
3. **Backward compatible**: Using `~~~worksplit` without a path uses the job's `output_file` setting
4. **Unified verification**: All generated files are verified together as a cohesive unit

### Example Job

```markdown
---
context_files:
  - src/lib.rs
output_dir: src/
output_file: models/mod.rs
---

# Create User and Role Models

Create the user and role model files with proper module exports.

Files to generate:
- src/models/user.rs - User struct with fields
- src/models/role.rs - Role enum
- src/models/mod.rs - Module exports
```

The LLM response can include multiple files:

```
~~~worksplit:src/models/user.rs
pub struct User {
    pub id: i32,
    pub name: String,
    pub role: Role,
}
~~~worksplit

~~~worksplit:src/models/role.rs
#[derive(Debug, Clone, Copy)]
pub enum Role {
    Admin,
    User,
    Guest,
}
~~~worksplit

~~~worksplit:src/models/mod.rs
pub mod user;
pub mod role;

pub use user::User;
pub use role::Role;
~~~worksplit
```

### Best Practices

1. **Keep related files together**: Use multi-file output for files that depend on each other
2. **Stay within limits**: Total output across all files should stay under 900 lines
3. **Clear instructions**: List the expected files in your job instructions
4. **Logical grouping**: Group files that would naturally be reviewed together

## Sequential Multi-File Mode

For larger changes that exceed token limits, sequential mode allows generating multiple files with separate LLM calls while maintaining context:

### Syntax

```yaml
---
context_files:
  - src/lib.rs
output_dir: src/
output_file: main.rs  # Default/fallback
output_files:
  - src/main.rs
  - src/commands/run.rs
  - src/core/runner.rs
sequential: true  # One Ollama call per file
---

# Feature Implementation

Generate the complete feature across multiple files.
```

### How It Works

1. **One file per LLM call**: Each file in `output_files` is generated with its own Ollama call
2. **Accumulated context**: Previously generated files in this job automatically become context for subsequent files
3. **Forward planning**: The LLM sees which files remain to be generated, helping it design compatible interfaces
4. **Unified verification**: After all files are generated, verification sees all files together

### Benefits

- **Handle larger changes**: Break token limits by processing files sequentially
- **Maintain consistency**: Previously generated files provide context for later ones
- **Better interfaces**: Knowing what files come next helps design compatible APIs
- **Single verification**: All files are verified together as a cohesive unit

### When to Use Sequential Mode

Use sequential mode when:
- Total output would exceed 900 lines across all files
- Files have dependencies on each other that need to be respected
- You want the LLM to have visibility into previously generated code
- The change is too large for a single LLM call

Use standard multi-file output when:
- All files fit comfortably within a single LLM call
- Files are relatively independent
- Total output is under 900 lines

### Example Workflow

```markdown
---
output_files:
  - src/models/user.rs      # Generated first
  - src/services/user.rs    # Sees user.rs as context
  - src/handlers/user.rs    # Sees both previous files
  - src/routes/user.rs      # Sees all three previous files
sequential: true
---

# User Management Feature

Generate the complete user management feature from model to routes.
```

With this configuration:
1. `src/models/user.rs` is generated first (no accumulated context)
2. `src/services/user.rs` is generated next (sees user model as context)
3. `src/handlers/user.rs` is generated (sees model + service as context)
4. `src/routes/user.rs` is generated last (sees all previous files as context)
5. Verification checks all four files together

## Job Status Flow

```
created → pending_work → pending_verification → pass/fail
```

- **created**: Job file exists, ready to process
- **pending_work**: Sent to Ollama for code generation
- **pending_verification**: Generation complete, awaiting verification
- **pass**: Verification succeeded
- **fail**: Verification failed (see error for details)

## CLI Commands

### `worksplit init`

Initialize a new WorkSplit project in the current directory.

```bash
worksplit init
worksplit init --path /path/to/project
```

### `worksplit run`

Process pending jobs.

```bash
# Run all pending jobs
worksplit run

# Run a specific job
worksplit run --job my_job_001

# Preview without executing
worksplit run --dry-run

# Resume stuck jobs
worksplit run --resume

# Reset a job to created status (legacy)
worksplit run --reset my_job_001

# Override settings
worksplit run --model llama3 --timeout 600
```

### `worksplit status`

Show job status summary.

```bash
worksplit status
worksplit status -v        # Verbose: show each job
worksplit status --summary # Single-line summary
worksplit status --json    # Machine-readable output
```

### `worksplit reset`

Reset a job (or all failed jobs) to created status.

```bash
worksplit reset my_job_001
worksplit reset all
worksplit reset all --status partial
```

### `worksplit deps`

Show dependency ordering for jobs that specify `depends_on`.

```bash
worksplit deps
```

### `worksplit validate`

Validate the jobs folder structure and job files.

```bash
worksplit validate
```

### `worksplit cancel`

Cancel a running job or all running jobs.

```bash
# Cancel a specific job
worksplit cancel my_job_001

# Cancel all running jobs
worksplit cancel all
```

### `worksplit retry`

Retry a failed job from the beginning.

```bash
# Retry a specific job
worksplit retry my_job_001
```

### `worksplit status`

Show job status summary.

```bash
worksplit status
worksplit status -v        # Verbose: show each job
worksplit status --summary # Single-line summary
worksplit status --json    # Machine-readable output
worksplit status --watch   # Watch for changes in real-time
```

### `worksplit run`

Process pending jobs.

```bash
# Run all pending jobs
worksplit run

# Run a specific job
worksplit run --job my_job_001

# Preview without executing
worksplit run --dry-run

# Resume stuck jobs
worksplit run --resume

# Set per-job timeout (seconds)
worksplit run --job-timeout 300

# Override settings
worksplit run --model llama3 --timeout 600
```

## Configuration

Create `worksplit.toml` in your project root:

```toml
[ollama]
url = "http://localhost:11434"
model = "qwen3"
timeout_seconds = 300

[limits]
max_output_lines = 900
max_context_lines = 1000
max_context_files = 2

[build]
# build_command = "cargo check"
# test_command = "cargo test"
# verify_build = true
# verify_tests = false

[behavior]
stream_output = true
create_output_dirs = true
```

CLI flags override config file values.

## Requirements

- **Ollama**: Must be running locally (or remotely with URL configured)
- **Model**: A capable coding model (e.g., qwen3, codellama, deepseek-coder)
- **Rust**: 1.70+ for building from source

## Writing Efficient Jobs (Minimize Your Work)

The goal is to write jobs that **pass on the first try** with **minimal manager effort**. Every retry or edit costs you time/tokens.

### Job File Efficiency

**DO**: Write jobs that are unambiguous and complete
```markdown
---
context_files:
  - src/models/user.rs
output_dir: src/services/
output_file: user_service.rs
---

# Create UserService

Implement CRUD operations for User model.

## Required methods
- `create(user: NewUser) -> Result<User, Error>`
- `get(id: i32) -> Result<Option<User>, Error>`
- `update(id: i32, data: UpdateUser) -> Result<User, Error>`
- `delete(id: i32) -> Result<(), Error>`

## Constraints
- Use the User struct from context_files
- Return ServiceError for all error cases
- No unwrap() calls
```

**DON'T**: Write vague jobs that need iteration
```markdown
# Create a user service
Make it work with the user model.
```

### Reducing Manager Overhead

1. **Explicit over implicit**: List every method signature, every constraint
2. **Include examples**: If output format matters, show an example
3. **Reference context files**: Tell Ollama exactly which types to use
4. **Specify error handling**: "Use Result<T, E>" not "handle errors gracefully"
5. **One concern per job**: Split complex features into multiple jobs
6. **Avoid edit drift**: Prefer replace mode when FIND/REPLACE may change

If a context or target file exceeds 900 LOC, WorkSplit will fail with split-job instructions.

### Naming Convention

```
feature_order_component.md

Examples:
auth_001_user_model.md
auth_002_session_service.md
api_001_user_endpoints.md
```

Jobs run in alphabetical order. Use prefixes to control dependencies.

### When Jobs Fail

If a job fails, you have two options:

**Option 1: Fix and retry** (costs manager effort)
```bash
worksplit status -v              # See the error
# Edit the job file to fix the issue
worksplit reset job_id           # Reset and retry
```

**Option 2: Accept and move on** (for non-critical failures)
- Sometimes verification is too strict
- If the generated code is acceptable, manually mark as complete or delete the job

### Checking Results Efficiently

```bash
# Quick check: just counts
worksplit status
# Output: 5 passed, 1 failed, 2 pending

# Summary or JSON for automation
worksplit status --summary
worksplit status --json

# Only if needed: see which failed
worksplit status -v
# Output: Lists each job with status

# Don't read generated files unless necessary
# Trust the verification unless status shows FAIL
```

## For AI Managers (Claude, Opus, etc.)

If you're an AI assistant using WorkSplit to generate code, here's how to minimize your token usage and maximize success rate.

### Workflow

1. **Read `_managerinstruction.md`** once to understand job format
2. **Plan your batching** - group tasks by file, not by feature
3. **Write job files** using the template (don't craft verbose prompts)
4. **Validate first**: `worksplit validate`
5. **Run all jobs**: `worksplit run`
6. **Check summary only**: `worksplit status --summary`
7. **Only investigate failures**: `worksplit status --json`

### Batching Strategy

**Batch by FILE, not by task.** If multiple tasks modify the same file, create ONE job:

| Bad (7 jobs) | Good (4 jobs) |
|--------------|---------------|
| Task 1 → main.rs | main.rs ← Tasks 1,2,6,7 |
| Task 2 → main.rs | status.rs ← Tasks 1,2,4,7 |
| Task 4 → status.rs | run.rs ← Tasks 1,6 |
| Task 6 → main.rs | validate.rs ← Task 5 |
| Task 7 → main.rs | |

### Mode Selection

```
New file?                    → Replace mode
Small change (<50 lines)?    → Edit mode
Many similar patterns (>10)? → Replace mode (edit will fail)
Changing >50% of file?       → Replace mode
Adding struct field?         → See "Struct Field Pattern" below
Find/replace drift risk?     → Replace mode
```

### Struct Field Addition Pattern

Adding a field to a struct requires updating all struct literals. This is error-prone in edit mode.

**Recommended approach:**
1. Add field with `#[serde(default)]` so YAML parsing still works
2. Use replace mode for test files, or
3. Split into two jobs: one for struct definition, one for tests

### Token-Efficient Commands

```bash
# Quiet mode - no output, just exit code
worksplit run -q && echo "All passed" || echo "Some failed"

# Summary - single line
worksplit status --summary

# JSON - machine readable
worksplit status --json

# Skip verification for trusted jobs
# Add to job frontmatter: verify: false
```

### What NOT to Do

- Don't read generated files (Ollama verified them)
- Don't craft verbose prompts (job files are the prompts)
- Don't watch streaming output (just wait for completion)
- Don't manually retry (WorkSplit retries once automatically)
- Don't use edit mode for changes <10 lines (manual edit is faster)
- Don't use edit mode for multi-file changes (high failure rate)
- Don't retry edit mode more than twice (switch to replace or manual)

### Job File Template

```markdown
---
context_files:
  - path/to/relevant/file.rs
output_dir: path/to/output/
output_file: filename.rs
verify: true  # Set false for low-risk changes
---

# [Brief Title]

## Requirements
- [Requirement 1]
- [Requirement 2]

## Signatures
\`\`\`rust
fn function_name(args) -> ReturnType
\`\`\`

## Constraints
- [Constraint 1]
- [Constraint 2]

## Formatting Notes
- Uses 4-space indentation
- [Other style notes]
```

### Success Rate by Job Type

| Type | Success Rate | Best For | Recommendation |
|------|--------------|----------|----------------|
| Replace (single file) | ~95% | New files, rewrites | **Preferred** |
| Replace (multi-file) | ~90% | Related files, modules | **Preferred** |
| Split | ~90% | Breaking up large files | **Preferred** |
| Sequential | ~85% | Large multi-file features | **Preferred** |
| Edit (single location) | ~70% | One surgical change | Use with caution |
| Edit (2-5 locations) | ~50% | Small fixes | Often fails |
| Edit (5+ locations) | ~20% | **Avoid entirely** | Use replace mode |
| Edit (multiple files) | ~30% | **Avoid entirely** | Separate jobs or manual |

**Edit mode guidance**: For changes under 10 lines, manual editing is faster and more reliable than creating a job file. Reserve edit mode for 20-50 line changes in a single file where replace mode would regenerate too much code.

## Development

```bash
# Run tests
cargo test

# Run with verbose logging
RUST_LOG=debug worksplit run

# Check lints
cargo clippy
```

## License

AGPL-3.0-only
