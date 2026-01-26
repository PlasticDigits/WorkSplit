use std::fs;
use std::path::PathBuf;
use tracing::info;

use crate::error::WorkSplitError;

/// Initialize a new WorkSplit project
pub fn init_project(project_root: &PathBuf) -> Result<(), WorkSplitError> {
    let jobs_dir = project_root.join("jobs");

    // Create jobs directory
    if !jobs_dir.exists() {
        fs::create_dir_all(&jobs_dir)?;
        info!("Created jobs directory: {}", jobs_dir.display());
    } else {
        info!("Jobs directory already exists: {}", jobs_dir.display());
    }

    // Create system prompts
    create_file_if_not_exists(
        &jobs_dir.join("_systemprompt_create.md"),
        DEFAULT_CREATE_PROMPT,
    )?;

    create_file_if_not_exists(
        &jobs_dir.join("_systemprompt_verify.md"),
        DEFAULT_VERIFY_PROMPT,
    )?;

    create_file_if_not_exists(
        &jobs_dir.join("_systemprompt_test.md"),
        DEFAULT_TEST_PROMPT,
    )?;

    create_file_if_not_exists(
        &jobs_dir.join("_managerinstruction.md"),
        DEFAULT_MANAGER_INSTRUCTION,
    )?;

    // Create empty job status file
    create_file_if_not_exists(
        &jobs_dir.join("_jobstatus.json"),
        "[]",
    )?;

    // Create config file
    create_file_if_not_exists(
        &project_root.join("worksplit.toml"),
        DEFAULT_CONFIG,
    )?;

    // Create example job
    create_file_if_not_exists(
        &jobs_dir.join("example_001.md"),
        DEFAULT_EXAMPLE_JOB,
    )?;

    // Create TDD example job
    create_file_if_not_exists(
        &jobs_dir.join("example_002_tdd.md"),
        DEFAULT_TDD_EXAMPLE_JOB,
    )?;

    info!("WorkSplit project initialized successfully!");
    println!("WorkSplit project initialized at {}", project_root.display());
    println!("\nNext steps:");
    println!("1. Edit jobs/_systemprompt_create.md to customize code generation instructions");
    println!("2. Edit jobs/_systemprompt_verify.md to customize verification instructions");
    println!("3. Edit jobs/_systemprompt_test.md to customize test generation (for TDD workflow)");
    println!("4. Create job files in the jobs/ directory");
    println!("5. Run 'worksplit run' to process jobs");
    println!("\nTip: Add 'test_file: <filename>' to job frontmatter to enable TDD workflow");

    Ok(())
}

fn create_file_if_not_exists(path: &PathBuf, content: &str) -> Result<(), WorkSplitError> {
    if !path.exists() {
        fs::write(path, content)?;
        info!("Created file: {}", path.display());
    } else {
        info!("File already exists: {}", path.display());
    }
    Ok(())
}

const DEFAULT_CREATE_PROMPT: &str = r#"# Code Generation System Prompt

You are a code generation assistant. Your task is to generate high-quality code based on the provided context and instructions.

## Guidelines

1. **Output Format**: Output ONLY the code, wrapped in a markdown code fence with the appropriate language tag. Do not include explanations, comments about what you're doing, or any other text outside the code fence.

2. **Line Limit**: Your output must not exceed 900 lines of code. If the task requires more, focus on the most critical functionality.

3. **Code Style**: Follow the code style and patterns from the context files provided. Match the existing conventions for:
   - Naming (variables, functions, types)
   - Formatting and indentation
   - Error handling patterns
   - Documentation style

4. **Imports**: Include all necessary imports at the top of the file.

5. **Documentation**: Add doc comments for all public items (functions, structs, traits, etc.).

6. **Error Handling**: Use proper error handling. Prefer Result types over panics.

7. **Testing**: If the context includes tests, maintain consistency with the testing patterns used.

## Response Format

Your response should be ONLY a code fence containing the complete file content:

"#;

const DEFAULT_VERIFY_PROMPT: &str = r#"# Code Verification System Prompt

You are a code review assistant. Your task is to verify that the generated code meets the requirements specified in the instructions.

## Verification Checklist

1. **Syntax**: Is the code syntactically correct?
2. **Completeness**: Does the code implement all requirements from the instructions?
3. **Correctness**: Does the logic appear correct? Are there any obvious bugs?
4. **Style Consistency**: Does the code match the style of the context files?
5. **Error Handling**: Are errors handled appropriately?
6. **Documentation**: Are public items documented?

## Response Format

Your response MUST start with either:

- `PASS` - if the code meets all requirements
- `FAIL: <reason>` - if the code has issues

Examples:
- `PASS`
- `PASS - Code looks good, all requirements met.`
- `FAIL: Missing error handling for database connection`
- `FAIL: The get_user method is not implemented`

Be strict but fair. Minor style issues should not cause a failure if the code is functionally correct.
"#;

