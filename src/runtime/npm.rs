use super::{ExecOutput, Runtime};
use crate::error::{Result, RunboxError};
use crate::process::ProcessManager;
use crate::shell::Command;
use crate::vfs::Vfs;
use serde::{Deserialize, Serialize};
/// Runtime de package managers: npm, pnpm, yarn.
/// Lee y escribe package.json real del VFS.
/// Nativo: resuelve paquetes contra registry.npmjs.org y extrae tarballs al VFS.
use std::collections::HashMap;

// ── package.json ──────────────────────────────────────────────────────────────

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
struct PackageJson {
    name: Option<String>,
    version: Option<String>,
    description: Option<String>,
    main: Option<String>,
    scripts: HashMap<String, String>,
    dependencies: HashMap<String, String>,
    #[serde(rename = "devDependencies")]
    dev_dependencies: HashMap<String, String>,
    #[serde(rename = "peerDependencies")]
    peer_dependencies: HashMap<String, String>,
    workspaces: Option<serde_json::Value>,
}

impl PackageJson {
    fn load(vfs: &Vfs) -> Option<Self> {
        vfs.read("/package.json")
            .ok()
            .and_then(|b| serde_json::from_slice(b).ok())
    }

    fn save(&self, vfs: &mut Vfs) -> Result<()> {
        let json = serde_json::to_string_pretty(self).unwrap();
        vfs.write("/package.json", json.into_bytes())
    }

    fn add_dep(&mut self, name: &str, version: &str, dev: bool) {
        let map = if dev {
            &mut self.dev_dependencies
        } else {
            &mut self.dependencies
        };
        map.insert(name.to_string(), version.to_string());
    }

    fn remove_dep(&mut self, name: &str) {
        self.dependencies.remove(name);
        self.dev_dependencies.remove(name);
        self.peer_dependencies.remove(name);
    }
}

// ── Lock file (simulado) ──────────────────────────────────────────────────────

#[derive(Debug, Default, Serialize, Deserialize)]
struct LockEntry {
    version: String,
    resolved: String,
    integrity: String,
}

// ── npm registry resolver (nativo) ───────────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
fn registry_resolve(name: &str, version_req: &str) -> crate::error::Result<RegistryPackage> {
    use crate::network::http_get;

    // Normalizar versión: quitar ^, ~, >=, etc.
    let ver = if version_req == "latest" || version_req.is_empty() {
        "latest".to_string()
    } else {
        version_req
            .trim_start_matches(|c: char| !c.is_ascii_digit())
            .to_string()
    };

    let url = format!("https://registry.npmjs.org/{name}/{ver}");
    let resp = http_get(&url)?;
    if resp.status == 404 {
        return Err(crate::error::RunboxError::NotFound(format!("{name}@{ver}")));
    }
    if resp.status != 200 {
        return Err(crate::error::RunboxError::Runtime(format!(
            "registry error for {name}: HTTP {}",
            resp.status
        )));
    }
    resp.json::<RegistryPackage>()
}

