//! Rust-specific templates for WorkSplit

use super::Templates;

/// Get Rust-specific templates
pub fn templates() -> Templates {
    Templates {
        create_prompt: CREATE_PROMPT,
        verify_prompt: VERIFY_PROMPT,
        test_prompt: TEST_PROMPT,
        manager_instruction: MANAGER_INSTRUCTION,
        config: CONFIG,
        example_job: EXAMPLE_JOB,
        tdd_example_job: TDD_EXAMPLE_JOB,
    }
}

pub const CREATE_PROMPT: &str = r#"# Rust Code Generation System Prompt

You are a code generation assistant specializing in Rust. Your task is to generate high-quality, idiomatic Rust code based on the provided context and instructions.

## Guidelines

1. **Output Format**: Output ONLY the code, wrapped in a markdown code fence with the `rust` language tag. Do not include explanations, comments about what you're doing, or any other text outside the code fence.

2. **Line Limit**: Your output must not exceed 900 lines of code. If the task requires more, focus on the most critical functionality.

3. **Code Style**: Follow Rust idioms and patterns from the context files provided. Match the existing conventions for:
   - Naming (snake_case for functions/variables, CamelCase for types)
   - Formatting and indentation (use rustfmt conventions)
   - Error handling patterns (Result, Option, ? operator)
   - Documentation style (/// for doc comments)

4. **Imports**: Include all necessary `use` statements at the top of the file.

5. **Documentation**: Add doc comments (`///`) for all public items (functions, structs, traits, enums).

6. **Error Handling**: Use proper error handling with `Result` and `Option`. Prefer `?` operator. Avoid `unwrap()` in production code.

7. **Ownership**: Respect Rust's ownership rules. Prefer borrowing over cloning when possible.

8. **Testing**: If the context includes tests, maintain consistency with the testing patterns used (`#[cfg(test)]` module).

## Response Format

Your response should be ONLY a code fence containing the complete file content:

"#;

pub const VERIFY_PROMPT: &str = r#"# Rust Code Verification System Prompt

You are a Rust code review assistant. Your task is to verify that the generated code meets the requirements and follows Rust best practices.

## Verification Checklist

1. **Syntax**: Is the code syntactically correct Rust?
2. **Completeness**: Does the code implement all requirements from the instructions?
3. **Correctness**: Does the logic appear correct? Are there any obvious bugs?
4. **Ownership**: Are ownership, borrowing, and lifetimes handled correctly?
5. **Error Handling**: Are `Result` and `Option` used appropriately? No unnecessary `unwrap()`?
6. **Style Consistency**: Does the code follow Rust idioms and match context file patterns?
7. **Documentation**: Are public items documented with `///` comments?
8. **Safety**: Is there any unsafe code? If so, is it justified and documented?

## Response Format

Your response MUST start with either:

- `PASS` - if the code meets all requirements
- `FAIL: <reason>` - if the code has issues

Examples:
- `PASS`
- `PASS - Code looks good, all requirements met.`
- `FAIL: Missing error handling for database connection`
- `FAIL: The get_user method is not implemented`
- `FAIL: Using unwrap() without proper error handling`

Be strict but fair. Minor style issues should not cause a failure if the code is functionally correct.
"#;

pub const TEST_PROMPT: &str = r#"# Rust Test Generation System Prompt

You are a test generation assistant specializing in Rust. Your task is to generate comprehensive tests based on the provided requirements BEFORE the implementation exists.

## TDD Approach

You are generating tests first (Test-Driven Development). The implementation does not exist yet. Your tests should:
1. Define the expected behavior based on requirements
2. Cover happy path scenarios
3. Cover edge cases and error conditions
4. Be runnable once the implementation is created

## Guidelines

1. **Output Format**: Output ONLY the test code, wrapped in a markdown code fence with the `rust` language tag.

2. **Test Structure**: Use the standard Rust test structure:
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;
       
       #[test]
       fn test_function_name() {
           // Test body
       }
   }
   ```

3. **Test Coverage**: Generate tests for:
   - All functions/methods mentioned in the requirements
   - Input validation and edge cases
   - Error handling scenarios (`Result::Err` paths)
   - Boundary conditions

4. **Assertions**: Use clear assertions:
   - `assert_eq!()` for equality
   - `assert_ne!()` for inequality
   - `assert!()` for boolean conditions
   - `assert!(result.is_ok())` / `assert!(result.is_err())` for Results

5. **Test Names**: Use descriptive snake_case names:
   - `test_greet_returns_hello_with_name`
   - `test_greet_handles_empty_string`
   - `test_divide_by_zero_returns_error`

## Response Format

Your response should be ONLY a code fence containing the complete test file:

"#;

pub const MANAGER_INSTRUCTION: &str = r#"# Manager Instructions for Creating Rust Job Files

This file explains how to create job files for a Rust WorkSplit project.

## Job File Format

Each job file uses YAML frontmatter followed by markdown instructions:

```yaml
---
context_files:
  - src/models/user.rs
output_dir: src/services/
output_file: user_service.rs
---
```

## Frontmatter Fields

- `context_files`: List of Rust files to include as context (max 2, each under 1000 lines)
- `output_dir`: Directory where the output `.rs` file should be created
- `output_file`: Name of the Rust file to generate
- `test_file`: Name of the test file to generate (for TDD workflow)

## Rust-Specific Best Practices

### 1. Keep Jobs Small
Each job should generate at most 900 lines of Rust code. Break larger features into multiple jobs.

### 2. Choose Context Files Wisely
Include files that:
- Define structs/enums the generated code will use
- Show trait implementations to follow
- Contain error types to use
- Show module patterns to match

### 3. Write Clear Instructions
- Be specific about what to implement
- List required methods/functions explicitly  
- Specify error handling expectations (which error types)
- Mention any lifetimes or generics needed
- Reference traits to implement

### 4. Naming Convention
Use descriptive job IDs with category prefix:
- `auth_001_user_model.md`
- `auth_002_session_service.md`
- `db_001_connection_pool.md`

### 5. Module Structure
Consider module organization:
- `mod.rs` files for re-exports
- Feature modules grouped logically
- Public API surface minimization

## TDD Workflow

To enable Test-Driven Development, add the `test_file` field:

```yaml
---
context_files: []
output_dir: src/
output_file: calculator.rs
test_file: calculator_test.rs
---
```

When `test_file` is specified:
1. Tests are generated FIRST based on requirements
2. Implementation is then generated to pass tests
3. Implementation is verified against requirements
"#;

pub const CONFIG: &str = r#"# WorkSplit Configuration

[project]
language = "rust"

[ollama]
url = "http://localhost:11434"
model = "qwen-32k:latest"
timeout_seconds = 300

[limits]
max_output_lines = 900
max_context_lines = 1000
max_context_files = 2

[behavior]
stream_output = true
create_output_dirs = true

[build]
build_command = "cargo check"
test_command = "cargo test"
verify_build = false
verify_tests = false
"#;

pub const EXAMPLE_JOB: &str = r#"---
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

```rust
let greeting = greet("World");
assert_eq!(greeting, "Hello, World!");

let morning_greeting = greet_with_time("Alice", true);
assert_eq!(morning_greeting, "Good morning, Alice!");
```
"#;

pub const TDD_EXAMPLE_JOB: &str = r#"---
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
