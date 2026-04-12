use super::{ExecOutput, Runtime};
/// Comandos builtin del shell: cd, ls, echo, cat, pwd, mkdir, rm...
use crate::error::{Result, RunboxError};
use crate::process::ProcessManager;
use crate::shell::Command;
use crate::vfs::Vfs;

pub struct ShellBuiltins;

impl Runtime for ShellBuiltins {
    fn name(&self) -> &'static str {
        "shell"
    }

    fn exec(&self, cmd: &Command, vfs: &mut Vfs, _pm: &mut ProcessManager) -> Result<ExecOutput> {
        match cmd.program.as_str() {
            "echo" => Ok(ok(format!("{}\n", cmd.args.join(" ")))),

            "pwd" => Ok(ok("/\n")),

            "ls" => {
                let path = cmd.args.first().map(String::as_str).unwrap_or("/");
                let mut entries = vfs.list(path)?;
                entries.sort();
                Ok(ok(entries.join("\n") + "\n"))
            }

            "cat" => {
                let path = cmd
                    .args
                    .first()
                    .ok_or_else(|| RunboxError::Shell("cat: missing file".into()))?;
                let bytes = vfs.read(path)?.to_vec();
                Ok(ExecOutput {
                    stdout: bytes,
                    stderr: vec![],
                    exit_code: 0,
                })
            }

            "mkdir" => {
                let path = cmd
                    .args
                    .first()
                    .ok_or_else(|| RunboxError::Shell("mkdir: missing path".into()))?;
                // Crea un placeholder para marcar el dir
                vfs.write(&format!("{path}/.runbox_dir"), vec![])?;
                Ok(ok(""))
            }

            "rm" => {
                let path = cmd
                    .args
                    .first()
                    .ok_or_else(|| RunboxError::Shell("rm: missing path".into()))?;
                vfs.remove(path)?;
                Ok(ok(""))
            }

            "touch" => {
                let path = cmd
                    .args
                    .first()
                    .ok_or_else(|| RunboxError::Shell("touch: missing path".into()))?;
                if !vfs.exists(path) {
                    vfs.write(path, vec![])?;
                }
                Ok(ok(""))
            }

            other => Err(RunboxError::Shell(format!("{other}: command not found"))),
        }
    }
}

fn ok(s: impl AsRef<[u8]>) -> ExecOutput {
    ExecOutput {
        stdout: s.as_ref().to_vec(),
        stderr: vec![],
        exit_code: 0,
    }
}