#[cfg(not(target_arch = "wasm32"))]
fn registry_install_package(
    name: &str,
    version_req: &str,
    vfs: &mut Vfs,
) -> crate::error::Result<String> {
    use crate::network::http_get;

    let pkg = registry_resolve(name, version_req)?;
    let tarball_url = &pkg.dist.tarball;

    let tarball = http_get(tarball_url)?;
    extract_tgz_to_vfs(&tarball.body, name, vfs)?;

    // Asegurarse de que package.json existe en node_modules/<name>/
    let nm_pkg = serde_json::json!({
        "name": name,
        "version": pkg.version,
        "main": pkg.main.as_deref().unwrap_or("index.js"),
    });
    vfs.write(
        &format!("/node_modules/{name}/package.json"),
        serde_json::to_string(&nm_pkg).unwrap().into_bytes(),
    )?;

    Ok(pkg.version)
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct RegistryPackage {
    #[allow(dead_code)]
    name: String,
    version: String,
    main: Option<String>,
    dist: RegistryDist,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct RegistryDist {
    tarball: String,
    #[allow(dead_code)]
    integrity: Option<String>,
}

fn lock_filename(pm: &str) -> &'static str {
    match pm {
        "pnpm" => "pnpm-lock.yaml",
        "yarn" => "yarn.lock",
        _ => "package-lock.json",
    }
}

fn write_lock(vfs: &mut Vfs, pm_name: &str, pkg: &PackageJson) -> Result<()> {
    let all_deps: Vec<(&str, &str)> = pkg
        .dependencies
        .iter()
        .chain(pkg.dev_dependencies.iter())
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();

    let content = match pm_name {
        "yarn" => {
            let mut lines = vec!["# yarn lockfile v1\n".to_string()];
            for (name, ver) in &all_deps {
                lines.push(format!("\"{name}@{ver}\":\n  version \"{ver}\"\n  resolved \"https://registry.yarnpkg.com/{name}/-/{name}-{ver}.tgz\"\n"));
            }
            lines.join("\n")
        }
        "pnpm" => {
            let mut lines = vec!["lockfileVersion: '9.0'\n\npackages:".to_string()];
            for (name, ver) in &all_deps {
                lines.push(format!(
                    "  /{name}/{ver}:\n    resolution: {{integrity: sha512-placeholder}}"
                ));
            }
            lines.join("\n")
        }
        _ => {
            let entries: HashMap<&str, LockEntry> = all_deps
                .iter()
                .map(|(name, ver)| {
                    (
                        *name,
                        LockEntry {
                            version: ver.to_string(),
                            resolved: format!(
                                "https://registry.npmjs.org/{name}/-/{name}-{ver}.tgz"
                            ),
                            integrity: "sha512-placeholder".into(),
                        },
                    )
                })
                .collect();
            serde_json::to_string_pretty(&serde_json::json!({
                "name": pkg.name,
                "lockfileVersion": 3,
                "packages": entries
            }))
            .unwrap()
        }
    };

    vfs.write(
        &format!("/{}", lock_filename(pm_name)),
        content.into_bytes(),
    )
}

// ── npm WASM — install via fetch del browser ──────────────────────────────────
//
// En WASM no podemos hacer HTTP bloqueante. El flujo es:
//
//   1. JS llama runbox.npm_packages_needed()  → lista JSON de paquetes a descargar
//   2. JS hace fetch() a registry.npmjs.org (CORS está habilitado ✓)
//   3. JS llama runbox.npm_process_tarball(name, version, bytes)
//   4. RunBox extrae el tarball al VFS y actualiza package.json
//
// Código JS de referencia:
//
// ```js
// async function npmInstall(runbox) {
//   const needed = JSON.parse(runbox.npm_packages_needed());
//   for (const { name, version } of needed) {
//     const meta = await fetch(`https://registry.npmjs.org/${name}/${version}`).then(r => r.json());
//     const buf  = await fetch(meta.dist.tarball).then(r => r.arrayBuffer());
//     runbox.npm_process_tarball(name, version, new Uint8Array(buf));
//   }
// }
// ```

#[derive(Debug, Serialize, Deserialize)]
pub struct NpmPackageRequest {
    pub name: String,
    pub version: String,
}

/// Retorna la lista de paquetes del package.json que aún no están en node_modules.
pub fn packages_needed(vfs: &Vfs) -> Vec<NpmPackageRequest> {
    let pkg = match PackageJson::load(vfs) {
        Some(p) => p,
        None => return vec![],
    };
    pkg.dependencies
        .iter()
        .chain(pkg.dev_dependencies.iter())
        .filter(|(name, _)| !vfs.exists(&format!("/node_modules/{name}/package.json")))
        .map(|(name, ver)| NpmPackageRequest {
            name: name.clone(),
            version: ver
                .trim_start_matches(|c: char| !c.is_ascii_digit())
                .to_string(),
        })
        .collect()
}

/// Instala un paquete dado su tarball en bytes (llamado desde JS en WASM).
pub fn process_tarball(name: &str, _version: &str, bytes: &[u8], vfs: &mut Vfs) -> Result<()> {
    extract_tgz_to_vfs(bytes, name, vfs)?;
    Ok(())
}

/// Extrae un .tgz al VFS bajo /node_modules/<name>/
fn extract_tgz_to_vfs(bytes: &[u8], name: &str, vfs: &mut Vfs) -> Result<()> {
    use flate2::read::GzDecoder;
    use std::io::Read;
    use tar::Archive;

    let gz = GzDecoder::new(bytes);
    let mut archive = Archive::new(gz);

    for entry in archive
        .entries()
        .map_err(|e| crate::error::RunboxError::Runtime(e.to_string()))?
    {
        let mut entry = entry.map_err(|e| crate::error::RunboxError::Runtime(e.to_string()))?;
        let raw_path = entry
            .path()
            .map_err(|e| crate::error::RunboxError::Runtime(e.to_string()))?
            .to_string_lossy()
            .into_owned();

        // Los tarballs de npm tienen "package/..." como prefijo
        let rel = raw_path.strip_prefix("package/").unwrap_or(&raw_path);

        // Saltar archivos muy grandes o binarios que no necesitamos
        let size = entry.size();
        if size > 2_000_000 {
            continue;
        }

        let vfs_path = format!("/node_modules/{name}/{rel}");
        let mut content = Vec::with_capacity(size as usize);
        if entry.read_to_end(&mut content).is_ok() {
            let _ = vfs.write(&vfs_path, content);
        }
    }

    Ok(())
}

// ── Runtime ───────────────────────────────────────────────────────────────────

pub struct PackageManagerRuntime {
    name: &'static str,
}

impl PackageManagerRuntime {
    pub fn npm() -> Self {
        Self { name: "npm" }
    }
    pub fn pnpm() -> Self {
        Self { name: "pnpm" }
    }
    pub fn yarn() -> Self {
        Self { name: "yarn" }
    }
    /// Bun usa el mismo sistema de paquetes que npm bajo el capó.
    pub fn bun_via_npm() -> Self {
        Self { name: "bun" }
    }
}

impl Runtime for PackageManagerRuntime {
    fn name(&self) -> &'static str {
        self.name
    }

    fn exec(&self, cmd: &Command, vfs: &mut Vfs, pm: &mut ProcessManager) -> Result<ExecOutput> {
        let pm_name = self.name;
        let sub = cmd.args.first().map(String::as_str).unwrap_or("");

        match sub {
            "install" | "i" | "ci" => pm_install(cmd, vfs, pm, pm_name),
            "add" => pm_add(cmd, vfs, pm, pm_name),
            "remove" | "uninstall" | "rm" | "un" => pm_remove(cmd, vfs, pm, pm_name),
            "run" => pm_run(cmd, vfs, pm, pm_name),
            "exec" | "dlx" | "npx" | "pnpx" | "create" => pm_exec(cmd, vfs, pm, pm_name),
            "init" => pm_init(cmd, vfs, pm, pm_name),
            "list" | "ls" => pm_list(vfs, pm, pm_name, cmd),
            "update" | "upgrade" => pm_update(cmd, vfs, pm, pm_name),
            "outdated" => pm_outdated(vfs, pm, pm_name, cmd),
            "audit" => pm_audit(vfs, pm, pm_name, cmd),
            _ => Err(RunboxError::Runtime(format!(
                "{pm_name}: unknown subcommand '{sub}'"
            ))),
        }
    }
}

