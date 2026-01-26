# WorkSplit Improvement Plan 2

## Goal

Reduce job failure rate and improve recovery from partial failures, based on lessons learned from AI manager usage.

## Current Issues

1. **Edit mode fails with many similar patterns**: When adding a struct field, updating 20+ test fixtures via edit mode has ~30% success rate
2. **No partial completion**: If a job fails midway, all progress is lost
3. **No dry-run mode**: Can't preview edits before applying
4. **Manual recovery is expensive**: When edit mode fails, manager must make many manual fixes
5. **Test fixtures are tedious**: Struct changes require updating many test literals
6. **No batch text operations**: Simple "add X after every Y" isn't possible

## Already Implemented (PLAN.md)

- `--quiet` flag for suppressing output
- `--summary` flag for single-line status
- `--json` flag for machine-readable output
- `verify: false` option for skipping verification
- Job quality validation warnings
- Standardized exit codes

---

## Implementation Tasks

Each task below is designed to be executable as a WorkSplit job.

---

### Task 1: Add dry-run mode for edit jobs

**Job**: `enhance_001_dry_run.md`

Add `--dry-run` flag that shows what edits would be applied without applying them.

**Output format:**
```
[DRY RUN] Job: upgrade_001_main_rs
  Would apply 3 edits to src/main.rs:
    - Line 20: Add quiet flag to Cli struct
    - Line 45: Add summary flag to Status subcommand
    - Line 120: Update logging setup
  Would apply 1 edit to src/commands/run.rs:
    - Line 30: Add quiet field to RunOptions
```

**Files to modify**:
- `src/commands/run.rs` - Add `--dry-run` flag handling
- `src/main.rs` - Add CLI arg
- `src/core/runner/edit.rs` - Add dry-run logic

**Context files**:
- `src/commands/run.rs`
- `src/core/runner/edit.rs`

---

### Task 2: Add partial completion for edit mode

**Job**: `enhance_002_partial_edits.md`

When an edit job partially succeeds (some edits apply, some fail), save the successful edits and report which failed.

**Current behavior:**
- Any FIND failure → entire job fails, no files changed

**New behavior:**
- Apply successful edits
- Report failed edits with line number hints
- Mark job as `partial` status
- Allow `--continue` to retry just the failed edits

**Files to modify**:
- `src/core/runner/edit.rs` - Apply edits incrementally
- `src/models/status.rs` - Add `Partial` status variant
- `src/core/status.rs` - Track partial completions

**Context files**:
- `src/core/runner/edit.rs`
- `src/models/status.rs`

---

### Task 3: Add `replace_pattern` mode

**Job**: `enhance_003_replace_pattern.md`

New mode for batch text replacements without needing unique FIND contexts.

**Job syntax:**
```yaml
---
mode: replace_pattern
target_files:
  - src/models/job.rs
output_dir: src/models/
output_file: job.rs
---

# Add verify field to all JobMetadata literals

## Pattern
AFTER:
    target_file: None,
INSERT:
    verify: true,

## Scope
Only in #[cfg(test)] blocks
```

**How it works:**
1. Parse AFTER/INSERT instructions
2. Find all occurrences of AFTER pattern
3. Insert the INSERT text after each occurrence
4. Optionally scope to specific regions (e.g., test blocks)

**Files to modify**:
- `src/models/job.rs` - Add `ReplacePattern` to OutputMode enum
- `src/core/parser.rs` - Parse AFTER/INSERT syntax
- `src/core/runner/mod.rs` - Handle replace_pattern mode

**Context files**:
- `src/models/job.rs`
- `src/core/parser.rs`

---

### Task 4: Add `--continue` flag for partial jobs

**Job**: `enhance_004_continue_flag.md`

Allow resuming a job that partially completed.

**Usage:**
```bash
worksplit run --continue upgrade_001_main_rs
```

**Behavior:**
1. Load the job's saved partial state
2. Show which edits succeeded and which failed
3. Re-attempt only the failed edits
4. Update status based on retry results

**Files to modify**:
- `src/commands/run.rs` - Add `--continue` flag
- `src/main.rs` - Add CLI arg
- `src/core/status.rs` - Store partial edit state

**Context files**:
- `src/commands/run.rs`
- `src/core/status.rs`

---

### Task 5: Add fuzzy FIND matching with confirmation

**Job**: `enhance_005_fuzzy_find.md`

When FIND doesn't match exactly, attempt fuzzy matching and report alternatives.

**Current behavior:**
```
Error: FIND text not found in src/main.rs
```

**New behavior:**
```
Error: FIND text not found in src/main.rs
  Possible matches:
    - Line 45 (95% match): differs in whitespace
    - Line 120 (80% match): similar structure

  Hint: Use --apply-fuzzy to apply the closest match
```

**Files to modify**:
- `src/core/parser.rs` - Add fuzzy matching with similarity scores
- `src/core/runner/edit.rs` - Report fuzzy matches on failure

