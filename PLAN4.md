# PLAN4: WorkSplit Operator Tooling

This plan adds manager/operator convenience tools that improve reliability and reduce overhead during long runs.

---

## Task 1: Cancel/Abort Running Jobs

### Problem
When a job hangs or takes too long, the manager needs a clean way to stop it without killing the whole process.

### Implementation

**CLI** (`src/main.rs`):

```rust
/// Cancel a running job (or all)
Cancel {
    /// Job ID to cancel (or "all")
    job: String,
},
```

**Command** (`src/commands/cancel.rs` - new):

- Track running job PID(s) in status manager (or a lightweight runtime registry).
- Allow canceling a single job or all running jobs.
- Update status to `fail` with a clear cancellation message.

**Runner integration** (`src/core/runner/mod.rs`):

- When starting a job, register a PID marker or in-memory map.
- On cancel, signal the process and clean up status.

### Usage

```bash
worksplit cancel my_job_001
worksplit cancel all
```

---

## Task 2: Retry Command (Reset + Run)

### Problem
Resetting and rerunning is common; the workflow is currently two commands.

### Implementation

**CLI** (`src/main.rs`):

```rust
/// Retry a job (reset + run)
Retry {
    /// Job ID to retry
    job: String,
},
```

**Command** (`src/commands/retry.rs` - new):

- Reset job status to created.
- Immediately run the job (equivalent to `reset` + `run --job`).

### Usage

```bash
worksplit retry my_job_001
```

---

## Task 3: Status Watch Mode

### Problem
Managers often want a live view of job progress without re-running `status`.

### Implementation

**CLI** (`src/main.rs`, `src/commands/status.rs`):

```rust
/// Watch status updates
#[arg(long)]
watch: bool,
```

- Poll `status --summary` every N seconds (default: 2s).
- Print updates only when the summary changes.

### Usage

```bash
worksplit status --summary --watch
```

---

## Task 4: Per-Job Timeout / Deadline

### Problem
Long jobs can stall. Managers want a hard timeout per job or for the entire run.

### Implementation

**CLI** (`src/main.rs`, `src/commands/run.rs`):

```rust
/// Per-job timeout in seconds
#[arg(long)]
job_timeout: Option<u64>,
```

**Runner** (`src/core/runner/mod.rs`):

- Wrap Ollama calls in a per-job timeout.
- Mark the job as failed with an actionable timeout message.

### Usage

```bash
worksplit run --job-timeout 300
```

---

## Implementation Order

| Priority | Task | Effort | Impact |
|----------|------|--------|--------|
| 1 | Task 4: Per-Job Timeout | Medium | High |
| 2 | Task 1: Cancel/Abort | Medium | High |
| 3 | Task 2: Retry | Low | Medium |
| 4 | Task 3: Status Watch | Low | Medium |

---

## Files to Modify

| Task | Files |
|------|-------|
| Task 1 | `src/main.rs`, `src/commands/cancel.rs` (new), `src/commands/mod.rs`, `src/core/runner/mod.rs`, `src/core/status.rs` |
| Task 2 | `src/main.rs`, `src/commands/retry.rs` (new), `src/commands/mod.rs` |
| Task 3 | `src/main.rs`, `src/commands/status.rs` |
| Task 4 | `src/main.rs`, `src/commands/run.rs`, `src/core/runner/mod.rs` |

---

## Documentation Updates Required

After implementation, update:
- `README.md` - Add cancel/retry/watch/timeout usage
- `jobs/_managerinstruction.md` - Add recommended workflow and new CLI tools