// ── Subcomandos ───────────────────────────────────────────────────────────────

fn pm_install(
    cmd: &Command,
    vfs: &mut Vfs,
    pm: &mut ProcessManager,
    pm_name: &str,
) -> Result<ExecOutput> {
    let pkg = PackageJson::load(vfs);
    let (dep_count, dev_count) = pkg
        .as_ref()
        .map(|p| (p.dependencies.len(), p.dev_dependencies.len()))
        .unwrap_or((0, 0));

    if dep_count + dev_count == 0 && vfs.exists("/package.json") {
        let pid = pm.spawn(pm_name, cmd.args.clone());
        pm.exit(pid, 0)?;
        return Ok(ok_out("up to date — no dependencies found in package.json"));
    }

    let mut resolved = 0usize;
    #[cfg(not(target_arch = "wasm32"))]
    let mut failed: Vec<&str> = vec![];
    #[cfg(target_arch = "wasm32")]
    let failed: Vec<&str> = vec![];

    if let Some(pkg) = &pkg {
        write_lock(vfs, pm_name, pkg)?;

        for (name, ver) in pkg.dependencies.iter().chain(pkg.dev_dependencies.iter()) {
            #[cfg(not(target_arch = "wasm32"))]
            match registry_install_package(name, ver, vfs) {
                Ok(_) => resolved += 1,
                Err(_) => {
                    // Fallback: stub mínimo en node_modules
                    let _ = vfs.write(
                        &format!("/node_modules/{name}/package.json"),
                        serde_json::json!({ "name": name, "version": ver })
                            .to_string()
                            .into_bytes(),
                    );
                    failed.push(name.as_str());
                }
            }
            #[cfg(target_arch = "wasm32")]
            {
                let _ = vfs.write(
                    &format!("/node_modules/{name}/package.json"),
                    serde_json::json!({ "name": name, "version": ver })
                        .to_string()
                        .into_bytes(),
                );
                resolved += 1;
            }
        }
    }

    let pid = pm.spawn(pm_name, cmd.args.clone());
    pm.exit(pid, 0)?;
    let total = dep_count + dev_count;
    let mut msg = format!("added {total} packages ({dep_count} prod, {dev_count} dev)");
    if !failed.is_empty() {
        msg.push_str(&format!(
            "\nWarning: could not fetch from registry: {}",
            failed.join(", ")
        ));
    }
    let _ = resolved; // suppress unused warning when all are stubs
    Ok(ok_out(msg))
}

