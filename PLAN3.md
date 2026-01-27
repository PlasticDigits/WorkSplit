# PLAN3: WorkSplit Improvements

Based on real-world usage experience, this plan addresses key pain points and adds missing features.

---

## Task 1: Auto-Detect Large Files and Fail with Split Instructions

### Problem
When any file (context or target) exceeds the 900 LOC limit, WorkSplit should fail early with clear instructions for the manager to create a split job first.

### Implementation

**File**: `src/core/jobs.rs` and `src/core/runner/mod.rs`

Add a pre-flight check before running any job:

```rust
/// Check if any file exceeds LOC limit and return actionable error
pub fn check_file_sizes(&self, job: &Job) -> Result<(), WorkSplitError> {
    let limit = 900;
    
    // Check context files
    for path in &job.metadata.context_files {
        let content = self.load_context_file(path)?;
        let lines = content.lines().count();
        if lines > limit {
            return Err(WorkSplitError::FileTooLarge {
                path: path.clone(),
                lines,
                limit,
                suggestion: format!(
                    "Manager must create and run a split job first:\n\
                     worksplit new-job split_{} --template split --targets {}\n\
                     Then update this job to reference the smaller modules.",
                    path.file_stem().unwrap_or_default().to_string_lossy(),
                    path.display()
                ),
            });
        }
    }
    
    // Check target files (for edit mode)
    for path in job.metadata.get_target_files() {
        // ... same check
    }
    
    Ok(())
}
```

**Error Type** (add to `src/error.rs`):

```rust
#[error("File too large: {path} has {lines} lines (max: {limit})\n\n{suggestion}")]
FileTooLarge {
    path: PathBuf,
    lines: usize,
    limit: usize,
    suggestion: String,
},
```

### Error Message Format

```
ERROR: src/core/parser.rs is 1623 LOC, which exceeds the 900 LOC limit.

Manager action required:
1. Create a split job: worksplit new-job split_parser --template split --targets src/core/parser.rs
2. Run: worksplit run
3. Update jobs referencing parser.rs to use the new modules
4. Re-run this job
```

---

## Task 2: Dry Run Mode

### Problem
Managers want to preview what jobs would run without actually executing them.

### Implementation

**CLI** (`src/main.rs`):

```rust
/// Run pending jobs
Run {
    /// Only run a specific job
    #[arg(long)]
    job: Option<String>,
    
    /// Preview what would run without executing
    #[arg(long)]
    dry_run: bool,
    
    // ... existing args
},
```

**Runner** (`src/commands/run.rs`):

```rust
if options.dry_run {
    println!("=== DRY RUN ===\n");
    println!("Would process {} job(s):\n", pending.len());
    for job in &pending {
        println!("  {} [{}]", job.id, job.metadata.mode);
        println!("    Context: {:?}", job.metadata.context_files);
        println!("    Output:  {}", job.metadata.output_path().display());
        if let Some(targets) = &job.metadata.target_files {
            println!("    Targets: {:?}", targets);
        }
        println!();
    }
    println!("Run without --dry-run to execute.");
    return Ok(());
}
```

### Usage

```bash
worksplit run --dry-run
# Output:
# === DRY RUN ===
# Would process 3 job(s):
#   enhance_001_models [replace]
#     Context: ["src/models/status.rs"]
#     Output:  src/models/status.rs
```

---

## Task 3: Better Reset Workflow

### Problem
Currently requires manually editing `_jobstatus.json`. Need a proper `reset` command.

### Implementation

**CLI** (`src/main.rs`):

```rust
/// Reset job status
Reset {
    /// Job ID to reset (or "all" for all failed jobs)
    job: String,
    
    /// Reset all jobs matching status
    #[arg(long)]
    status: Option<String>,  // "fail", "partial"
},
```

**Command** (`src/commands/reset.rs`):

