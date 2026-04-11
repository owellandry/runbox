pub mod vfs;
pub mod shell;
pub mod process;
pub mod runtime;
pub mod console;
pub mod sandbox;
pub mod hotreload;
pub mod inspector;
pub mod network;
pub mod terminal;
pub mod error;
pub mod ai;
pub mod mcp;

#[cfg(target_arch = "wasm32")]
pub mod wasm;