const DEFAULT_TEST_PROMPT: &str = r#"# Test Generation System Prompt

You are a test generation assistant. Your task is to generate comprehensive tests based on the provided requirements BEFORE the implementation exists.

## TDD Approach

You are generating tests first (Test-Driven Development). The implementation does not exist yet. Your tests should:
1. Define the expected behavior based on requirements
2. Cover happy path scenarios
3. Cover edge cases and error conditions
4. Be runnable once the implementation is created

## Guidelines

1. **Output Format**: Output ONLY the test code, wrapped in a markdown code fence with the appropriate language tag.

2. **Test Coverage**: Generate tests for:
   - All functions/methods mentioned in the requirements
   - Input validation and edge cases
   - Error handling scenarios
   - Integration between components (if applicable)

3. **Test Style**: Follow the testing patterns from context files. Common patterns:
   - Rust: `#[cfg(test)]` module with `#[test]` functions
   - TypeScript/JavaScript: describe/it blocks (Jest, Vitest)
   - Python: pytest functions or unittest classes

4. **Assertions**: Use clear, specific assertions that document expected behavior.

5. **Test Names**: Use descriptive test names that explain what is being tested:
   - `test_greet_returns_hello_with_name`
   - `test_greet_handles_empty_string`

## Response Format

Your response should be ONLY a code fence containing the complete test file:

"#;

const DEFAULT_MANAGER_INSTRUCTION: &str = r#"# Manager Instructions for Creating Job Files

This file explains how to create job files for WorkSplit. Use this as a reference when breaking down work into individual jobs.

## Job File Format

Each job file uses YAML frontmatter followed by markdown instructions:

## Frontmatter Fields

- `context_files`: List of files to include as context (max 2, each under 1000 lines)
- `output_dir`: Directory where the output file should be created
- `output_file`: Name of the file to generate
- `test_file`: Name of the test file to generate (for TDD workflow)

## Best Practices

### 1. Keep Jobs Small
Each job should generate at most 900 lines of code. Break larger features into multiple jobs.

### 2. Choose Context Files Wisely
Include files that:
- Define types/structs the generated code will use
- Show patterns the generated code should follow
- Contain interfaces the generated code must implement

### 3. Write Clear Instructions
- Be specific about what to implement
- List required methods/functions explicitly
- Specify error handling expectations
- Mention any constraints or edge cases

### 4. Naming Convention
Use descriptive job IDs that indicate the feature and order:
- `auth_001_user_model.md`
- `auth_002_session_service.md`
- `api_001_user_endpoints.md`

### 5. Dependencies
If one job depends on another's output, ensure they are processed in order (alphabetical by filename).

## TDD Workflow

To enable Test-Driven Development for a job, add the `test_file` field to the frontmatter:

When `test_file` is specified:
1. Tests are generated FIRST based on requirements
2. Implementation is then generated
3. Implementation is verified against requirements
4. (Future) Tests are executed against implementation

This follows TDD principles: write tests first, then implementation.
"#;

const DEFAULT_CONFIG: &str = r#"# WorkSplit Configuration

[ollama]
url = "http://localhost:11434"
model = "qwen-32k:latest"  # Default model (Ollama will auto-start if not running)
timeout_seconds = 300

[limits]
max_output_lines = 900
max_context_lines = 1000
max_context_files = 2

[behavior]
stream_output = true
create_output_dirs = true
"#;

const DEFAULT_EXAMPLE_JOB: &str = r#"---
context_files: []
output_dir: src/
output_file: hello.rs
---

# Create Hello World Module

## Requirements
- Create a simple Rust module with a greeting function
- The function should accept a name parameter
- Return a formatted greeting string

## Functions to Implement

1. `greet(name: &str) -> String` - Returns "Hello, {name}!"
2. `greet_with_time(name: &str, morning: bool) -> String` - Returns appropriate greeting based on time

## Example Usage

"#;

const DEFAULT_TDD_EXAMPLE_JOB: &str = r#"---
context_files: []
output_dir: src/
output_file: calculator.rs
test_file: calculator_test.rs
---

# Create Calculator Module (TDD Example)

This job demonstrates TDD workflow - tests will be generated first!

## Requirements
- Create a calculator module with basic arithmetic operations
- Support add, subtract, multiply, divide functions
- Handle division by zero with Result type

## Functions to Implement

1. `add(a: i32, b: i32) -> i32` - Returns sum
2. `subtract(a: i32, b: i32) -> i32` - Returns difference
3. `multiply(a: i32, b: i32) -> i32` - Returns product
4. `divide(a: i32, b: i32) -> Result<i32, &'static str>` - Returns quotient or error

## Expected Behavior

- `add(2, 3)` returns `5`
- `subtract(5, 3)` returns `2`
- `multiply(4, 5)` returns `20`
- `divide(10, 2)` returns `Ok(5)`
- `divide(10, 0)` returns `Err("division by zero")`
"#;