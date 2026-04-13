use super::{ExecOutput, Runtime};
/// Runtime de Python.
/// Nativo: intenta ejecutar `python3` / `python` del sistema.
/// WASM: delega en Pyodide vía callback JS.
use crate::error::{Result, RunboxError};
use crate::process::ProcessManager;
use crate::shell::Command;
use crate::vfs::Vfs;

pub struct PythonRuntime;

impl Runtime for PythonRuntime {
    fn name(&self) -> &'static str {
        "python"
    }

    fn exec(&self, cmd: &Command, vfs: &mut Vfs, pm: &mut ProcessManager) -> Result<ExecOutput> {
        match cmd.program.as_str() {
            "pip" | "pip3" => pip_exec(cmd, vfs, pm),
            _ => python_exec(cmd, vfs, pm),
        }
    }
}

// ── python / python3 ──────────────────────────────────────────────────────────

fn python_exec(cmd: &Command, vfs: &mut Vfs, pm: &mut ProcessManager) -> Result<ExecOutput> {
    // Modo -c "código inline"
    if cmd.args.first().map(String::as_str) == Some("-c") {
        return python_inline(cmd, vfs, pm);
    }

    let file = match cmd.args.first() {
        Some(f) => f.clone(),
        None => {
            return Ok(ok_out(
                "Python 3.x (RunBox)\nType -c \"code\" to run inline.\n",
            ));
        }
    };

    let path = if file.starts_with('/') {
        file.clone()
    } else {
        format!("/{file}")
    };
    if !vfs.exists(&path) {
        return Err(RunboxError::NotFound(path));
    }

    spawn_python(cmd, vfs, pm)
}

fn python_inline(cmd: &Command, vfs: &mut Vfs, pm: &mut ProcessManager) -> Result<ExecOutput> {
    spawn_python(cmd, vfs, pm)
}

fn spawn_python(cmd: &Command, vfs: &mut Vfs, pm: &mut ProcessManager) -> Result<ExecOutput> {
    #[cfg(target_arch = "wasm32")]
    let _ = vfs;

    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Ok(output) = try_spawn_system_python(cmd, vfs) {
            let pid = pm.spawn(&cmd.program, cmd.args.clone());
            pm.exit(pid, output.exit_code)?;
            return Ok(output);
        }
    }

    // Python no disponible en el sistema / WASM — fallback
    let pid = pm.spawn(&cmd.program, cmd.args.clone());
    pm.exit(pid, 0)?;
    let file = cmd.args.first().map(String::as_str).unwrap_or("?");
    Ok(ok_out(format!(
        "[runbox] python3 not found in system PATH\n\
         In the browser build, Pyodide provides Python execution automatically.\n\
         To run '{file}' natively, install Python 3 and ensure it's in PATH.\n"
    )))
}

#[cfg(not(target_arch = "wasm32"))]
fn try_spawn_system_python(cmd: &Command, vfs: &mut Vfs) -> std::io::Result<ExecOutput> {
    use crate::network::materialize_vfs;
    use std::process::Command as SysCmd;
    use tempfile::TempDir;

    let tmp = TempDir::new()?;
    materialize_vfs(vfs, tmp.path()).unwrap_or_default();

    // Intentar python3, luego python
    let binary = if SysCmd::new("python3").arg("--version").output().is_ok() {
        "python3"
    } else {
        "python"
    };

    let output = SysCmd::new(binary)
        .args(&cmd.args)
        .current_dir(tmp.path())
        .output()?;

    Ok(ExecOutput {
        stdout: output.stdout,
        stderr: output.stderr,
        exit_code: output.status.code().unwrap_or(1),
    })
}

// ── pip / pip3 ────────────────────────────────────────────────────────────────

fn pip_exec(cmd: &Command, vfs: &mut Vfs, pm: &mut ProcessManager) -> Result<ExecOutput> {
    let sub = cmd.args.first().map(String::as_str).unwrap_or("");

    match sub {
        "install" => pip_install(cmd, vfs, pm),
        "list" => pip_list(vfs, pm, cmd),
        "show" => pip_show(cmd, vfs, pm),
        "freeze" => pip_freeze(vfs, pm, cmd),
        _ => Err(RunboxError::Runtime(format!(
            "pip: unknown subcommand '{sub}'"
        ))),
    }
}