```rust
pub fn reset_jobs(
    project_root: &PathBuf,
    job_id: &str,
    status_filter: Option<&str>,
) -> Result<(), WorkSplitError> {
    let mut status_manager = StatusManager::new(&project_root.join("jobs"))?;
    
    if job_id == "all" {
        let filter = status_filter.unwrap_or("fail");
        let to_reset: Vec<String> = status_manager
            .all_entries()
            .iter()
            .filter(|e| match filter {
                "fail" => e.status == JobStatus::Fail,
                "partial" => e.status == JobStatus::Partial,
                _ => false,
            })
            .map(|e| e.id.clone())
            .collect();
        
        for id in &to_reset {
            status_manager.reset_job(id)?;
            println!("Reset: {}", id);
        }
        println!("\nReset {} job(s). Run 'worksplit run' to re-execute.", to_reset.len());
    } else {
        status_manager.reset_job(job_id)?;
        println!("Reset: {}", job_id);
    }
    
    Ok(())
}
```

### Usage

```bash
# Reset specific job
worksplit reset my_job_001

# Reset all failed jobs
worksplit reset all

# Reset all partial jobs
worksplit reset all --status partial
```

### Documentation Update

Add to README and _managerinstruction.md:

```markdown
## Resetting Jobs

# Reset a specific failed job
worksplit reset my_job_001

# Reset all failed jobs  
worksplit reset all

# Then re-run
worksplit run
```

---

## Task 4: Dependency Ordering

### Problem
Jobs with dependencies should run in order. Currently they run in alphabetical order.

### Implementation

**Frontmatter** (add to job.rs):

```yaml
---
depends_on:
  - enhance_001_models
  - enhance_002_parser
output_dir: src/
output_file: runner.rs
---
```

**Dependency Resolution** (`src/core/dependency.rs`):

```rust
/// Topological sort of jobs based on depends_on
pub fn order_by_dependencies(jobs: &[Job]) -> Result<Vec<&Job>, WorkSplitError> {
    let mut graph: HashMap<&str, Vec<&str>> = HashMap::new();
    let mut in_degree: HashMap<&str, usize> = HashMap::new();
    
    // Build graph
    for job in jobs {
        in_degree.entry(&job.id).or_insert(0);
        if let Some(deps) = &job.metadata.depends_on {
            for dep in deps {
                graph.entry(dep.as_str()).or_default().push(&job.id);
                *in_degree.entry(&job.id).or_insert(0) += 1;
            }
        }
    }
    
    // Kahn's algorithm
    let mut queue: VecDeque<&str> = in_degree
        .iter()
        .filter(|(_, &deg)| deg == 0)
        .map(|(&id, _)| id)
        .collect();
    
    let mut result = Vec::new();
    while let Some(id) = queue.pop_front() {
        if let Some(job) = jobs.iter().find(|j| j.id == id) {
            result.push(job);
        }
        if let Some(neighbors) = graph.get(id) {
            for &neighbor in neighbors {
                let deg = in_degree.get_mut(neighbor).unwrap();
                *deg -= 1;
                if *deg == 0 {
                    queue.push_back(neighbor);
                }
            }
        }
    }
    
    if result.len() != jobs.len() {
        return Err(WorkSplitError::CyclicDependency);
    }
    
    Ok(result)
}
```

### Usage

```yaml
---
depends_on:
  - models_001  # This job runs first
  - parser_001  # This job runs second
output_dir: src/
output_file: runner.rs
---

# Runner Implementation
This job uses types from models_001 and parser_001.
```

### CLI

```bash
# Show dependency graph
worksplit deps

# Output:
# models_001 (no dependencies)
# parser_001 (no dependencies)
# runner_001
#   └── depends_on: models_001, parser_001
```

---

## Task 5: Configurable Build/Test Verification

### Problem
Current verification only uses Ollama. Should also run actual build/test commands to catch real compilation errors.

### Implementation

**Config** (`worksplit.toml`):

```toml
[build]
# Command to verify code compiles (optional)
build_command = "cargo check"

# Command to run tests (optional)
test_command = "cargo test"

# Whether to run build verification after generation
verify_build = true

# Whether to run tests after generation
verify_tests = false
```

**Runner Integration** (`src/core/runner/mod.rs`):

