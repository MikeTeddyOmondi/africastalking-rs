//! # AfricasTalking Rust SDK
//!
//! This crate provides a comprehensive Rust SDK for the AfricasTalking API.
//!
//! ## Quick Start
//!
//! ```rust
//! use africastalking::{AfricasTalkingClient, Config, Environment};
//!
//! let config = Config::new("your-api-key", "your-username")
//!     .environment(Environment::Sandbox);
//!
//! let client = AfricasTalkingClient::new(config);
//! ```

pub mod client;
pub mod config;
pub mod error;
pub mod modules;
pub mod types;

// Re-export main types for easier usage
pub use client::AfricasTalkingClient;
pub use config::{Config, Environment};
pub use error::{AfricasTalkingError, Result};
pub use types::*;

// Re-export modules for direct access
pub use modules::*;
