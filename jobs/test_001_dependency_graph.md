---
mode: edit
context_files:
  - src/core/dependency.rs
target_files:
  - src/core/dependency.rs
output_dir: src/core/
output_file: dependency.rs
---

# Add Tests for Dependency Graph

## Overview

Add comprehensive unit tests for the DependencyGraph struct in dependency.rs.

## Formatting Notes

- Uses 4-space indentation
- The file currently has 134 lines
- Last line is a closing brace `}`
- No existing tests module

## Edit Location

At the end of the file (line 134), after the closing brace of the `impl DependencyGraph` block, add a tests module.

## Tests to Add

1. `test_empty_graph` - Empty graph returns empty execution groups
2. `test_no_dependencies` - Jobs with no dependencies can run in parallel
3. `test_simple_dependency` - Job B depends on Job A, runs after
4. `test_chain_dependency` - A -> B -> C chain executes in order
5. `test_parallel_with_shared_dependency` - B and C both depend on A, B and C can run in parallel after A
6. `test_build_from_context_files` - Dependencies detected from context_files
7. `test_build_from_target_files` - Dependencies detected from edit mode target_files

## FIND/REPLACE Instructions

Add the tests module at the end of the file. The FIND block should match the end of the `impl DependencyGraph` block including the `get_dependencies` function.

Note: The file ends with:
```rust
    pub fn get_dependencies(&self, job_id: &str) -> Vec<String> {
        self.dependencies
            .get(job_id)
            .map(|deps| deps.iter().cloned().collect())
            .unwrap_or_default()
    }
}
```

Add tests after this closing brace.