```rust
async fn verify_with_build(&self, job: &Job, files: &[(PathBuf, String)]) -> Result<(), WorkSplitError> {
    if !self.config.build.verify_build {
        return Ok(());
    }
    
    let Some(ref cmd) = self.config.build.build_command else {
        return Ok(());
    };
    
    info!("Running build verification: {}", cmd);
    
    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .current_dir(&self.project_root)
        .output()?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        // Feed error back to Ollama for retry
        let error_context = format!(
            "Build failed after generating files:\n\nFiles generated:\n{}\n\nBuild output:\n{}\n{}",
            files.iter().map(|(p, _)| p.display().to_string()).collect::<Vec<_>>().join("\n"),
            stdout,
            stderr
        );
        
        return Err(WorkSplitError::BuildFailed {
            command: cmd.clone(),
            output: error_context,
        });
    }
    
    Ok(())
}
```

**Retry with Build Error**:

When build fails, feed the error to Ollama for a fix attempt:

```rust
if let Err(WorkSplitError::BuildFailed { output, .. }) = self.verify_with_build(&job, &files).await {
    info!("Build failed, attempting fix with Ollama...");
    
    let fix_prompt = format!(
        "[BUILD ERROR]\nThe generated code failed to compile:\n\n{}\n\n\
         [INSTRUCTIONS]\nFix the compilation errors. Output the corrected code.",
        output
    );
    
    // Retry generation with error context
    let fixed = self.ollama.generate_with_retry(...).await?;
    // ... apply fixed code
}
```

### Language Examples

**Rust:**
```toml
[build]
build_command = "cargo check"
test_command = "cargo test"
```

**Python:**
```toml
[build]
build_command = "python -m py_compile *.py"
test_command = "pytest"
```

**TypeScript:**
```toml
[build]
build_command = "tsc --noEmit"
test_command = "npm test"
```

**Go:**
```toml
[build]
build_command = "go build ./..."
test_command = "go test ./..."
```

---

## Task 6: Preventing Dead Code (Caller Integration Problem)

### Problem
Jobs generate new types/functions but don't update callers to use them. Result: code compiles but new features are unused.

### Analysis

This happens because:
1. Job A creates `PartialEditState` struct
2. Job B creates `process_edit_mode()` that uses it
3. But the main `run_job()` function is never updated to call the new code

### Solution Ideas

#### Option A: Mandatory Caller Update Jobs

Add guidance to `_managerinstruction.md`:

```markdown
## Preventing Dead Code

When adding new functionality, ALWAYS create two jobs:

1. **Implementation job**: Creates the new types/functions
2. **Integration job**: Updates callers to use the new code

Example:
- `feature_001_types.md` - Creates new structs
- `feature_001_integration.md` - Updates main.rs, runner.rs to use them

The integration job should use `mode: edit` to surgically add calls to the new code.
```

#### Option B: Dead Code Detection (Language-Agnostic)

**Two approaches:**

##### B1: Configurable Lint Command

Add a `lint_command` to `worksplit.toml` that outputs unused code warnings:

```toml
[build]
build_command = "cargo check"
test_command = "cargo test"

# Optional: lint command for dead code detection
# Should output warnings to stdout/stderr
lint_command = "cargo check 2>&1 | grep 'never used'"
```

Language examples:

```toml
# Rust
lint_command = "cargo check 2>&1 | grep -E '(never used|unused)'

# Python (with pylint)
lint_command = "pylint --disable=all --enable=unused-import,unused-variable src/"

# TypeScript (with eslint)
lint_command = "eslint --rule 'no-unused-vars: warn' src/"

# Go
lint_command = "go vet ./... 2>&1 | grep -i unused"
```

WorkSplit runs the command and checks if output mentions files we just generated:

```rust
async fn check_dead_code(&self, files: &[(PathBuf, String)]) -> Vec<String> {
    let Some(ref cmd) = self.config.build.lint_command else {
        return Vec::new();  // No lint command configured
    };
    
    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .current_dir(&self.project_root)
        .output()?;
    
    let combined = format!("{}{}", 
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    
    // Filter to warnings that mention our generated files
    files.iter()
        .filter_map(|(path, _)| {
            let filename = path.file_name()?.to_string_lossy();
            if combined.contains(&*filename) {
                Some(format!("WARNING: Possible unused code in {}", path.display()))
            } else {
                None
            }
        })
        .collect()
}
```

