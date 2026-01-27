//! Prompt assembly functions for LLM interactions.

use std::path::PathBuf;

/// Assemble a creation prompt
pub fn assemble_creation_prompt(
    system_prompt: &str,
    context_files: &[(PathBuf, String)],
    instructions: &str,
    output_path: &str,
) -> String {
    let mut prompt = String::new();

    // System prompt
    prompt.push_str("[SYSTEM]\n");
    prompt.push_str(system_prompt);
    prompt.push_str("\n\n");

    // Context files
    if !context_files.is_empty() {
        prompt.push_str("[CONTEXT]\n");
        for (path, content) in context_files {
            prompt.push_str(&format!("### File: {}\n", path.display()));
            prompt.push_str("```\n");
            prompt.push_str(content);
            if !content.ends_with('\n') {
                prompt.push('\n');
            }
            prompt.push_str("```\n\n");
        }
    }

    // Instructions
    prompt.push_str("[INSTRUCTIONS]\n");
    prompt.push_str(instructions);
    prompt.push_str("\n\n");
    prompt.push_str(&format!("Output to: {}\n", output_path));

    prompt
}

/// Assemble a creation prompt for sequential multi-file mode
/// 
/// In sequential mode, each file is generated with its own LLM call.
/// Previously generated files in this job are added as context for subsequent files.
pub fn assemble_sequential_creation_prompt(
    system_prompt: &str,
    context_files: &[(PathBuf, String)],
    previously_generated: &[(PathBuf, String)],
    instructions: &str,
    current_output_path: &str,
    remaining_files: &[PathBuf],
) -> String {
    let mut prompt = String::new();

    // System prompt
    prompt.push_str("[SYSTEM]\n");
    prompt.push_str(system_prompt);
    prompt.push_str("\n\n");

    // Original context files
    if !context_files.is_empty() {
        prompt.push_str("[CONTEXT]\n");
        for (path, content) in context_files {
            prompt.push_str(&format!("### File: {}\n", path.display()));
            prompt.push_str("```\n");
            prompt.push_str(content);
            if !content.ends_with('\n') {
                prompt.push('\n');
            }
            prompt.push_str("```\n\n");
        }
    }

    // Previously generated files
    if !previously_generated.is_empty() {
        prompt.push_str("[PREVIOUSLY GENERATED IN THIS JOB]\n");
        prompt.push_str("These files were already generated as part of this same task. ");
        prompt.push_str("Use them as reference for consistency.\n\n");
        for (path, content) in previously_generated {
            prompt.push_str(&format!("### File: {}\n", path.display()));
            prompt.push_str("```\n");
            prompt.push_str(content);
            if !content.ends_with('\n') {
                prompt.push('\n');
            }
            prompt.push_str("```\n\n");
        }
    }

    // Instructions
    prompt.push_str("[INSTRUCTIONS]\n");
    prompt.push_str(instructions);
    prompt.push_str("\n\n");

    // Current file to generate
    prompt.push_str("[CURRENT OUTPUT FILE]\n");
    prompt.push_str(&format!("Generate: {}\n\n", current_output_path));

    // List remaining files for context
    if !remaining_files.is_empty() {
        prompt.push_str("[REMAINING FILES]\n");
        prompt.push_str("These files will be generated after this one:\n");
        for path in remaining_files {
            prompt.push_str(&format!("  - {}\n", path.display()));
        }
        prompt.push_str("\nConsider their requirements when designing interfaces.\n");
    }

    prompt
}

/// Assemble a verification prompt (single file version for backward compatibility)
pub fn assemble_verification_prompt(
    system_prompt: &str,
    context_files: &[(PathBuf, String)],
    generated_output: &str,
    output_path: &str,
    instructions: &str,
) -> String {
    assemble_verification_prompt_multi(
        system_prompt,
        context_files,
        &[(PathBuf::from(output_path), generated_output.to_string())],
        instructions,
    )
}

/// Assemble a verification prompt for multiple generated files
pub fn assemble_verification_prompt_multi(
    system_prompt: &str,
    context_files: &[(PathBuf, String)],
    generated_files: &[(PathBuf, String)],
    instructions: &str,
) -> String {
    let mut prompt = String::new();

    // System prompt
    prompt.push_str("[SYSTEM]\n");
    prompt.push_str(system_prompt);
    prompt.push_str("\n\n");

    // Context files
    if !context_files.is_empty() {
        prompt.push_str("[CONTEXT]\n");
        for (path, content) in context_files {
            prompt.push_str(&format!("### File: {}\n", path.display()));
            prompt.push_str("```\n");
            prompt.push_str(content);
            if !content.ends_with('\n') {
                prompt.push('\n');
            }
            prompt.push_str("```\n\n");
        }
    }

    // Generated output(s)
    prompt.push_str("[GENERATED OUTPUT]\n");
    for (path, content) in generated_files {
        prompt.push_str(&format!("### File: {}\n", path.display()));
        prompt.push_str("```\n");
        prompt.push_str(content);
        if !content.ends_with('\n') {
            prompt.push('\n');
        }
        prompt.push_str("```\n\n");
    }

    // Original instructions
    prompt.push_str("[ORIGINAL INSTRUCTIONS]\n");
    prompt.push_str(instructions);
    prompt.push('\n');

    prompt
}

