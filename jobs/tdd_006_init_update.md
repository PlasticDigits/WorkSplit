---
context_files:
  - src/commands/init.rs
output_dir: src/commands/
output_file: init.rs
---

# Update Init Command for TDD Support

## Overview
Update the `init` command to create the `_systemprompt_test.md` file for TDD workflow support.

## Requirements

### New System Prompt File

Add creation of `_systemprompt_test.md` in `init_project()`:

```rust
create_file_if_not_exists(
    &jobs_dir.join("_systemprompt_test.md"),
    DEFAULT_TEST_PROMPT,
)?;
```

### Default Test Prompt Content

Add a new constant `DEFAULT_TEST_PROMPT` with content for test generation:

```rust
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

```language
// Your generated tests here
```
"#;
```

### Update Next Steps Output

Update the printed next steps to mention TDD:

```rust
println!("\nNext steps:");
println!("1. Edit jobs/_systemprompt_create.md to customize code generation instructions");
println!("2. Edit jobs/_systemprompt_verify.md to customize verification instructions");
println!("3. Edit jobs/_systemprompt_test.md to customize test generation (for TDD workflow)");
println!("4. Create job files in the jobs/ directory");
println!("5. Run 'worksplit run' to process jobs");
println!("\nTip: Add 'test_file: <filename>' to job frontmatter to enable TDD workflow");
```

### Update Manager Instructions

Update `DEFAULT_MANAGER_INSTRUCTION` to document TDD workflow. Add this new section after the existing "## Best Practices" section:

```markdown
## TDD Workflow

To enable Test-Driven Development for a job, add the `test_file` field to the frontmatter:

```yaml
---
context_files:
  - src/models/user.rs
output_dir: src/services/
output_file: user_service.rs
test_file: user_service_test.rs
---
```

When `test_file` is specified:
1. Tests are generated FIRST based on requirements
2. Implementation is then generated
3. Implementation is verified against requirements
4. (Future) Tests are executed against implementation

This follows TDD principles: write tests first, then implementation.
```

### File Creation Order

The files should be created in this order in `init_project()`:
1. `_systemprompt_create.md`
2. `_systemprompt_verify.md`
3. `_systemprompt_test.md` (NEW)
4. `_managerinstruction.md`
5. `_jobstatus.json`
6. `worksplit.toml` (in project_root, not jobs_dir)
7. `example_001.md`

### Optional: Add TDD Example Job

Add a second example job that demonstrates TDD workflow:

```rust
create_file_if_not_exists(
    &jobs_dir.join("example_002_tdd.md"),
    DEFAULT_TDD_EXAMPLE_JOB,
)?;
```

With this constant:

```rust
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
```

## Implementation Notes

- Keep all existing constants unchanged except where noted
- Ensure backward compatibility - existing projects should still work
- The test prompt file is created by init but is optional for running (only needed if jobs use TDD)
- Add the new constant in the same style as existing constants
