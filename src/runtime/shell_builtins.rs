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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shell::Command;
    use crate::vfs::Vfs;

    #[test]
    fn test_echo() {
        let mut vfs = Vfs::new();
        let mut pm = ProcessManager::new();
        let runtime = ShellBuiltins;

        let cmd = Command {
            program: "echo".to_string(),
            args: vec!["hello".to_string(), "world".to_string()],
            env: vec![],
        };
        let out = runtime.exec(&cmd, &mut vfs, &mut pm).unwrap();
        assert_eq!(out.stdout, b"hello world\n");
    }

    #[test]
    fn test_pwd() {
        let mut vfs = Vfs::new();
        let mut pm = ProcessManager::new();
        let runtime = ShellBuiltins;

        let cmd = Command {
            program: "pwd".to_string(),
            args: vec![],
            env: vec![],
        };
        let out = runtime.exec(&cmd, &mut vfs, &mut pm).unwrap();
        assert_eq!(out.stdout, b"/\n");
    }

    #[test]
    fn test_touch_and_ls() {
        let mut vfs = Vfs::new();
        let mut pm = ProcessManager::new();
        let runtime = ShellBuiltins;

        let cmd_touch = Command {
            program: "touch".to_string(),
            args: vec!["/test.txt".to_string()],
            env: vec![],
        };
        runtime.exec(&cmd_touch, &mut vfs, &mut pm).unwrap();

        let cmd_ls = Command {
            program: "ls".to_string(),
            args: vec!["/".to_string()],
            env: vec![],
        };
        let out_ls = runtime.exec(&cmd_ls, &mut vfs, &mut pm).unwrap();
        assert_eq!(out_ls.stdout, b"test.txt\n");
    }

    #[test]
    fn test_cat() {
        let mut vfs = Vfs::new();
        let mut pm = ProcessManager::new();
        let runtime = ShellBuiltins;

        vfs.write("/hello.txt", b"world".to_vec()).unwrap();

        let cmd = Command {
            program: "cat".to_string(),
            args: vec!["/hello.txt".to_string()],
            env: vec![],
        };
        let out = runtime.exec(&cmd, &mut vfs, &mut pm).unwrap();
        assert_eq!(out.stdout, b"world");
    }

    #[test]
    fn test_mkdir() {
        let mut vfs = Vfs::new();
        let mut pm = ProcessManager::new();
        let runtime = ShellBuiltins;

        let cmd = Command {
            program: "mkdir".to_string(),
            args: vec!["/mydir".to_string()],
            env: vec![],
        };
        runtime.exec(&cmd, &mut vfs, &mut pm).unwrap();
        assert!(vfs.exists("/mydir/.runbox_dir"));
    }

    #[test]
    fn test_rm() {
        let mut vfs = Vfs::new();
        let mut pm = ProcessManager::new();
        let runtime = ShellBuiltins;

        vfs.write("/delete_me.txt", vec![]).unwrap();

        let cmd = Command {
            program: "rm".to_string(),
            args: vec!["/delete_me.txt".to_string()],
            env: vec![],
        };
        runtime.exec(&cmd, &mut vfs, &mut pm).unwrap();
        assert!(!vfs.exists("/delete_me.txt"));
    }
}