fn pip_install(cmd: &Command, vfs: &mut Vfs, pm: &mut ProcessManager) -> Result<ExecOutput> {
    if cmd.args.iter().any(|a| a == "-r") {
        return pip_install_requirements(cmd, vfs, pm);
    }

    let packages: Vec<String> = cmd
        .args
        .iter()
        .skip(1)
        .filter(|a| !a.starts_with('-'))
        .cloned()
        .collect();

    if packages.is_empty() {
        return Err(RunboxError::Runtime(
            "pip install: specify package(s)".into(),
        ));
    }

    let pid = pm.spawn("pip", cmd.args.clone());

    // Registrar en site-packages del VFS
    for pkg in &packages {
        let (name, ver) = pkg
            .split_once("==")
            .map(|(n, v)| (n, v.to_string()))
            .unwrap_or((pkg.as_str(), "latest".to_string()));
        vfs.write(
            &format!("/site-packages/{name}-{ver}.dist-info/METADATA"),
            format!("Name: {name}\nVersion: {ver}\n").into_bytes(),
        )?;
    }

    pm.exit(pid, 0)?;
    Ok(ok_out(format!(
        "Collecting packages...\nSuccessfully installed {}\n",
        packages.join(" ")
    )))
}

fn pip_install_requirements(
    cmd: &Command,
    vfs: &Vfs,
    pm: &mut ProcessManager,
) -> Result<ExecOutput> {
    let req_file = cmd
        .args
        .iter()
        .skip_while(|a| *a != "-r")
        .nth(1)
        .map(String::as_str)
        .unwrap_or("requirements.txt");

    let path = if req_file.starts_with('/') {
        req_file.to_string()
    } else {
        format!("/{req_file}")
    };
    let content = vfs
        .read(&path)
        .map(|b| String::from_utf8_lossy(b).into_owned())
        .unwrap_or_default();

    let packages: Vec<&str> = content
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect();

    let pid = pm.spawn("pip", cmd.args.clone());
    pm.exit(pid, 0)?;
    Ok(ok_out(format!(
        "Installing {} packages from {req_file}...\nDone.\n",
        packages.len()
    )))
}

fn pip_list(vfs: &Vfs, pm: &mut ProcessManager, cmd: &Command) -> Result<ExecOutput> {
    let packages = vfs.list("/site-packages").unwrap_or_default();
    let pid = pm.spawn("pip", cmd.args.clone());
    pm.exit(pid, 0)?;
    if packages.is_empty() {
        return Ok(ok_out("Package    Version\n---------- -------\n"));
    }
    let rows = packages
        .iter()
        .filter(|p| p.ends_with(".dist-info"))
        .map(|p| {
            let name = p.replace(".dist-info", "");
            format!("{:<20} installed", name)
        })
        .collect::<Vec<_>>()
        .join("\n");
    Ok(ok_out(format!(
        "Package    Version\n---------- -------\n{rows}\n"
    )))
}

fn pip_show(cmd: &Command, vfs: &Vfs, pm: &mut ProcessManager) -> Result<ExecOutput> {
    let pkg = cmd
        .args
        .get(1)
        .ok_or_else(|| RunboxError::Runtime("pip show: specify a package".into()))?;
    let pid = pm.spawn("pip", cmd.args.clone());
    pm.exit(pid, 0)?;
    if let Ok(meta) = vfs.read(&format!("/site-packages/{pkg}-latest.dist-info/METADATA")) {
        Ok(ok_out(String::from_utf8_lossy(meta)))
    } else {
        Ok(ExecOutput {
            stdout: vec![],
            stderr: format!("WARNING: Package '{pkg}' not found\n").into_bytes(),
            exit_code: 1,
        })
    }
}

