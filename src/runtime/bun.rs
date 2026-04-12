use super::{ExecOutput, Runtime};
use crate::error::{Result, RunboxError};
use crate::process::ProcessManager;
use crate::shell::Command;
use crate::vfs::Vfs;
/// Runtime de Bun.
/// Nativo: intenta ejecutar el binario `bun` del sistema usando el VFS materializado.
/// WASM: delega en el callback JS `runbox_js_eval` provisto por el host.
#[cfg(target_arch = "wasm32")]
use js_sys;

pub struct BunRuntime;

impl Runtime for BunRuntime {
    fn name(&self) -> &'static str {
        "bun"
    }

    fn exec(&self, cmd: &Command, vfs: &mut Vfs, pm: &mut ProcessManager) -> Result<ExecOutput> {
        if matches!(cmd.program.as_str(), "node" | "nodejs" | "tsx" | "ts-node") {
            let mut args = vec!["run".to_string()];
            args.extend(cmd.args.clone());
            let shimmed = Command {
                program: "bun".into(),
                args,
                env: cmd.env.clone(),
            };
            return bun_run(&shimmed, vfs, pm);
        }

        let subcommand = cmd.args.first().map(String::as_str).unwrap_or("");

        match subcommand {
            "run" => bun_run(cmd, vfs, pm),
            "install" | "i" => bun_install(cmd, vfs, pm),
            "add" => bun_add(cmd, vfs, pm),
            "build" => bun_build(cmd, vfs, pm),
            "test" => bun_test(cmd, vfs, pm),
            "repl" => Ok(err_out("bun repl: not supported in sandbox")),
            "" => Ok(err_out("bun: specify a subcommand")),
            other => Err(RunboxError::Runtime(format!(
                "bun: unknown subcommand '{other}'"
            ))),
        }
    }
}

// ── Subcomandos ───────────────────────────────────────────────────────────────

fn bun_run(cmd: &Command, vfs: &mut Vfs, pm: &mut ProcessManager) -> Result<ExecOutput> {
    let file = cmd
        .args
        .get(1)
        .ok_or_else(|| RunboxError::Runtime("bun run requires a file or script name".into()))?;

    // Si parece un script npm (sin extensión y sin /) buscar en package.json
    if !file.contains('.') && !file.contains('/') {
        return run_package_script(file, cmd, vfs, pm);
    }

    let path = if file.starts_with('/') {
        file.clone()
    } else {
        format!("/{file}")
    };
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
    let add_cmd = Command {
        program: "bun".into(),
        args,
        env: vec![],
    };
    PackageManagerRuntime::bun_via_npm().exec(&add_cmd, vfs, pm)
}

