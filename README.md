# WorkSplit

A Rust CLI tool that orchestrates local Ollama LLM for code generation and verification.

## Overview

WorkSplit processes job files through a local Ollama LLM instance. It handles a two-phase workflow:

1. **Creation**: Generate code (up to 900 LOC) based on instructions and context
2. **Verification**: Validate the generated output against requirements

## Features

- **Job-based workflow**: Define code generation tasks as markdown files
- **Context-aware**: Include up to 2 existing files as context for generation
- **Two-phase validation**: Generate then verify with the same LLM
- **Status tracking**: Track job progress through the pipeline
- **Auto-discovery**: New job files are automatically detected
- **Configurable**: Customize model, timeouts, and behavior via `worksplit.toml`

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

# Create job files in the jobs/ directory
# See examples/sample_project for reference

# Run all pending jobs
worksplit run

# Check status
worksplit status -v

# Validate project structure
worksplit validate
```

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

# Resume stuck jobs
worksplit run --resume

# Reset a job to created status
worksplit run --reset my_job_001

# Override settings
worksplit run --model llama3 --timeout 600
```

### `worksplit status`

Show job status summary.

```bash
worksplit status
worksplit status -v  # Verbose: show each job
```

### `worksplit validate`

Validate the jobs folder structure and job files.

```bash
worksplit validate
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

[behavior]
stream_output = true
create_output_dirs = true
```

CLI flags override config file values.

## Requirements

- **Ollama**: Must be running locally (or remotely with URL configured)
- **Model**: A capable coding model (e.g., qwen3, codellama, deepseek-coder)
- **Rust**: 1.70+ for building from source

## Best Practices

### Job Design

1. **Keep jobs small**: Each job should generate ≤900 lines
2. **Choose context wisely**: Include files that define types/patterns to follow
3. **Be specific**: Clear instructions produce better results
4. **Order matters**: Name jobs alphabetically for dependency order

### Naming Convention

```
feature_order_component.md

Examples:
auth_001_user_model.md
auth_002_session_service.md
api_001_user_endpoints.md
```

### Iterating

```bash
# If a job fails
worksplit status -v              # Check the error
# Edit the job file instructions
worksplit run --reset job_id     # Reset and retry
worksplit run --job job_id       # Run single job
```

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

MIT
