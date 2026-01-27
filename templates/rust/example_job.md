---
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
