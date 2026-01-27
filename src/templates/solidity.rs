//! Solidity-specific templates for WorkSplit
//!
//! Templates are loaded from external files in the `templates/solidity/` directory.

use super::Templates;

/// Get Solidity-specific templates (Foundry)
pub fn templates() -> Templates {
    Templates {
        create_prompt: include_str!("../../templates/solidity/systemprompt_create.md"),
        verify_prompt: include_str!("../../templates/solidity/systemprompt_verify.md"),
        edit_prompt: include_str!("../../templates/solidity/systemprompt_edit.md"),
        verify_edit_prompt: include_str!("../../templates/solidity/systemprompt_verify_edit.md"),
        split_prompt: include_str!("../../templates/solidity/systemprompt_split.md"),
        test_prompt: include_str!("../../templates/solidity/systemprompt_test.md"),
        fix_prompt: include_str!("../../templates/solidity/systemprompt_fix.md"),
        manager_instruction: include_str!("../../templates/solidity/manager_instruction.md"),
        config: include_str!("../../templates/solidity/config.toml"),
        example_job: include_str!("../../templates/solidity/example_job.md"),
        tdd_example_job: include_str!("../../templates/solidity/example_tdd_job.md"),
    }
}