##### B2: Ollama-Based Detection (Universal Fallback)

For languages without good lint tools, use Ollama to detect dead code:

```rust
async fn check_dead_code_with_ollama(&self, files: &[(PathBuf, String)]) -> Vec<String> {
    // Extract public symbols from generated files
    let symbols = extract_public_symbols(files);  // function names, class names, etc.
    
    let prompt = format!(
        "[DEAD CODE CHECK]\n\n\
         These items were just generated:\n{}\n\n\
         Scan the codebase and list any items that are NEVER called or used.\n\
         Output format: one item per line, or 'NONE' if all items are used.",
        symbols.join("\n")
    );
    
    let response = self.ollama.generate(...).await?;
    
    if response.trim() == "NONE" {
        Vec::new()
    } else {
        response.lines()
            .map(|l| format!("WARNING: {}", l.trim()))
            .collect()
    }
}
```

This is slower but works for any language.

##### Output (either approach):

```
=== Dead Code Warnings ===
WARNING: Possible unused code in src/core/runner/edit.rs
WARNING: PartialEditState::new() may not be called

Suggested action: Create an integration job to wire up the new code.
```

#### Option C: Two-Phase Jobs

New job mode that generates implementation AND updates callers:

```yaml
---
mode: implement_and_integrate
output_files:
  - src/core/partial.rs       # New implementation
target_files:
  - src/core/runner/mod.rs    # Caller to update
---

# Add Partial Completion

## Implementation (src/core/partial.rs)
Create PartialEditState struct...

## Integration (src/core/runner/mod.rs)  
Update run_job() to use PartialEditState...
```

This forces the LLM to think about both implementation AND usage in the same context.

#### Option D: Ollama Integration Check

After generation, have Ollama verify that the new code is actually called:

```
[INTEGRATION CHECK]

You generated these new items:
- PartialEditState::new()
- process_edit_mode()

Verify: Are these items called from somewhere in the codebase?
If not, generate edit instructions to integrate them.
```

### Recommended Approach

**Combine A + B + D:**

1. **Documentation (A)**: Update `_managerinstruction.md` with "Implementation + Integration" job pattern
2. **Detection (B)**: Use configurable `lint_command` when available, Ollama fallback otherwise
3. **Auto-fix (D)**: When dead code detected, offer to generate integration edits via Ollama
4. When dead code is detected, log actionable message:

```
WARNING: 3 new items are unused. Create integration job:

worksplit new-job integrate_partial --template edit --targets src/core/runner/mod.rs

Add these items to the job instructions:
- Call PartialEditState::new() in process_edit_mode()
- Use EditModeResult.partial_state in run_job()
```

---

## Implementation Order

| Priority | Task | Effort | Impact |
|----------|------|--------|--------|
| 1 | Task 5: Build/Test Verification | Medium | High - catches real errors |
| 2 | Task 1: Large File Detection | Low | High - prevents wasted runs |
| 3 | Task 3: Reset Command | Low | Medium - better UX |
| 4 | Task 2: Dry Run | Low | Medium - better UX |
| 5 | Task 4: Dependency Ordering | Medium | Medium - correctness |
| 6 | Task 6: Dead Code Prevention | High | High - quality |

---

## Files to Modify

| Task | Files |
|------|-------|
| Task 1 | `src/error.rs`, `src/core/jobs.rs`, `src/core/runner/mod.rs` |
| Task 2 | `src/main.rs`, `src/commands/run.rs` |
| Task 3 | `src/main.rs`, `src/commands/reset.rs` (new), `src/commands/mod.rs` |
| Task 4 | `src/models/job.rs`, `src/core/dependency.rs`, `src/core/runner/mod.rs` |
| Task 5 | `src/models/config.rs`, `src/core/runner/mod.rs`, `worksplit.toml` |
| Task 6 | `src/core/runner/mod.rs`, `jobs/_managerinstruction.md` |

---

## Documentation Updates Required

After implementation, update:
- `README.md` - Add reset, dry-run, dependency, build verification sections
- `jobs/_managerinstruction.md` - Add integration job pattern, split job guidance
- `worksplit.toml` - Add [build] section with examples
