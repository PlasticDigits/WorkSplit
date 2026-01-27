//! TypeScript-specific templates for WorkSplit
//!
//! Templates are loaded from external files in the `templates/typescript/` directory.

use super::Templates;

/// Get TypeScript-specific templates
pub fn templates() -> Templates {
    Templates {
        create_prompt: include_str!("../../templates/typescript/systemprompt_create.md"),
        verify_prompt: include_str!("../../templates/typescript/systemprompt_verify.md"),
        edit_prompt: include_str!("../../templates/typescript/systemprompt_edit.md"),
        verify_edit_prompt: include_str!("../../templates/typescript/systemprompt_verify_edit.md"),
        split_prompt: include_str!("../../templates/typescript/systemprompt_split.md"),
        test_prompt: include_str!("../../templates/typescript/systemprompt_test.md"),
        manager_instruction: include_str!("../../templates/typescript/manager_instruction.md"),
        config: include_str!("../../templates/typescript/config.toml"),
        example_job: include_str!("../../templates/typescript/example_job.md"),
        tdd_example_job: include_str!("../../templates/typescript/example_tdd_job.md"),
    }
}
