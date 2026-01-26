---
context_files: []
output_dir: src/
output_file: greeting.rs
---

# Create Greeting Module

## Overview
Create a simple greeting module that provides functions for generating personalized greetings.

## Requirements
- Create greeting functions that accept a name parameter
- Support different greeting styles (casual, formal, time-based)
- Return formatted greeting strings
- Include proper documentation

## Functions to Implement

### `greet(name: &str) -> String`
Returns a simple greeting: "Hello, {name}!"

### `greet_formal(name: &str) -> String`
Returns a formal greeting: "Good day, {name}. How may I assist you?"

### `greet_casual(name: &str) -> String`
Returns a casual greeting: "Hey {name}! What's up?"

### `greet_with_time(name: &str, hour: u8) -> String`
Returns a time-appropriate greeting:
- Hours 5-11: "Good morning, {name}!"
- Hours 12-17: "Good afternoon, {name}!"
- Hours 18-21: "Good evening, {name}!"
- Hours 22-4: "Good night, {name}!"

## Error Handling
- All functions are infallible (no Result needed)
- The `hour` parameter for `greet_with_time` should be clamped to 0-23 range

## Example Usage

```rust
use greeting::*;

let msg = greet("World");
assert_eq!(msg, "Hello, World!");

let formal = greet_formal("Dr. Smith");
assert_eq!(formal, "Good day, Dr. Smith. How may I assist you?");

let morning = greet_with_time("Alice", 9);
assert_eq!(morning, "Good morning, Alice!");
```

## Notes
- Include unit tests for all functions
- Document each function with doc comments