/// Assemble a test generation prompt for TDD workflow
pub fn assemble_test_prompt(
    system_prompt: &str,
    context_files: &[(PathBuf, String)],
    instructions: &str,
    test_path: &str,
) -> String {
    let mut prompt = String::new();

    // System prompt
    prompt.push_str("[SYSTEM]\n");
    prompt.push_str(system_prompt);
    prompt.push_str("\n\n");

    // Context files
    if !context_files.is_empty() {
        prompt.push_str("[CONTEXT]\n");
        for (path, content) in context_files {
            prompt.push_str(&format!("### File: {}\n", path.display()));
            prompt.push_str("```\n");
            prompt.push_str(content);
            if !content.ends_with('\n') {
                prompt.push('\n');
            }
            prompt.push_str("```\n\n");
        }
    }

    // Requirements
    prompt.push_str("[REQUIREMENTS]\n");
    prompt.push_str(instructions);
    prompt.push_str("\n\n");

    // Test output info with TDD context
    prompt.push_str("[TEST OUTPUT]\n");
    prompt.push_str(&format!("Generate tests for: {}\n\n", test_path));
    prompt.push_str("The implementation does not exist yet. Generate tests that will:\n");
    prompt.push_str("1. Verify the requirements are met when implementation exists\n");
    prompt.push_str("2. Cover edge cases and error conditions\n");
    prompt.push_str("3. Be immediately runnable once implementation is created\n");

    prompt
}

/// Assemble a retry prompt with feedback from failed verification (single file version)
pub fn assemble_retry_prompt(
    system_prompt: &str,
    context_files: &[(PathBuf, String)],
    instructions: &str,
    output_path: &str,
    previous_output: &str,
    verification_error: &str,
) -> String {
    assemble_retry_prompt_multi(
        system_prompt,
        context_files,
        instructions,
        &[(PathBuf::from(output_path), previous_output.to_string())],
        verification_error,
    )
}

/// Assemble a retry prompt with feedback from failed verification (multi-file version)
pub fn assemble_retry_prompt_multi(
    system_prompt: &str,
    context_files: &[(PathBuf, String)],
    instructions: &str,
    previous_outputs: &[(PathBuf, String)],
    verification_error: &str,
) -> String {
    let mut prompt = String::new();

    // System prompt
    prompt.push_str("[SYSTEM]\n");
    prompt.push_str(system_prompt);
    prompt.push_str("\n\n");

    // Context files
    if !context_files.is_empty() {
        prompt.push_str("[CONTEXT]\n");
        for (path, content) in context_files {
            prompt.push_str(&format!("### File: {}\n", path.display()));
            prompt.push_str("```\n");
            prompt.push_str(content);
            if !content.ends_with('\n') {
                prompt.push('\n');
            }
            prompt.push_str("```\n\n");
        }
    }

    // Previous attempt(s)
    prompt.push_str("[PREVIOUS ATTEMPT]\n");
    for (path, content) in previous_outputs {
        prompt.push_str(&format!("### File: {}\n", path.display()));
        prompt.push_str("```\n");
        prompt.push_str(content);
        if !content.ends_with('\n') {
            prompt.push('\n');
        }
        prompt.push_str("```\n\n");
    }

    // Verification feedback
    prompt.push_str("[VERIFICATION FEEDBACK]\n");
    prompt.push_str("The previous attempt failed verification with the following feedback:\n");
    prompt.push_str(verification_error);
    prompt.push_str("\n\n");

    // Instructions
    prompt.push_str("[INSTRUCTIONS]\n");
    prompt.push_str(instructions);
    prompt.push_str("\n\n");
    
    // List output files
    if previous_outputs.len() == 1 {
        prompt.push_str(&format!("Output to: {}\n\n", previous_outputs[0].0.display()));
    } else {
        prompt.push_str("Output files:\n");
        for (path, _) in previous_outputs {
            prompt.push_str(&format!("  - {}\n", path.display()));
        }
        prompt.push('\n');
    }
    prompt.push_str("Please fix the issues mentioned in the verification feedback and generate improved code.\n");

    prompt
}

