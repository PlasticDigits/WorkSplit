//! System prompts for different task types
//!
//! These prompts set the model's behavior at the system level via the chat API.
//! Task-specific details continue to be provided in the user message.

/// System prompt for code generation tasks (create, sequential, split)
pub const SYSTEM_PROMPT_CREATE: &str = r#"You are a coding agent. Read the prompt once, then output code immediately.
Do NOT re-read or re-analyze the prompt. Do NOT say "wait" or reconsider.
Output ONLY clean code using the specified delimiters.
For multi-file output, use ~~~worksplit:path/to/file delimiters.
For edit mode, use FILE: / FIND: / REPLACE: / END format.
Be concise. No explanations unless requested."#;

/// System prompt for verification tasks
///
/// Key differences from generation:
/// - Focus on analysis, not output
/// - Must provide judgment (PASS/FAIL)
/// - Should explain reasoning for failures
pub const SYSTEM_PROMPT_VERIFY: &str = r#"You are a code verification agent. Analyze the provided code carefully against the requirements.

Your task is to determine if the generated code correctly implements what was asked for.

Response format:
- If the code is correct: Start your response with "PASS" (optionally followed by brief notes)
- If the code has issues: Start with "FAIL:" followed by a clear explanation of what's wrong

Be thorough but fair. Minor style issues should not cause failure if functionality is correct.
Focus on: correctness, completeness, proper error handling, and adherence to instructions."#;

/// System prompt for edit mode generation
///
/// Similar to create but emphasizes surgical edits
pub const SYSTEM_PROMPT_EDIT: &str = r#"You are a code editing agent. Make surgical changes to existing code.

Output edits in this EXACT format:
FILE: path/to/file.rs
FIND:
<exact text to find>
REPLACE:
<replacement text>
END

Critical rules:
- FIND text must match EXACTLY (including whitespace and indentation)
- Include enough context to make FIND unique
- Output ONLY the edit blocks, no explanations
- Multiple edits per file are allowed
- Multiple files use separate FILE: lines"#;

/// System prompt for test generation (TDD mode)
pub const SYSTEM_PROMPT_TEST: &str = r#"You are a test generation agent. Create comprehensive unit tests.

Output ONLY the test code wrapped in code fences.
Write tests that:
- Cover the main functionality
- Include edge cases
- Test error conditions
- Follow the project's testing patterns

Be concise. No explanations unless requested."#;

/// System prompt for retry after failed verification
///
/// Emphasizes learning from the error
pub const SYSTEM_PROMPT_RETRY: &str = r#"You are a coding agent fixing a failed attempt. 
The previous code had issues identified by the verifier.
Read the error feedback carefully and fix the specific problems mentioned.
Output ONLY the corrected code using the specified delimiters.
Be concise. Focus on fixing the identified issues."#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompts_not_empty() {
        assert!(!SYSTEM_PROMPT_CREATE.is_empty());
        assert!(!SYSTEM_PROMPT_VERIFY.is_empty());
        assert!(!SYSTEM_PROMPT_EDIT.is_empty());
        assert!(!SYSTEM_PROMPT_TEST.is_empty());
        assert!(!SYSTEM_PROMPT_RETRY.is_empty());
    }

    #[test]
    fn test_verify_prompt_mentions_pass_fail() {
        assert!(SYSTEM_PROMPT_VERIFY.contains("PASS"));
        assert!(SYSTEM_PROMPT_VERIFY.contains("FAIL"));
    }

    #[test]
    fn test_edit_prompt_mentions_format() {
        assert!(SYSTEM_PROMPT_EDIT.contains("FILE:"));
        assert!(SYSTEM_PROMPT_EDIT.contains("FIND:"));
        assert!(SYSTEM_PROMPT_EDIT.contains("REPLACE:"));
        assert!(SYSTEM_PROMPT_EDIT.contains("END"));
    }
}
