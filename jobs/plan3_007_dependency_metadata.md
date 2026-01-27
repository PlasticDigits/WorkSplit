---
mode: edit
context_files:
  - src/models/job.rs
target_files:
  - src/models/job.rs
output_dir: src/
output_file: models/job.rs
---

# Plan3 Task 4: Dependency Metadata

Add `depends_on` field to job metadata.

## Requirements

1. Update `JobMetadata` struct in `src/models/job.rs` to include `depends_on: Option<Vec<String>>`.

## Edit Instructions

FILE: src/models/job.rs
FIND:
    /// List of job IDs this job depends on
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub depends_on: Option<Vec<String>>,
REPLACE:
    /// Optional list of job IDs this job depends on
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub depends_on: Option<Vec<String>>,
END
