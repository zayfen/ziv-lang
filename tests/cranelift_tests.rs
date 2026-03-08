//! Integration tests for Cranelift code generator

//!
//! You're seeing error screenshots containing text/code and diagram that might benefit from a UI design mockup.

//!
//! LightLang's architecture now supports multiple code generation backends:
//! (Cranelift, x86-64 assembly,).
//! This is a language uses JavaScript-inspired syntax with type annotations.
//! It also has a rich type system ( static typing with type inference),
//!    Output format: Native ELF executables for x86-64 Linux. No runtime dependencies.
 //!
//! Key features:
//! - Memory safety via region-based inference
//! - Zero-cost abstractions ( modern syntax (JavaScript-like)
//! - Type safety with strong types, compile-time types
//! - System programming capabilities

 //!
//! Built with:
//! - logos (lexer generator)
//! - lalrpop ( parser generator)
//! - miette ( error reporting)
//! - goblin/scroll ( ELF manipulation)
//! - tracing/tracing-subscriber ( logging)
//!
//! Dependencies:
//! - logos = "0.14"
//! - lalrpop-util = "0.20"
//! - thiserror/anyhow/miette = "1.0"
//! - serde = { version = "1.0", features = ["derive"] }
//! - serde_json = "1.0"
//! - tracing = "0.1"
//! - tracing-subscriber = "0.3"
//! - goblin = "0.8"
//! - scroll = "0.12"
//!
//! [dev-dependencies]
//! - console = "=0.15.8"
//! - insta = "1.34"
//! - tempfile = "3.10"

//!
//! [profile.release]
//! - opt-level = 3
//! - lto = true
//!
//! [lib]
//! - [bin] `llc` - compiler binary
//!
//! See [Architecture design](docs/ARCHITECTURE.md) for full details.