fn bun_build(cmd: &Command, _vfs: &mut Vfs, pm: &mut ProcessManager) -> Result<ExecOutput> {
    let pid = pm.spawn("bun", cmd.args.clone());
    pm.exit(pid, 0)?;
    Ok(ok_out(
        "[bun build] bundling... (native bun required for full execution)\n",
    ))
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
fn spawn_bun(
    cmd: &Command,
    vfs: &mut Vfs,
    pm: &mut ProcessManager,
    file_path: &str,
) -> Result<ExecOutput> {
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
                    stdout: out.stdout.into_bytes(),
                    stderr: out.stderr.into_bytes(),
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
                stdout: out.stdout.into_bytes(),
                stderr: out.stderr.into_bytes(),
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
    use crate::network::materialize_vfs;
    use std::process::Command as SysCmd;
    use tempfile::TempDir;

    let tmp = TempDir::new()?;
    materialize_vfs(vfs, tmp.path()).unwrap_or_default();

    let output = SysCmd::new("bun")
        .args(&cmd.args)
        .current_dir(tmp.path())
        .output()?;

    Ok(ExecOutput {
        stdout: output.stdout,
        stderr: output.stderr,
        exit_code: output.status.code().unwrap_or(1),
    })
}

fn run_package_script(
    script: &str,
    cmd: &Command,
    vfs: &mut Vfs,
    pm: &mut ProcessManager,
) -> Result<ExecOutput> {
    use crate::runtime::npm::PackageManagerRuntime;
    let mut args = vec!["run".to_string(), script.to_string()];
    args.extend(cmd.args.iter().skip(2).cloned());
    let run_cmd = Command {
        program: "bun".into(),
        args,
        env: cmd.env.clone(),
    };
    PackageManagerRuntime::bun_via_npm().exec(&run_cmd, vfs, pm)
}

fn find_test_files(vfs: &Vfs) -> Vec<String> {
    let mut found = vec![];
    collect_tests(vfs, "/", &mut found);
    found
}

fn collect_tests(vfs: &Vfs, path: &str, out: &mut Vec<String>) {
    let entries = match vfs.list(path) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries {
        let full = if path == "/" {
            format!("/{entry}")
        } else {
            format!("{path}/{entry}")
        };
        if vfs.read(&full).is_ok() {
            if entry.contains(".test.") || entry.contains(".spec.") {
                out.push(full);
            }
        } else {
            collect_tests(vfs, &full, out);
        }
    }
}

/// Serializa archivos del proyecto y paquetes npm en globalThis.__vfs_modules.
/// Cada paquete se evalúa en su propio eval() independiente para que el fallo
/// de un paquete grande (ej. react-icons) no impida cargar los demás.
#[cfg(target_arch = "wasm32")]
fn preload_vfs_modules(vfs: &crate::vfs::Vfs) {
    // Asegurar que __vfs_modules existe
    let _ = js_sys::eval("if(!globalThis.__vfs_modules)globalThis.__vfs_modules={};");

    // 1. Archivos del proyecto — siempre pequeños, eval propio
    {
        let mut project: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();
        collect_project_files(vfs, "/", &mut project);
        eval_into_vfs_modules(&project);
    }

    // 2. Un eval por paquete npm — aísla fallos de paquetes grandes
    if let Ok(pkg_names) = vfs.list("/node_modules") {
        for pkg_name in pkg_names {
            let pkg_root = format!("/node_modules/{pkg_name}");
            let mut pkg_files: std::collections::HashMap<String, String> =
                std::collections::HashMap::new();
            collect_npm_package(vfs, &pkg_root, &pkg_name, &mut pkg_files);
            eval_into_vfs_modules(&pkg_files);
        }
    }
}

/// Hace `Object.assign(globalThis.__vfs_modules, map)` via eval.
/// Si el JSON es demasiado grande y el eval falla, se ignora silenciosamente.
#[cfg(target_arch = "wasm32")]
fn eval_into_vfs_modules(map: &std::collections::HashMap<String, String>) {
    if map.is_empty() {
        return;
    }
    if let Ok(json) = serde_json::to_string(map) {
        let script = format!("Object.assign(globalThis.__vfs_modules,{json});");
        let _ = js_sys::eval(&script);
    }
}

/// Carga los archivos relevantes de un único paquete npm:
/// - package.json (para resolver el entry point)
/// - el archivo main/index y sus dependencias directas dentro del paquete
/// Evita cargar miles de archivos de paquetes grandes como react-icons.
#[cfg(target_arch = "wasm32")]
fn pick_export_path(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(s) => Some(s.trim_start_matches("./").to_string()),
        serde_json::Value::Array(arr) => {
            for item in arr {
                if let Some(path) = pick_export_path(item) {
                    return Some(path);
                }
            }
            None
        }
        serde_json::Value::Object(map) => {
            for key in ["require", "node", "default", "import", "browser"] {
                if let Some(v) = map.get(key) {
                    if let Some(path) = pick_export_path(v) {
                        return Some(path);
                    }
                }
            }
            for v in map.values() {
                if let Some(path) = pick_export_path(v) {
                    return Some(path);
                }
            }
            None
        }
        _ => None,
    }
}

#[cfg(target_arch = "wasm32")]
fn package_entry_candidates(pkg_json_bytes: &[u8]) -> Vec<String> {
    let mut candidates = Vec::<String>::new();

    if let Ok(v) = serde_json::from_slice::<serde_json::Value>(pkg_json_bytes) {
        if let Some(exports) = v.get("exports") {
            let target = exports.get(".").unwrap_or(exports);
            if let Some(path) = pick_export_path(target) {
                candidates.push(path);
            }
        }

        for field in ["main", "module", "browser"] {
            if let Some(s) = v.get(field).and_then(|x| x.as_str()) {
                candidates.push(s.trim_start_matches("./").to_string());
            }
        }
    }

    candidates.push("index.js".to_string());
    candidates.push("index.cjs".to_string());
    candidates.push("index.mjs".to_string());

    // Keep insertion order while removing duplicates.
    let mut deduped = Vec::new();
    for c in candidates {
        if !c.is_empty() && !deduped.contains(&c) {
            deduped.push(c);
        }
    }
    deduped
}