fn pip_freeze(vfs: &Vfs, pm: &mut ProcessManager, cmd: &Command) -> Result<ExecOutput> {
    let packages = vfs.list("/site-packages").unwrap_or_default();
    let pid = pm.spawn("pip", cmd.args.clone());
    pm.exit(pid, 0)?;
    let freeze = packages
        .iter()
        .filter(|p| p.ends_with(".dist-info"))
        .map(|p| p.replace(".dist-info", "").replace('-', "=="))
        .collect::<Vec<_>>()
        .join("\n");
    Ok(ok_out(freeze))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn ok_out(s: impl Into<String>) -> ExecOutput {
    ExecOutput {
        stdout: s.into().into_bytes(),
        stderr: vec![],
        exit_code: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shell::Command;
    use crate::vfs::Vfs;
    use crate::process::ProcessManager;

    #[test]
    fn test_python_name() {
        let runtime = PythonRuntime;
        assert_eq!(runtime.name(), "python");
    }

    #[test]
    fn test_pip_install_single_package() {
        let mut vfs = Vfs::new();
        let mut pm = ProcessManager::new();
        let runtime = PythonRuntime;

        let cmd = Command {
            program: "pip".to_string(),
            args: vec!["install".to_string(), "requests==2.28.1".to_string()],
            env: vec![],
        };

        let out = runtime.exec(&cmd, &mut vfs, &mut pm).unwrap();
        assert_eq!(out.exit_code, 0);
        
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(stdout.contains("Successfully installed requests==2.28.1"));

        // Verify VFS has the package metadata
        assert!(vfs.exists("/site-packages/requests-2.28.1.dist-info/METADATA"));
        let meta = vfs.read_string("/site-packages/requests-2.28.1.dist-info/METADATA").unwrap();
        assert!(meta.contains("Name: requests"));
        assert!(meta.contains("Version: 2.28.1"));
    }

    #[test]
    fn test_pip_install_requirements() {
        let mut vfs = Vfs::new();
        let mut pm = ProcessManager::new();
        let runtime = PythonRuntime;

        // Create requirements.txt
        vfs.write("/requirements.txt", b"flask==2.0.1\nnumpy\n".to_vec()).unwrap();

        let cmd = Command {
            program: "pip".to_string(),
            args: vec!["install".to_string(), "-r".to_string(), "requirements.txt".to_string()],
            env: vec![],
        };

        let out = runtime.exec(&cmd, &mut vfs, &mut pm).unwrap();
        assert_eq!(out.exit_code, 0);
        let stdout = String::from_utf8_lossy(&out.stdout);
        println!("STDOUT: {}", stdout);
        assert!(stdout.contains("Installing 2 packages from requirements.txt"));
    }

    #[test]
    fn test_pip_list() {
        let mut vfs = Vfs::new();
        let mut pm = ProcessManager::new();
        let runtime = PythonRuntime;

        vfs.write("/site-packages/flask-2.0.1.dist-info/METADATA", vec![]).unwrap();
        vfs.write("/site-packages/numpy-latest.dist-info/METADATA", vec![]).unwrap();

        let cmd = Command {
            program: "pip".to_string(),
            args: vec!["list".to_string()],
            env: vec![],
        };

        let out = runtime.exec(&cmd, &mut vfs, &mut pm).unwrap();
        assert_eq!(out.exit_code, 0);
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(stdout.contains("flask-2.0.1"));
        assert!(stdout.contains("numpy-latest"));
    }

    #[test]
    fn test_pip_show() {
        let mut vfs = Vfs::new();
        let mut pm = ProcessManager::new();
        let runtime = PythonRuntime;

        vfs.write("/site-packages/pytest-latest.dist-info/METADATA", b"Name: pytest\nVersion: latest\n".to_vec()).unwrap();

        let cmd = Command {
            program: "pip".to_string(),
            args: vec!["show".to_string(), "pytest".to_string()],
            env: vec![],
        };

        let out = runtime.exec(&cmd, &mut vfs, &mut pm).unwrap();
        assert_eq!(out.exit_code, 0);
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(stdout.contains("Name: pytest"));
        assert!(stdout.contains("Version: latest"));
    }

    #[test]
    fn test_pip_freeze() {
        let mut vfs = Vfs::new();
        let mut pm = ProcessManager::new();
        let runtime = PythonRuntime;

        vfs.write("/site-packages/django-4.0.0.dist-info/METADATA", vec![]).unwrap();

        let cmd = Command {
            program: "pip".to_string(),
            args: vec!["freeze".to_string()],
            env: vec![],
        };

        let out = runtime.exec(&cmd, &mut vfs, &mut pm).unwrap();
        assert_eq!(out.exit_code, 0);
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(stdout.contains("django==4.0.0"));
    }
}
