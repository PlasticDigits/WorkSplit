---
context_files:
  - src/core/parser/extract.rs
  - src/core/parser/mod.rs
output_dir: src/core/parser/
output_file: extract.rs
---

# Verification Tiers (Severity Levels)

## Overview
Extend the verification parsing to support multiple severity levels instead of just PASS/FAIL. This provides more nuanced feedback about code quality and allows different handling based on severity.

## Problem
Currently, verification is binary (PASS or FAIL). This doesn't capture situations like:
- Code works but has style issues (warnings)
- Code might work but has potential problems (soft fail)
- Code definitely won't compile (hard fail)

## Solution
Add verification tiers that the LLM can use to communicate severity:
- `PASS` - Everything is good
- `PASS_WITH_WARNINGS` - Works but has minor issues
- `FAIL_SOFT` - Has issues but code might work
- `FAIL_HARD` - Won't compile or has critical bugs

## Requirements

### 1. Add VerificationResult enum
Add to `src/core/parser.rs`:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationResult {
    Pass,
    PassWithWarnings,
    FailSoft,
    FailHard,
}

impl VerificationResult {
    /// Returns true if the job should be marked as passed
    pub fn is_pass(&self) -> bool {
        matches!(self, VerificationResult::Pass | VerificationResult::PassWithWarnings)
    }
    
    /// Returns true if the job failed critically
    pub fn is_hard_fail(&self) -> bool {
        matches!(self, VerificationResult::FailHard)
    }
}
```

### 2. Update parse_verification function
Change the signature to:
```rust
pub fn parse_verification(response: &str) -> (VerificationResult, Option<String>)
```

Parse logic:
1. Check first word/phrase for exact matches:
   - "PASS" or "PASSED" → `Pass`
   - "PASS_WITH_WARNINGS" or "PASS WITH WARNINGS" → `PassWithWarnings`
   - "FAIL_SOFT" or "FAIL SOFT" → `FailSoft`
   - "FAIL_HARD" or "FAIL HARD" → `FailHard`
   - "FAIL" or "FAILED" (without suffix) → `FailHard` (default to strictest)
2. For unclear responses, return `FailHard` with "Unclear verification response"

### 3. Add helper function to convert to JobStatus
```rust
impl VerificationResult {
    pub fn to_job_status(&self) -> JobStatus {
        match self {
            VerificationResult::Pass | VerificationResult::PassWithWarnings => JobStatus::Pass,
            VerificationResult::FailSoft | VerificationResult::FailHard => JobStatus::Fail,
        }
    }
}
```

### 4. Update tests
- Test all four tier patterns
- Test case insensitivity
- Test underscore and space variants
- Test that simple FAIL defaults to FailHard
- Keep existing tests working by accepting the new return type

### 5. Export the enum
Add `VerificationResult` to the module's public exports.

## Implementation Notes
- The existing `JobStatus::Pass` and `JobStatus::Fail` remain unchanged
- `VerificationResult` is for the parsing stage, `JobStatus` for storage
- Runners will use `is_pass()` to determine success
- The tier information can be logged or stored for reporting

## Expected Behavior
```rust
let (result, msg) = parse_verification("PASS_WITH_WARNINGS: Minor style issues");
assert_eq!(result, VerificationResult::PassWithWarnings);
assert!(result.is_pass());
assert!(!result.is_hard_fail());
assert_eq!(result.to_job_status(), JobStatus::Pass);

let (result, msg) = parse_verification("FAIL_HARD: Syntax errors on line 42");
assert_eq!(result, VerificationResult::FailHard);
assert!(!result.is_pass());
assert!(result.is_hard_fail());
assert_eq!(result.to_job_status(), JobStatus::Fail);
```

## Future Considerations
- Could add `VerificationResult` to `JobStatusEntry` for historical tracking
- Could add `--fail-on-warnings` flag to treat warnings as failures
- Could add different retry behavior based on severity
