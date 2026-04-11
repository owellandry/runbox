/// Runtime de Bun.
/// Nativo: intenta ejecutar el binario `bun` del sistema usando el VFS materializado.
/// WASM: delega en el callback JS `runbox_js_eval` provisto por el host.
#[cfg(target_arch = "wasm32")]
use js_sys;
use crate::error::{Result, RunboxError};
use crate::vfs::Vfs;
use crate::process::ProcessManager;
use crate::shell::Command;
use super::{ExecOutput, Runtime};

pub struct BunRuntime;

impl Runtime for BunRuntime {
    fn name(&self) -> &'static str { "bun" }

    fn exec(&self, cmd: &Command, vfs: &mut Vfs, pm: &mut ProcessManager) -> Result<ExecOutput> {
        let subcommand = cmd.args.first().map(String::as_str).unwrap_or("");

        match subcommand {
            "run"  => bun_run(cmd, vfs, pm),
            "install" | "i" => bun_install(cmd, vfs, pm),
            "add"  => bun_add(cmd, vfs, pm),
            "build"=> bun_build(cmd, vfs, pm),
            "test" => bun_test(cmd, vfs, pm),
            "repl" => Ok(err_out("bun repl: not supported in sandbox")),
            ""     => Ok(err_out("bun: specify a subcommand")),
            other  => Err(RunboxError::Runtime(format!("bun: unknown subcommand '{other}'"))),
        }
    }
}

// ── Subcomandos ───────────────────────────────────────────────────────────────

fn bun_run(cmd: &Command, vfs: &mut Vfs, pm: &mut ProcessManager) -> Result<ExecOutput> {
    let file = cmd.args.get(1).ok_or_else(|| {
        RunboxError::Runtime("bun run requires a file or script name".into())
    })?;

    // Si parece un script npm (sin extensión y sin /) buscar en package.json
    if !file.contains('.') && !file.contains('/') {
        return run_package_script(file, cmd, vfs, pm);
    }

    let path = if file.starts_with('/') { file.clone() } else { format!("/{file}") };
    if !vfs.exists(&path) {
        return Err(RunboxError::NotFound(path));
    }

    spawn_bun(cmd, vfs, pm, &path)
}

fn bun_install(_cmd: &Command, vfs: &mut Vfs, pm: &mut ProcessManager) -> Result<ExecOutput> {
    // Delegar al PackageManagerRuntime con soporte real de package.json
    use crate::runtime::npm::PackageManagerRuntime;
    let install_cmd = Command::parse("bun install").unwrap();
    PackageManagerRuntime::bun_via_npm().exec(&install_cmd, vfs, pm)
}

fn bun_add(cmd: &Command, vfs: &mut Vfs, pm: &mut ProcessManager) -> Result<ExecOutput> {
    use crate::runtime::npm::PackageManagerRuntime;
    let mut args = vec!["add".to_string()];
    args.extend(cmd.args.iter().skip(1).cloned());
    let add_cmd = Command { program: "bun".into(), args, env: vec![] };
    PackageManagerRuntime::bun_via_npm().exec(&add_cmd, vfs, pm)
}

fn bun_build(cmd: &Command, _vfs: &mut Vfs, pm: &mut ProcessManager) -> Result<ExecOutput> {
    let pid = pm.spawn("bun", cmd.args.clone());
    pm.exit(pid, 0)?;
    Ok(ok_out("[bun build] bundling... (native bun required for full execution)\n"))
}

fn bun_test(cmd: &Command, vfs: &mut Vfs, pm: &mut ProcessManager) -> Result<ExecOutput> {
    // Buscar archivos *.test.ts / *.spec.ts en el VFS
    let test_files = find_test_files(vfs);
    if test_files.is_empty() {
        let pid = pm.spawn("bun", cmd.args.clone());
        pm.exit(pid, 0)?;
        return Ok(ok_out("No test files found (*.test.ts / *.spec.ts)\n"));
    }
    spawn_bun(cmd, vfs, pm, "")
}

// ── Ejecutor ──────────────────────────────────────────────────────────────────

