---
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
