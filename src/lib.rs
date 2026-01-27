#![allow(dead_code)]
//! WorkSplit - Ollama-powered job orchestrator for code generation
//!
//! WorkSplit is a CLI tool that processes job files through a local Ollama LLM instance.
//! It handles a two-phase workflow: creation (generate code) and verification (validate output).
//!
//! # Architecture
//!
//! - **commands**: CLI command implementations (init, run, status, validate)
//! - **core**: Core functionality (runner, jobs, status, ollama client, parser)
//! - **models**: Data structures (config, job, status)
//! - **error**: Error types

pub mod commands;
pub mod core;
pub mod error;
pub mod models;

pub use error::{Result, WorkSplitError};
