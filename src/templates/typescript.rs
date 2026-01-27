//! TypeScript-specific templates for WorkSplit

use super::Templates;

/// Get TypeScript-specific templates
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

pub const CREATE_PROMPT: &str = r#"# TypeScript Code Generation System Prompt

You are a code generation assistant specializing in TypeScript. Your task is to generate high-quality, type-safe TypeScript code based on the provided context and instructions.

## Guidelines

1. **Output Format**: Output ONLY the code, wrapped in a markdown code fence with the `typescript` language tag. Do not include explanations, comments about what you're doing, or any other text outside the code fence.

2. **Line Limit**: Your output must not exceed 900 lines of code. If the task requires more, focus on the most critical functionality.

3. **Code Style**: Follow TypeScript best practices and patterns from the context files provided. Match the existing conventions for:
   - Naming (camelCase for variables/functions, PascalCase for types/classes)
   - Formatting and indentation
   - Error handling patterns (try/catch, Result types, error classes)
   - Documentation style (JSDoc comments)

4. **Imports**: Include all necessary imports at the top of the file. Prefer named imports over default imports.

5. **Type Safety**: 
   - Use explicit type annotations for function parameters and return types
   - Avoid `any` type - use `unknown` if type is truly unknown
   - Use strict null checks - handle `null` and `undefined` explicitly
   - Leverage union types, generics, and utility types appropriately

6. **Documentation**: Add JSDoc comments for all exported items (functions, classes, interfaces, types).

7. **Error Handling**: Use proper error handling with try/catch. Consider creating custom error classes for domain errors.

8. **Async/Await**: Use async/await for asynchronous operations. Handle Promise rejections properly.

9. **Module Format**: Use ESM (ES Modules) syntax with `import`/`export`.

## Response Format

Your response should be ONLY a code fence containing the complete file content:

"#;

pub const VERIFY_PROMPT: &str = r#"# TypeScript Code Verification System Prompt

You are a TypeScript code review assistant. Your task is to verify that the generated code meets the requirements and follows TypeScript best practices.

## Verification Checklist

1. **Syntax**: Is the code syntactically correct TypeScript?
2. **Completeness**: Does the code implement all requirements from the instructions?
3. **Correctness**: Does the logic appear correct? Are there any obvious bugs?
4. **Type Safety**: 
   - Are types properly annotated?
   - Is `any` avoided?
   - Are null/undefined handled correctly?
5. **Error Handling**: Are errors handled appropriately with try/catch?
6. **Async Operations**: Are Promises and async/await used correctly?
7. **Style Consistency**: Does the code follow TypeScript idioms and match context file patterns?
8. **Documentation**: Are exported items documented with JSDoc?
9. **Imports**: Are all imports valid and used?

## Response Format

Your response MUST start with either:

- `PASS` - if the code meets all requirements
- `FAIL: <reason>` - if the code has issues

Examples:
- `PASS`
- `PASS - Code looks good, all requirements met.`
- `FAIL: Missing error handling for API call`
- `FAIL: The getUser method is not implemented`
- `FAIL: Using 'any' type instead of proper typing`
- `FAIL: Missing null check for optional parameter`

Be strict but fair. Minor style issues should not cause a failure if the code is functionally correct.
"#;

pub const TEST_PROMPT: &str = r#"# TypeScript Test Generation System Prompt

You are a test generation assistant specializing in TypeScript. Your task is to generate comprehensive tests based on the provided requirements BEFORE the implementation exists.

## TDD Approach

You are generating tests first (Test-Driven Development). The implementation does not exist yet. Your tests should:
1. Define the expected behavior based on requirements
2. Cover happy path scenarios
3. Cover edge cases and error conditions
4. Be runnable once the implementation is created

## Guidelines

1. **Output Format**: Output ONLY the test code, wrapped in a markdown code fence with the `typescript` language tag.

2. **Test Framework**: Use Jest/Vitest patterns with describe/it/expect:
   ```typescript
   import { functionName } from './module';

   describe('functionName', () => {
     it('should do something specific', () => {
       expect(functionName(input)).toBe(expected);
     });
   });
   ```

3. **Test Coverage**: Generate tests for:
   - All functions/methods mentioned in the requirements
   - Input validation and edge cases
   - Error handling scenarios (thrown errors, rejected promises)
   - Boundary conditions
   - Async operations

4. **Assertions**: Use clear Jest/Vitest matchers:
   - `expect(x).toBe(y)` for primitives
   - `expect(x).toEqual(y)` for objects/arrays
   - `expect(x).toBeTruthy()` / `expect(x).toBeFalsy()`
   - `expect(() => fn()).toThrow()`
   - `await expect(asyncFn()).resolves.toBe(x)`
   - `await expect(asyncFn()).rejects.toThrow()`

