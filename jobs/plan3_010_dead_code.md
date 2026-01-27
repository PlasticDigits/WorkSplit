---
mode: edit
context_files:
  - src/core/runner/mod.rs
  - jobs/_managerinstruction.md
target_files:
  - src/core/runner/mod.rs
  - jobs/_managerinstruction.md
output_dir: src/
output_file: core/runner/mod.rs
---

# Plan3 Task 6: Dead Code Prevention

Update documentation and add basic dead code detection placeholder/logic.

## Requirements

1. Update the integration reminder log message to be more explicit.

## Edit Instructions

FILE: src/core/runner/mod.rs
FIND:
        } else {
            self.status_manager.update_status(job_id, final_status)?;
        }

        info!("Job '{}' completed with status: {:?}", job_id, final_status);
REPLACE:
        } else {
            self.status_manager.update_status(job_id, final_status)?;
        }

        if final_status == JobStatus::Pass {
            info!("Generation complete. REMINDER: Ensure new code is wired into callers.");
        }

        info!("Job '{}' completed with status: {:?}", job_id, final_status);
END