**Context files**:
- `src/core/parser.rs`
- `src/core/runner/edit.rs`

---

### Task 6: Add test fixture generator mode

**Job**: `enhance_006_fixture_update.md`

Special mode for updating test struct literals when a struct changes.

**Job syntax:**
```yaml
---
mode: update_fixtures
target_files:
  - src/models/job.rs
struct_name: JobMetadata
new_field: "verify: true"
---

# Add verify field to all JobMetadata test fixtures

Insert `verify: true` as the last field in every `JobMetadata { ... }` 
struct literal in the #[cfg(test)] module.
```

**How it works:**
1. Parse the target file's AST
2. Find all struct literals of the specified type
3. Add the new field to each one
4. Preserve formatting and indentation

**Files to modify**:
- `src/models/job.rs` - Add `UpdateFixtures` mode
- `src/core/parser.rs` - Add struct literal detection
- `src/core/runner/mod.rs` - Handle update_fixtures mode

**Context files**:
- `src/models/job.rs`
- `src/core/parser.rs`

---

### Task 7: Add job dependency graph visualization

**Job**: `enhance_007_dep_graph.md`

Add `worksplit deps` command to visualize job dependencies.

**Output:**
```
worksplit deps

Job Dependencies:
  auth_001_user_model
    └── auth_002_password_hasher (depends on src/models/user.rs)
        └── auth_003_session_service (depends on src/auth/hasher.rs)

Execution Groups:
  Group 1: [auth_001_user_model]
  Group 2: [auth_002_password_hasher]
  Group 3: [auth_003_session_service]
```

**Files to modify**:
- `src/commands/mod.rs` - Add deps command
- `src/main.rs` - Add CLI subcommand
- New file: `src/commands/deps.rs`

**Context files**:
- `src/core/dependency.rs`
- `src/main.rs`

---

### Task 8: Improve error messages with suggestions

**Job**: `enhance_008_error_suggestions.md`

Add actionable suggestions to common error messages.

**Current:**
```
Error: Edit failed: FIND text not found
```

**New:**
```
Error: Edit failed: FIND text not found in src/main.rs

Suggestions:
  1. Check whitespace: file uses 4 spaces, your FIND may use tabs
  2. Include more context: your FIND appears on lines 45, 78, 120
  3. Consider replace mode: this job has 15+ edits, replace is safer

See: worksplit help edit-troubleshooting
```

**Files to modify**:
- `src/error.rs` - Add suggestion generation
- `src/core/runner/edit.rs` - Collect context for suggestions

**Context files**:
- `src/error.rs`
- `src/core/runner/edit.rs`

---

## Execution Plan

Jobs should be executed in order:

```bash
# Phase 1: Better error handling
worksplit run --job enhance_005_fuzzy_find
worksplit run --job enhance_008_error_suggestions

# Phase 2: Partial completion
worksplit run --job enhance_002_partial_edits
worksplit run --job enhance_004_continue_flag

# Phase 3: New modes
worksplit run --job enhance_003_replace_pattern
worksplit run --job enhance_006_fixture_update

# Phase 4: Tooling
worksplit run --job enhance_001_dry_run
worksplit run --job enhance_007_dep_graph
```

## Success Criteria

After all tasks complete:

1. `worksplit run --dry-run` shows planned edits without applying
2. Failed edit jobs report which edits succeeded and which failed
3. `worksplit run --continue` retries only failed edits
4. `replace_pattern` mode handles batch "add X after Y" operations
5. `update_fixtures` mode handles struct literal updates
6. Fuzzy matching suggests corrections for near-miss FINDs
7. `worksplit deps` visualizes job dependencies
8. Error messages include actionable suggestions

## Estimated Complexity

| Task | Files Changed | Complexity | LOC Estimate |
|------|---------------|------------|--------------|
| 1. Dry-run mode | 3 | Medium | ~80 |
| 2. Partial edits | 3 | High | ~150 |
| 3. Replace pattern | 3 | High | ~200 |
| 4. Continue flag | 3 | Medium | ~60 |
| 5. Fuzzy matching | 2 | Medium | ~100 |
| 6. Fixture updates | 3 | High | ~180 |
| 7. Dep graph viz | 3 | Low | ~80 |
| 8. Error suggestions | 2 | Low | ~50 |

Total: ~900 lines across 8 jobs

## Priority Ranking

Based on impact on AI manager efficiency:

1. **Task 5 (Fuzzy matching)** - Immediate help for edit failures
2. **Task 3 (Replace pattern)** - Solves the test fixture problem
3. **Task 2 (Partial edits)** - Reduces wasted work
4. **Task 8 (Error suggestions)** - Faster debugging
5. **Task 1 (Dry-run)** - Better planning
6. **Task 4 (Continue flag)** - Recovery improvement
7. **Task 6 (Fixture updates)** - Nice to have
8. **Task 7 (Dep graph)** - Nice to have
