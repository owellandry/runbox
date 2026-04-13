pub mod ai;
pub mod console;
pub mod error;
pub mod hotreload;
pub mod inspector;
pub mod mcp;
pub mod network;
pub mod preview;
pub mod process;
pub mod runtime;
pub mod sandbox;
pub mod shell;
pub mod terminal;
pub mod vfs;

#[cfg(target_arch = "wasm32")]
pub mod wasm;
