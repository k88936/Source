//! Helpers shared by the examples.
//!
//! Cargo builds every file in `examples/` as its own binary, so one example
//! cannot import another. Including this module with
//! `#[path = "example_utils/mod.rs"] mod example_utils;` compiles it into each
//! example that needs it.

pub mod env;

pub mod login;

pub mod mock;
