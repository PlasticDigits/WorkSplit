//! Rust-specific templates for WorkSplit
//!
//! Templates are loaded from external files in the `templates/rust/` directory.

use super::Templates;

/// Get Rust-specific templates
pub fn templates() -> Templates {
    Templates {
        create_prompt: include_str!("../../templates/rust/systemprompt_create.md"),
        verify_prompt: include_str!("../../templates/rust/systemprompt_verify.md"),
        edit_prompt: include_str!("../../templates/rust/systemprompt_edit.md"),
        verify_edit_prompt: include_str!("../../templates/rust/systemprompt_verify_edit.md"),
        split_prompt: include_str!("../../templates/rust/systemprompt_split.md"),
        test_prompt: include_str!("../../templates/rust/systemprompt_test.md"),
        fix_prompt: include_str!("../../templates/rust/systemprompt_fix.md"),
        manager_instruction: include_str!("../../templates/rust/manager_instruction.md"),
        config: include_str!("../../templates/rust/config.toml"),
        example_job: include_str!("../../templates/rust/example_job.md"),
        tdd_example_job: include_str!("../../templates/rust/example_tdd_job.md"),
    }
}