/// Intenta ejecutar el binario `bun` del sistema; si no está disponible
/// usa boa_engine (native) o js_sys::eval (WASM).
fn spawn_bun(cmd: &Command, vfs: &mut Vfs, pm: &mut ProcessManager, file_path: &str) -> Result<ExecOutput> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        // 1. Intentar bun del sistema
        if let Ok(output) = try_spawn_system_bun(cmd, vfs) {
            let pid = pm.spawn("bun", cmd.args.clone());
            pm.exit(pid, output.exit_code)?;
            return Ok(output);
        }

        // 2. Fallback: boa_engine si el archivo está en VFS
        if !file_path.is_empty() {
            if let Ok(source) = vfs.read(file_path) {
                let src = String::from_utf8_lossy(source).into_owned();
                let is_ts = file_path.ends_with(".ts") || file_path.ends_with(".tsx");
                let out = super::js_engine::run(&src, is_ts);
                let pid = pm.spawn("bun", cmd.args.clone());
                pm.exit(pid, out.exit_code)?;
                return Ok(ExecOutput {
                    stdout:    out.stdout.into_bytes(),
                    stderr:    out.stderr.into_bytes(),
                    exit_code: out.exit_code,
                });
            }
        }
    }

    // WASM: js_sys::eval vía el motor del browser
    #[cfg(target_arch = "wasm32")]
    if !file_path.is_empty() {
        if let Ok(source) = vfs.read(file_path) {
            let src = String::from_utf8_lossy(source).into_owned();
            let is_ts = file_path.ends_with(".ts") || file_path.ends_with(".tsx");

            // Precargar node_modules del VFS en globalThis para que require() funcione
            // con cualquier paquete instalado (react-icons, lodash, etc.)
            preload_vfs_modules(vfs);

            let out = super::js_engine::run(&src, is_ts);
            let pid = pm.spawn("bun", cmd.args.clone());
            pm.exit(pid, out.exit_code)?;
            return Ok(ExecOutput {
                stdout:    out.stdout.into_bytes(),
                stderr:    out.stderr.into_bytes(),
                exit_code: out.exit_code,
            });
        }
    }

    let pid = pm.spawn("bun", cmd.args.clone());
    pm.exit(pid, 1)?;
    let file = cmd.args.get(1).map(String::as_str).unwrap_or("?");
    Ok(err_out(format!("bun: could not execute '{file}'")))
}

#[cfg(not(target_arch = "wasm32"))]
fn try_spawn_system_bun(cmd: &Command, vfs: &mut Vfs) -> std::io::Result<ExecOutput> {
    use std::process::Command as SysCmd;
    use tempfile::TempDir;
    use crate::network::materialize_vfs;

    let tmp = TempDir::new()?;
    materialize_vfs(vfs, tmp.path()).unwrap_or_default();

    let output = SysCmd::new("bun")
        .args(&cmd.args)
        .current_dir(tmp.path())
        .output()?;

    Ok(ExecOutput {
        stdout:    output.stdout,
        stderr:    output.stderr,
        exit_code: output.status.code().unwrap_or(1),
    })
}

fn run_package_script(script: &str, cmd: &Command, vfs: &mut Vfs, pm: &mut ProcessManager) -> Result<ExecOutput> {
    use crate::runtime::npm::PackageManagerRuntime;
    let mut args = vec!["run".to_string(), script.to_string()];
    args.extend(cmd.args.iter().skip(2).cloned());
    let run_cmd = Command { program: "bun".into(), args, env: cmd.env.clone() };
    PackageManagerRuntime::bun_via_npm().exec(&run_cmd, vfs, pm)
}

fn find_test_files(vfs: &Vfs) -> Vec<String> {
    let mut found = vec![];
    collect_tests(vfs, "/", &mut found);
    found
}

fn collect_tests(vfs: &Vfs, path: &str, out: &mut Vec<String>) {
    let entries = match vfs.list(path) { Ok(e) => e, Err(_) => return };
    for entry in entries {
        let full = if path == "/" { format!("/{entry}") } else { format!("{path}/{entry}") };
        if vfs.read(&full).is_ok() {
            if entry.contains(".test.") || entry.contains(".spec.") {
                out.push(full);
            }
        } else {
            collect_tests(vfs, &full, out);
        }
    }
}

/// Escanea /node_modules del VFS y serializa todos los archivos JS/JSON
/// en globalThis.__vfs_modules para que require() los cargue en eval().
#[cfg(target_arch = "wasm32")]
fn preload_vfs_modules(vfs: &crate::vfs::Vfs) {
    let mut modules: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    collect_module_files(vfs, "/node_modules", &mut modules);

    if modules.is_empty() { return; }

    let json = match serde_json::to_string(&modules) {
        Ok(j) => j,
        Err(_) => return,
    };

    let script = format!("globalThis.__vfs_modules = Object.assign(globalThis.__vfs_modules || {{}}, {json});");
    let _ = js_sys::eval(&script);
}

#[cfg(target_arch = "wasm32")]
fn collect_module_files(vfs: &crate::vfs::Vfs, path: &str, out: &mut std::collections::HashMap<String, String>) {
    let entries = match vfs.list(path) { Ok(e) => e, Err(_) => return };
    for entry in entries {
        let full = format!("{path}/{entry}");
        // Saltar archivos .wasm, .map, binarios, y carpetas con muchos archivos irrelevantes
        if entry.ends_with(".wasm") || entry.ends_with(".map") || entry.ends_with(".md")
            || entry.ends_with(".ts") && !entry.ends_with(".d.ts") { continue; }

        if let Ok(bytes) = vfs.read(&full) {
            if let Ok(content) = std::str::from_utf8(bytes) {
                let key = full.strip_prefix("/node_modules/").unwrap_or(&full).to_string();
                out.insert(key, content.to_string());
            }
        } else {
            collect_module_files(vfs, &full, out);
        }
    }
}

fn ok_out(s: impl AsRef<str>) -> ExecOutput {
    ExecOutput { stdout: s.as_ref().as_bytes().to_vec(), stderr: vec![], exit_code: 0 }
}

fn err_out(s: impl AsRef<str>) -> ExecOutput {
    ExecOutput { stdout: vec![], stderr: s.as_ref().as_bytes().to_vec(), exit_code: 1 }
}
