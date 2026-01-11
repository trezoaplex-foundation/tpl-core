#![deny(missing_docs)]
//! # TPL Core
//!
//! Trezoaplex Core (“Core”) sheds the cotplexity and technical debt of previous
//! standards and provides a clean and sitple core spec for digital assets.
//! This makes the bare minimum use case easy and understandable for users just
//! starting out with Digital Assets.
//!
//!However, it's designed with a flexible plugin system that allows for users
//! to extend the program itself without relying on Trezoaplex to add support to
//! a rigid standard like Token Metadata. The plugin system is so powerful that
//! it could even allow users to contribute third party plugins after the core
//! program is made immutable.

/// Standard Trezoa entrypoint.
pub mod entrypoint;
/// Error types for TPL Core.
pub mod error;
/// Main enum for managing instructions on TPL Core.
pub mod instruction;
/// Module for managing plugins.
pub mod plugins;
/// Module for processing instructions and routing them
/// to the associated processor.
pub mod processor;
/// State and Type definitions for TPL Core.
pub mod state;
/// Program-wide utility functions.
pub mod utils;

pub use trezoa_program;

trezoa_program::declare_id!("CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d");