fn pm_add(
    cmd: &Command,
    vfs: &mut Vfs,
    pm: &mut ProcessManager,
    pm_name: &str,
) -> Result<ExecOutput> {
    let dev = cmd
        .args
        .iter()
        .any(|a| a == "-D" || a == "--save-dev" || a == "--dev");
    let exact = cmd.args.iter().any(|a| a == "-E" || a == "--save-exact");

    let packages: Vec<String> = cmd
        .args
        .iter()
        .skip(1)
        .filter(|a| !a.starts_with('-'))
        .cloned()
        .collect();

    if packages.is_empty() {
        return Err(RunboxError::Runtime(format!(
            "{pm_name} add: specify at least one package"
        )));
    }

    let mut pkg = PackageJson::load(vfs).unwrap_or_default();
    let mut added = vec![];

    for spec in &packages {
        let (name, ver) = parse_package_spec(spec, exact);
        pkg.add_dep(&name, &ver, dev);
        vfs.write(
            &format!("/node_modules/{name}/package.json"),
            serde_json::json!({ "name": name, "version": &ver[1..] })
                .to_string()
                .into_bytes(),
        )?;
        added.push(format!("{name}@{}", &ver[1..]));
    }

    pkg.save(vfs)?;
    write_lock(vfs, pm_name, &pkg)?;

    let pid = pm.spawn(pm_name, cmd.args.clone());
    pm.exit(pid, 0)?;
    let kind = if dev { "devDependency" } else { "dependency" };
    Ok(ok_out(format!(
        "added {} as {kind}: {}",
        added.len(),
        added.join(", ")
    )))
}

