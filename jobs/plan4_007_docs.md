---
mode: edit
context_files:
  - README.md
  - jobs/_managerinstruction.md
target_files:
  - README.md
  - jobs/_managerinstruction.md
output_dir: .
output_file: README.md
---

# Update docs for Plan4 operator tooling

## Requirements
- Update `README.md` to document the new CLI capabilities:
  - `worksplit cancel <job_id|all>`
  - `worksplit retry <job_id>`
  - `worksplit status --watch`
  - `worksplit run --job-timeout <seconds>`
- Update `jobs/_managerinstruction.md` to mention:
  - Cancel and retry as recommended recovery tools.
  - Status watch for long runs.
  - Per-job timeout for stuck runs.
- Keep the docs aligned with actual CLI flags (`--watch`, `--job-timeout`).
- Fix any inconsistent examples that mention a `--summary` flag for watch; use `status --watch` instead.

## Suggested Placement
- `README.md`:
  - Add to "Features" list.
  - Add to "CLI Commands" section as new subsections or examples.
- `jobs/_managerinstruction.md`:
  - Add to "Useful CLI Tools" and/or "Token-Efficient Workflow".

## Constraints
- Keep edits concise and consistent with existing formatting.