5. **Test Names**: Use descriptive names:
   - `it('returns greeting with provided name')`
   - `it('handles empty string input')`
   - `it('throws error when dividing by zero')`

6. **Async Tests**: For async functions, use async/await:
   ```typescript
   it('fetches user data', async () => {
     const user = await getUser(1);
     expect(user.name).toBe('Alice');
   });
   ```

## Response Format

Your response should be ONLY a code fence containing the complete test file:

"#;

pub const MANAGER_INSTRUCTION: &str = r#"# Manager Instructions for Creating TypeScript Job Files

This file explains how to create job files for a TypeScript WorkSplit project.

## Job File Format

Each job file uses YAML frontmatter followed by markdown instructions:

```yaml
---
context_files:
  - src/models/user.ts
output_dir: src/services/
output_file: userService.ts
---
```

## Frontmatter Fields

- `context_files`: List of TypeScript files to include as context (max 2, each under 1000 lines)
- `output_dir`: Directory where the output `.ts` file should be created
- `output_file`: Name of the TypeScript file to generate
- `test_file`: Name of the test file to generate (for TDD workflow)

## TypeScript-Specific Best Practices

### 1. Keep Jobs Small
Each job should generate at most 900 lines of TypeScript code. Break larger features into multiple jobs.

### 2. Choose Context Files Wisely
Include files that:
- Define interfaces/types the generated code will use
- Show class patterns to follow
- Contain utility functions to use
- Show module patterns to match

### 3. Write Clear Instructions
- Be specific about what to implement
- List required methods/functions explicitly
- Specify error handling expectations
- Mention any generics or type constraints needed
- Reference interfaces to implement

### 4. Naming Convention
Use descriptive job IDs with category prefix:
- `auth_001_user_model.md`
- `auth_002_session_service.md`
- `api_001_user_endpoints.md`

### 5. Module Structure
Consider module organization:
- `index.ts` files for barrel exports
- Feature modules grouped logically
- Types in separate `.types.ts` files if complex

## TDD Workflow

To enable Test-Driven Development, add the `test_file` field:

```yaml
---
context_files: []
output_dir: src/
output_file: calculator.ts
test_file: calculator.test.ts
---
```

When `test_file` is specified:
1. Tests are generated FIRST based on requirements
2. Implementation is then generated to pass tests
3. Implementation is verified against requirements
"#;

pub const CONFIG: &str = r#"# WorkSplit Configuration

[project]
language = "typescript"

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
build_command = "npm run build"
test_command = "npm test"
verify_build = false
verify_tests = false
"#;

pub const EXAMPLE_JOB: &str = r#"---
context_files: []
output_dir: src/
output_file: hello.ts
---

# Create Hello World Module

## Requirements
- Create a simple TypeScript module with a greeting function
- The function should accept a name parameter
- Return a formatted greeting string
- Use proper TypeScript types

## Functions to Implement

1. `greet(name: string): string` - Returns "Hello, {name}!"
2. `greetWithTime(name: string, morning: boolean): string` - Returns appropriate greeting based on time

## Example Usage

```typescript
const greeting = greet("World");
// Returns: "Hello, World!"

const morningGreeting = greetWithTime("Alice", true);
// Returns: "Good morning, Alice!"
```

## Type Definitions

Consider exporting these types:
```typescript
export interface GreetingOptions {
  name: string;
  formal?: boolean;
}
```
"#;

pub const TDD_EXAMPLE_JOB: &str = r#"---
context_files: []
output_dir: src/
output_file: calculator.ts
test_file: calculator.test.ts
---

# Create Calculator Module (TDD Example)

This job demonstrates TDD workflow - tests will be generated first!

## Requirements
- Create a calculator module with basic arithmetic operations
- Support add, subtract, multiply, divide functions
- Handle division by zero by throwing an error
- Use proper TypeScript types

## Functions to Implement

1. `add(a: number, b: number): number` - Returns sum
2. `subtract(a: number, b: number): number` - Returns difference
3. `multiply(a: number, b: number): number` - Returns product
4. `divide(a: number, b: number): number` - Returns quotient, throws on division by zero

## Expected Behavior

```typescript
add(2, 3)        // returns 5
subtract(5, 3)   // returns 2
multiply(4, 5)   // returns 20
divide(10, 2)    // returns 5
divide(10, 0)    // throws Error("Division by zero")
```

## Type Definitions

Consider creating:
```typescript
export type Operation = 'add' | 'subtract' | 'multiply' | 'divide';

export interface CalculatorResult {
  operation: Operation;
  operands: [number, number];
  result: number;
}
```
"#;
