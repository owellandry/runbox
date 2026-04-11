pub mod bun;
pub mod python;
pub mod git;
pub mod npm;
pub mod shell_builtins;
pub mod js_engine;

use crate::error::Result;
use crate::vfs::Vfs;
use crate::process::ProcessManager;
use crate::shell::Command;

#[derive(Debug, Clone)]
pub struct ExecOutput {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub exit_code: i32,
}

impl ExecOutput {
    pub fn stdout_str(&self) -> &str {
        std::str::from_utf8(&self.stdout).unwrap_or("")
    }
}

pub trait Runtime {
    fn name(&self) -> &'static str;
    fn exec(&self, cmd: &Command, vfs: &mut Vfs, pm: &mut ProcessManager) -> Result<ExecOutput>;
}