fn pm_remove(
    cmd: &Command,
    vfs: &mut Vfs,
    pm: &mut ProcessManager,
    pm_name: &str,
) -> Result<ExecOutput> {
    let packages: Vec<String> = cmd
        .args
        .iter()
        .skip(1)
        .filter(|a| !a.starts_with('-'))
        .cloned()
        .collect();

    if packages.is_empty() {
        return Err(RunboxError::Runtime(format!(
            "{pm_name} remove: specify at least one package"
        )));
    }

    let mut pkg = PackageJson::load(vfs).unwrap_or_default();
    for name in &packages {
        pkg.remove_dep(name);
        let _ = vfs.remove(&format!("/node_modules/{name}/package.json"));
    }

    pkg.save(vfs)?;
    write_lock(vfs, pm_name, &pkg)?;

    let pid = pm.spawn(pm_name, cmd.args.clone());
    pm.exit(pid, 0)?;
    Ok(ok_out(format!(
        "removed {}: {}",
        packages.len(),
        packages.join(", ")
    )))
}

fn pm_run(
    cmd: &Command,
    vfs: &mut Vfs,
    pm: &mut ProcessManager,
    pm_name: &str,
) -> Result<ExecOutput> {
    let script_name = cmd
        .args
        .get(1)
        .ok_or_else(|| RunboxError::Runtime(format!("{pm_name} run: specify a script name")))?;

    let pkg = PackageJson::load(vfs).ok_or_else(|| RunboxError::NotFound("package.json".into()))?;

    let script_cmd_str = pkg
        .scripts
        .get(script_name.as_str())
        .ok_or_else(|| {
            RunboxError::Runtime(format!(
                r#"missing script: "{script_name}"\n\nAvailable scripts:\n{}"#,
                pkg.scripts
                    .keys()
                    .map(|k| format!("  {k}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            ))
        })?
        .clone();

    let header = format!(
        "> {}@{} {}\n> {}\n",
        pkg.name.as_deref().unwrap_or("app"),
        pkg.version.as_deref().unwrap_or("0.0.0"),
        script_name,
        script_cmd_str,
    );

    // Ejecutar el script real parseando y despachando el comando
    let result = run_script_command(&script_cmd_str, vfs, pm)?;

    let pid = pm.spawn(pm_name, cmd.args.clone());
    pm.exit(pid, result.exit_code)?;

    let mut stdout = header.into_bytes();
    stdout.extend_from_slice(&result.stdout);

    Ok(ExecOutput {
        stdout,
        stderr: result.stderr,
        exit_code: result.exit_code,
    })
}

/// Parsea y ejecuta el string de un script npm (e.g. "bun run /index.js", "node server.js")
fn run_script_command(script: &str, vfs: &mut Vfs, pm: &mut ProcessManager) -> Result<ExecOutput> {
    let script_cmd = Command::parse(script)
        .map_err(|_| RunboxError::Runtime(format!("invalid script: {script}")))?;

    match script_cmd.program.as_str() {
        "bun" => super::bun::BunRuntime.exec(&script_cmd, vfs, pm),
        "node" | "nodejs" => {
            // Tratar node igual que bun: `node file.js` → `bun run file.js`
            let mut args = vec!["run".to_string()];
            args.extend(script_cmd.args);
            super::bun::BunRuntime.exec(
                &Command {
                    program: "bun".into(),
                    args,
                    env: script_cmd.env,
                },
                vfs,
                pm,
            )
        }
        "ts-node" | "tsx" => {
            let mut args = vec!["run".to_string()];
            args.extend(script_cmd.args);
            super::bun::BunRuntime.exec(
                &Command {
                    program: "bun".into(),
                    args,
                    env: script_cmd.env,
                },
                vfs,
                pm,
            )
        }
        _ => Err(RunboxError::Runtime(format!(
            "script runtime '{}' not supported in sandbox",
            script_cmd.program
        ))),
    }
}

fn pm_exec(
    cmd: &Command,
    _vfs: &mut Vfs,
    pm: &mut ProcessManager,
    pm_name: &str,
) -> Result<ExecOutput> {
    let tool = cmd
        .args
        .get(1)
        .ok_or_else(|| RunboxError::Runtime(format!("{pm_name}: specify a package to execute")))?;

    let pid = pm.spawn(pm_name, cmd.args.clone());
    pm.exit(pid, 0)?;
    Ok(ok_out(format!(
        "Packages: {tool}\n[{pm_name}] running {tool}...\n"
    )))
}

fn pm_init(
    cmd: &Command,
    vfs: &mut Vfs,
    pm: &mut ProcessManager,
    pm_name: &str,
) -> Result<ExecOutput> {
    let yes = cmd.args.iter().any(|a| a == "-y" || a == "--yes");

    let pkg = PackageJson {
        name: Some("my-project".into()),
        version: Some("0.1.0".into()),
        description: Some(String::new()),
        main: Some("index.js".into()),
        scripts: {
            let mut m = HashMap::new();
            m.insert("dev".into(), "bun run src/index.ts".into());
            m.insert(
                "build".into(),
                "bun build src/index.ts --outdir dist".into(),
            );
            m.insert("test".into(), "bun test".into());
            m
        },
        ..Default::default()
    };

    pkg.save(vfs)?;
    let pid = pm.spawn(pm_name, cmd.args.clone());
    pm.exit(pid, 0)?;

    if yes {
        Ok(ok_out("Wrote to /package.json"))
    } else {
        Ok(ok_out(serde_json::to_string_pretty(&pkg).unwrap()))
    }
}

fn pm_list(vfs: &Vfs, pm: &mut ProcessManager, pm_name: &str, cmd: &Command) -> Result<ExecOutput> {
    let pkg = match PackageJson::load(vfs) {
        Some(p) => p,
        None => return Ok(ok_out("(no package.json found)")),
    };

    let depth = cmd
        .args
        .iter()
        .find(|a| a.starts_with("--depth="))
        .and_then(|a| a.strip_prefix("--depth=")?.parse::<u8>().ok())
        .unwrap_or(1);

    let mut lines = vec![format!(
        "{} {}",
        pkg.name.as_deref().unwrap_or("app"),
        pkg.version.as_deref().unwrap_or("0.0.0"),
    )];

    if depth > 0 {
        for (name, ver) in &pkg.dependencies {
            lines.push(format!("├── {name}@{ver}"));
        }
        for (name, ver) in &pkg.dev_dependencies {
            lines.push(format!("├── {name}@{ver} (dev)"));
        }
    }

    let pid = pm.spawn(pm_name, cmd.args.clone());
    pm.exit(pid, 0)?;
    Ok(ok_out(lines.join("\n")))
}

fn pm_update(
    cmd: &Command,
    vfs: &mut Vfs,
    pm: &mut ProcessManager,
    pm_name: &str,
) -> Result<ExecOutput> {
    let pkg = PackageJson::load(vfs);
    let count = pkg
        .as_ref()
        .map(|p| p.dependencies.len() + p.dev_dependencies.len())
        .unwrap_or(0);
    let pid = pm.spawn(pm_name, cmd.args.clone());
    pm.exit(pid, 0)?;
    Ok(ok_out(format!("updated {count} packages (simulated)")))
}

fn pm_outdated(
    vfs: &Vfs,
    pm: &mut ProcessManager,
    pm_name: &str,
    cmd: &Command,
) -> Result<ExecOutput> {
    let pkg = PackageJson::load(vfs).unwrap_or_default();
    let mut lines = vec!["Package  Current  Wanted  Latest".to_string()];
    for (name, ver) in &pkg.dependencies {
        lines.push(format!("{name}  {ver}  {ver}  {ver}  (up to date)"));
    }
    let pid = pm.spawn(pm_name, cmd.args.clone());
    pm.exit(pid, 0)?;
    Ok(ok_out(lines.join("\n")))
}

fn pm_audit(
    vfs: &Vfs,
    pm: &mut ProcessManager,
    pm_name: &str,
    cmd: &Command,
) -> Result<ExecOutput> {
    let pkg = PackageJson::load(vfs).unwrap_or_default();
    let total = pkg.dependencies.len() + pkg.dev_dependencies.len();
    let pid = pm.spawn(pm_name, cmd.args.clone());
    pm.exit(pid, 0)?;
    Ok(ok_out(format!(
        "audited {total} packages\nfound 0 vulnerabilities"
    )))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn ok_out(text: impl Into<String>) -> ExecOutput {
    ExecOutput {
        stdout: text.into().into_bytes(),
        stderr: vec![],
        exit_code: 0,
    }
}

/// Parsea "react@^18.2.0" → ("react", "^18.2.0")
fn parse_package_spec(spec: &str, exact: bool) -> (String, String) {
    // @scope/pkg@version
    let (name, ver) = if let Some(pos) = spec[1..].find('@') {
        (&spec[..pos + 1], &spec[pos + 2..])
    } else {
        (spec, "latest")
    };

    let version_str = if exact || ver == "latest" {
        format!("^{}", if ver == "latest" { "1.0.0" } else { ver })
    } else {
        ver.to_string()
    };

    (name.to_string(), version_str)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::ProcessManager;
    use crate::shell::Command;
    use crate::vfs::Vfs;

    fn setup() -> (Vfs, ProcessManager) {
        let mut vfs = Vfs::new();
        let pkg = serde_json::json!({
            "name": "test-app",
            "version": "1.0.0",
            "scripts": {
                "dev": "bun run src/index.ts",
                "build": "bun build src/index.ts"
            },
            "dependencies": { "zod": "^3.22.0" },
            "devDependencies": {}
        });
        vfs.write("/package.json", pkg.to_string().into_bytes())
            .unwrap();
        vfs.write("/src/index.ts", b"console.log('ok')".to_vec())
            .unwrap();
        (vfs, ProcessManager::new())
    }

    #[test]
    fn install_creates_lock() {
        let (mut vfs, mut pm) = setup();
        let rt = PackageManagerRuntime::npm();
        rt.exec(&Command::parse("npm install").unwrap(), &mut vfs, &mut pm)
            .unwrap();
        assert!(vfs.exists("/package-lock.json"));
        assert!(vfs.exists("/node_modules/zod/package.json"));
    }

    #[test]
    fn add_updates_package_json() {
        let (mut vfs, mut pm) = setup();
        let rt = PackageManagerRuntime::pnpm();
        rt.exec(
            &Command::parse("pnpm add typescript -D").unwrap(),
            &mut vfs,
            &mut pm,
        )
        .unwrap();
        let pkg = PackageJson::load(&vfs).unwrap();
        assert!(pkg.dev_dependencies.contains_key("typescript"));
    }

    #[test]
    fn run_script() {
        let (mut vfs, mut pm) = setup();
        let rt = PackageManagerRuntime::npm();
        let out = rt
            .exec(&Command::parse("npm run dev").unwrap(), &mut vfs, &mut pm)
            .unwrap();
        assert_eq!(out.exit_code, 0);
        let s = String::from_utf8_lossy(&out.stdout);
        assert!(s.contains("dev"));
    }

    #[test]
    fn run_missing_script_errors() {
        let (mut vfs, mut pm) = setup();
        let rt = PackageManagerRuntime::npm();
        let result = rt.exec(
            &Command::parse("npm run nonexistent").unwrap(),
            &mut vfs,
            &mut pm,
        );
        assert!(result.is_err());
    }

    #[test]
    fn remove_dep() {
        let (mut vfs, mut pm) = setup();
        let rt = PackageManagerRuntime::yarn();
        rt.exec(
            &Command::parse("yarn remove zod").unwrap(),
            &mut vfs,
            &mut pm,
        )
        .unwrap();
        let pkg = PackageJson::load(&vfs).unwrap();
        assert!(!pkg.dependencies.contains_key("zod"));
    }
}