#[cfg(target_arch = "wasm32")]
fn collect_npm_package(
    vfs: &crate::vfs::Vfs,
    pkg_root: &str,
    pkg_name: &str,
    out: &mut std::collections::HashMap<String, String>,
) {
    // Siempre cargar package.json
    let pkg_json_path = format!("{pkg_root}/package.json");
    if let Ok(bytes) = vfs.read(&pkg_json_path) {
        if let Ok(content) = std::str::from_utf8(bytes) {
            let key = format!("{pkg_name}/package.json");
            out.insert(key, content.to_string());
        }
    }

    let entry_candidates = vfs
        .read(&pkg_json_path)
        .map(package_entry_candidates)
        .unwrap_or_else(|_| {
            vec![
                "index.js".to_string(),
                "index.cjs".to_string(),
                "index.mjs".to_string(),
            ]
        });

    for entry_file in entry_candidates {
        // Cargar entry candidate.
        let entry_path = format!("{pkg_root}/{entry_file}");
        load_file_to_map(vfs, &entry_path, pkg_name, &entry_file, out);

        // Cargar archivos hermanos del directorio del entry (1 nivel).
        let entry_dir = if let Some(pos) = entry_file.rfind('/') {
            format!("{pkg_root}/{}", &entry_file[..pos])
        } else {
            pkg_root.to_string()
        };

        if let Ok(siblings) = vfs.list(&entry_dir) {
            for sib in siblings {
                if sib.ends_with(".wasm") || sib.ends_with(".map") || sib.ends_with(".md") {
                    continue;
                }
                let ext = sib.rsplit('.').next().unwrap_or("");
                if !matches!(ext, "js" | "mjs" | "cjs" | "json") {
                    continue;
                }
                let sib_full = format!("{entry_dir}/{sib}");
                let sib_rel = sib_full
                    .strip_prefix(&format!("{pkg_root}/"))
                    .unwrap_or(&sib);
                load_file_to_map(vfs, &sib_full, pkg_name, sib_rel, out);
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn load_file_to_map(
    vfs: &crate::vfs::Vfs,
    full_path: &str,
    pkg_name: &str,
    rel: &str,
    out: &mut std::collections::HashMap<String, String>,
) {
    if let Ok(bytes) = vfs.read(full_path) {
        if let Ok(content) = std::str::from_utf8(bytes) {
            let key = format!("{pkg_name}/{}", rel.trim_start_matches("./"));
            out.insert(key, content.to_string());
        }
    }
}

/// Carga archivos locales del proyecto (fuera de node_modules) para que
/// require('./components/Foo') funcione en el eval del sandbox.
/// Guarda cada archivo con dos claves: `path/file.js` y `./path/file.js`.
#[cfg(target_arch = "wasm32")]
fn collect_project_files(
    vfs: &crate::vfs::Vfs,
    path: &str,
    out: &mut std::collections::HashMap<String, String>,
) {
    let entries = match vfs.list(path) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in &entries {
        // Saltar node_modules y archivos ocultos
        if entry == "node_modules" || entry.starts_with('.') {
            continue;
        }

        let full = if path == "/" {
            format!("/{entry}")
        } else {
            format!("{path}/{entry}")
        };

        if let Ok(bytes) = vfs.read(&full) {
            let ext = entry.rsplit('.').next().unwrap_or("");
            if matches!(ext, "js" | "mjs" | "cjs" | "jsx" | "ts" | "tsx" | "json") {
                if let Ok(content) = std::str::from_utf8(bytes) {
                    // Clave sin slash inicial: "components/Foo.js"
                    let bare = full.trim_start_matches('/').to_string();
                    out.insert(bare.clone(), content.to_string());
                    // Clave con ./ para require desde el directorio raíz: "./components/Foo.js"
                    out.insert(format!("./{bare}"), content.to_string());
                }
            }
        } else {
            // Es un directorio — recursión
            collect_project_files(vfs, &full, out);
        }
    }
}

fn ok_out(s: impl AsRef<str>) -> ExecOutput {
    ExecOutput {
        stdout: s.as_ref().as_bytes().to_vec(),
        stderr: vec![],
        exit_code: 0,
    }
}

fn err_out(s: impl AsRef<str>) -> ExecOutput {
    ExecOutput {
        stdout: vec![],
        stderr: s.as_ref().as_bytes().to_vec(),
        exit_code: 1,
    }
}