/// Assemble a split prompt for breaking a large file into modules
pub fn assemble_split_prompt(
    system_prompt: &str,
    target_file: (&PathBuf, &str),
    context_files: &[(PathBuf, String)],
    instructions: &str,
    output_files: &[PathBuf],
) -> String {
    let mut prompt = String::new();

    // System prompt
    prompt.push_str("[SYSTEM]\n");
    prompt.push_str(system_prompt);
    prompt.push_str("\n\n");

    // Target file to split
    prompt.push_str("[TARGET FILE TO SPLIT]\n");
    prompt.push_str(&format!("### File: {} (to be split into modules)\n", target_file.0.display()));
    prompt.push_str("```\n");
    prompt.push_str(target_file.1);
    if !target_file.1.ends_with('\n') {
        prompt.push('\n');
    }
    prompt.push_str("```\n\n");

    // Additional context files
    if !context_files.is_empty() {
        prompt.push_str("[ADDITIONAL CONTEXT]\n");
        for (path, content) in context_files {
            prompt.push_str(&format!("### File: {}\n", path.display()));
            prompt.push_str("```\n");
            prompt.push_str(content);
            if !content.ends_with('\n') {
                prompt.push('\n');
            }
            prompt.push_str("```\n\n");
        }
    }

    // Instructions
    prompt.push_str("[INSTRUCTIONS]\n");
    prompt.push_str(instructions);
    prompt.push_str("\n\n");

    // Output files to generate
    prompt.push_str("[OUTPUT FILES]\n");
    prompt.push_str("Generate the following files:\n");
    for path in output_files {
        prompt.push_str(&format!("  - {}\n", path.display()));
    }
    prompt.push_str("\nUse the ~~~worksplit:path/to/file.rs delimiter for each output file.\n");
    prompt.push_str("Ensure ALL functionality from the target file is preserved across the output files.\n");

    prompt
}

/// Assemble a sequential split prompt (one file at a time)
pub fn assemble_sequential_split_prompt(
    system_prompt: &str,
    target_file: (&PathBuf, &str),
    context_files: &[(PathBuf, String)],
    previously_generated: &[(PathBuf, String)],
    instructions: &str,
    current_output_path: &str,
    remaining_files: &[PathBuf],
) -> String {
    let mut prompt = String::new();

    // System prompt
    prompt.push_str("[SYSTEM]\n");
    prompt.push_str(system_prompt);
    prompt.push_str("\n\n");

    // Target file to split
    prompt.push_str("[TARGET FILE TO SPLIT]\n");
    prompt.push_str(&format!("### File: {} (original file being split)\n", target_file.0.display()));
    prompt.push_str("```\n");
    prompt.push_str(target_file.1);
    if !target_file.1.ends_with('\n') {
        prompt.push('\n');
    }
    prompt.push_str("```\n\n");

    // Additional context files
    if !context_files.is_empty() {
        prompt.push_str("[ADDITIONAL CONTEXT]\n");
        for (path, content) in context_files {
            prompt.push_str(&format!("### File: {}\n", path.display()));
            prompt.push_str("```\n");
            prompt.push_str(content);
            if !content.ends_with('\n') {
                prompt.push('\n');
            }
            prompt.push_str("```\n\n");
        }
    }

    // Previously generated files
    if !previously_generated.is_empty() {
        prompt.push_str("[ALREADY GENERATED IN THIS SPLIT]\n");
        prompt.push_str("These files were already generated from the target file. ");
        prompt.push_str("Ensure consistency and avoid duplicating code that's already in these files.\n\n");
        for (path, content) in previously_generated {
            prompt.push_str(&format!("### File: {}\n", path.display()));
            prompt.push_str("```\n");
            prompt.push_str(content);
            if !content.ends_with('\n') {
                prompt.push('\n');
            }
            prompt.push_str("```\n\n");
        }
    }

    // Instructions
    prompt.push_str("[INSTRUCTIONS]\n");
    prompt.push_str(instructions);
    prompt.push_str("\n\n");

    // Current file to generate
    prompt.push_str("[CURRENT OUTPUT FILE]\n");
    prompt.push_str(&format!("Generate ONLY this file: {}\n", current_output_path));
    prompt.push_str("Extract the appropriate code from the target file into this module.\n\n");

    // List remaining files for context
    if !remaining_files.is_empty() {
        prompt.push_str("[REMAINING FILES]\n");
        prompt.push_str("These files will be generated after this one:\n");
        for path in remaining_files {
            prompt.push_str(&format!("  - {}\n", path.display()));
        }
        prompt.push_str("\nDo NOT include code that belongs in these files. Focus only on the current file.\n");
    }

    prompt.push_str("\nOutput the file using the ~~~worksplit:path/to/file.rs delimiter.\n");

    prompt
}
