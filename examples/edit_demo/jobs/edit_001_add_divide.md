---
mode: edit
context_files:
  - src/calculator.rs
target_files:
  - src/calculator.rs
output_dir: src/
output_file: calculator.rs
---

# Add Divide Function to Calculator

## Overview

Add a `divide` function to the calculator module that safely handles division by zero.

## Requirements

1. Add a new `divide` function after the `multiply` function:
   - Signature: `pub fn divide(a: i32, b: i32) -> Option<i32>`
   - Returns `None` if b is zero
   - Returns `Some(a / b)` otherwise

2. Add corresponding tests in the test module:
   - Test normal division
   - Test division by zero returns None
   - Test negative number division

## Formatting Notes

- Uses 4-space indentation
- Doc comments use `///`
- The file uses `Option<i32>` for fallible operations

## Edit Locations

1. After the multiply function (around line 14-16), add the divide function
2. In the tests module, after test_multiply (around line 36-38), add test_divide

## IMPORTANT: How to format edits

The FIND block must contain ONLY text that already exists. For example, to add divide after multiply:

```
FILE: src/calculator.rs
FIND:
/// Multiplies two numbers.
pub fn multiply(a: i32, b: i32) -> i32 {
    a * b
}

#[cfg(test)]
REPLACE:
/// Multiplies two numbers.
pub fn multiply(a: i32, b: i32) -> i32 {
    a * b
}

/// Divides a by b, returning None if b is zero.
pub fn divide(a: i32, b: i32) -> Option<i32> {
    if b == 0 { None } else { Some(a / b) }
}

#[cfg(test)]
END
```

Notice: The FIND block contains the multiply function AND the `#[cfg(test)]` line that follows it - both of which EXIST in the file. The new divide function appears ONLY in REPLACE.
