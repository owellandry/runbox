use crate::ai::tools::{ToolCall, ToolResult};
use crate::console::Console;
use crate::error::RunboxError;
use crate::preview::PreviewManager;
use crate::process::ProcessManager;
use crate::runtime::Runtime;
use crate::runtime::bun::BunRuntime;
use crate::runtime::git::GitRuntime;
use crate::runtime::npm::PackageManagerRuntime;
use crate::runtime::python::PythonRuntime;
use crate::runtime::shell_builtins::ShellBuiltins;
use crate::shell::{Command, RuntimeTarget};
use crate::vfs::Vfs;
/// Implementación de skills — ejecuta las tool calls que pide el AI.
use serde_json::{Value, json};

/// Ejecuta una tool call del AI y retorna el resultado.
pub fn dispatch(
    call: &ToolCall,
    vfs: &mut Vfs,
    pm: &mut ProcessManager,
    console: &mut Console,
) -> ToolResult {
    dispatch_with_preview(call, vfs, pm, console, None)
}

/// Ejecuta una tool call con acceso opcional al PreviewManager.
pub fn dispatch_with_preview(
    call: &ToolCall,
    vfs: &mut Vfs,
    pm: &mut ProcessManager,
    console: &mut Console,
    preview: Option<&mut PreviewManager>,
) -> ToolResult {
    let content = match call.name.as_str() {
        "read_file" => skill_read_file(call, vfs),
        "write_file" => skill_write_file(call, vfs),
        "list_dir" => skill_list_dir(call, vfs),
        "exec_command" => skill_exec_command(call, vfs, pm, console),
        "search_code" => skill_search_code(call, vfs),
        "get_console_logs" => skill_get_console_logs(call, console),
        "reload_sandbox" => skill_reload(call),
        "install_packages" => skill_install_packages(call, vfs, pm, console),
        "get_file_tree" => skill_file_tree(call, vfs),
        "preview_start" => skill_preview_start(call, preview),
        "preview_stop" => skill_preview_stop(preview),
        "preview_configure" => skill_preview_configure(call, preview),
        "preview_share" => skill_preview_share(preview),
        other => Err(RunboxError::Runtime(format!("unknown skill: {other}"))),
    };

    match content {
        Ok(value) => ToolResult {
            name: call.name.clone(),
            content: value,
            error: None,
        },
        Err(e) => ToolResult {
            name: call.name.clone(),
            content: json!(null),
            error: Some(e.to_string()),
        },
    }
}

// ── Skills ────────────────────────────────────────────────────────────────────

fn skill_read_file(call: &ToolCall, vfs: &Vfs) -> crate::error::Result<Value> {
    let path = str_arg(&call.arguments, "path")?;
    let bytes = vfs.read(path)?;
    let content = String::from_utf8_lossy(bytes).to_string();
    Ok(json!({ "path": path, "content": content, "size": bytes.len() }))
}

fn skill_write_file(call: &ToolCall, vfs: &mut Vfs) -> crate::error::Result<Value> {
    let path = str_arg(&call.arguments, "path")?;
    let content = str_arg(&call.arguments, "content")?;
    vfs.write(path, content.as_bytes().to_vec())?;
    Ok(json!({ "path": path, "written": content.len() }))
}

fn skill_list_dir(call: &ToolCall, vfs: &Vfs) -> crate::error::Result<Value> {
    let path = call.arguments["path"].as_str().unwrap_or("/");
    let mut entries = vfs.list(path)?;
    entries.sort();
    Ok(json!({ "path": path, "entries": entries }))
}

fn skill_exec_command(
    call: &ToolCall,
    vfs: &mut Vfs,
    pm: &mut ProcessManager,
    console: &mut Console,
) -> crate::error::Result<Value> {
    let line = str_arg(&call.arguments, "command")?;
    let cmd = Command::parse(line)?;

    let output = match RuntimeTarget::detect(&cmd) {
        RuntimeTarget::Bun => BunRuntime.exec(&cmd, vfs, pm),
        RuntimeTarget::Python => PythonRuntime.exec(&cmd, vfs, pm),
        RuntimeTarget::Git => GitRuntime.exec(&cmd, vfs, pm),
        RuntimeTarget::Shell => ShellBuiltins.exec(&cmd, vfs, pm),
        RuntimeTarget::Npm => PackageManagerRuntime::npm().exec(&cmd, vfs, pm),
        RuntimeTarget::Pnpm => PackageManagerRuntime::pnpm().exec(&cmd, vfs, pm),
        RuntimeTarget::Yarn => PackageManagerRuntime::yarn().exec(&cmd, vfs, pm),
        _ => Err(RunboxError::Shell(format!(
            "{}: command not found",
            cmd.program
        ))),
    }?;

    // Ingestar output en consola
    let pid = pm.running().last().map(|p| p.pid);
    if let Some(pid) = pid {
        console.ingest_process(pid, &output.stdout, &output.stderr);
    }

    Ok(json!({
        "stdout": String::from_utf8_lossy(&output.stdout),
        "stderr": String::from_utf8_lossy(&output.stderr),
        "exit_code": output.exit_code,
    }))
}

fn skill_search_code(call: &ToolCall, vfs: &Vfs) -> crate::error::Result<Value> {
    let query = str_arg(&call.arguments, "query")?;
    let root = call.arguments["path"].as_str().unwrap_or("/");
    let extension = call.arguments["extension"].as_str();

    let mut matches: Vec<Value> = vec![];
    search_recursive(vfs, root, query, extension, &mut matches);

    Ok(json!({ "query": query, "matches": matches, "total": matches.len() }))
}

fn search_recursive(vfs: &Vfs, path: &str, query: &str, ext: Option<&str>, out: &mut Vec<Value>) {
    let entries = match vfs.list(path) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries {
        let full_path = if path == "/" {
            format!("/{entry}")
        } else {
            format!("{path}/{entry}")
        };

        if let Ok(bytes) = vfs.read(&full_path) {
            // Es un archivo
            if let Some(ext_filter) = ext {
                if !entry.ends_with(ext_filter) {
                    continue;
                }
            }
            let text = String::from_utf8_lossy(bytes);
            for (i, line) in text.lines().enumerate() {
                if line.contains(query) {
                    out.push(json!({
                        "file": full_path,
                        "line": i + 1,
                        "text": line.trim(),
                    }));
                }
            }
        } else {
            // Es un directorio, recursión
            search_recursive(vfs, &full_path, query, ext, out);
        }
    }
}

fn skill_get_console_logs(call: &ToolCall, console: &Console) -> crate::error::Result<Value> {
    let since_id = call.arguments["since_id"].as_u64();
    let level_str = call.arguments["level"].as_str();

    let entries: Vec<_> = match (since_id, level_str) {
        (Some(id), _) => console.since(id).into_iter().cloned().collect(),
        (None, Some(l)) => {
            use crate::console::LogLevel;
            let level = match l {
                "info" => LogLevel::Info,
                "warn" => LogLevel::Warn,
                "error" => LogLevel::Error,
                "debug" => LogLevel::Debug,
                _ => LogLevel::Log,
            };
            console.by_level(&level).into_iter().cloned().collect()
        }
        _ => console.all().into_iter().cloned().collect(),
    };

    Ok(json!({ "entries": entries, "count": entries.len() }))
}

fn skill_reload(call: &ToolCall) -> crate::error::Result<Value> {
    let hard = call.arguments["hard"].as_bool().unwrap_or(false);
    Ok(json!({ "action": "reload", "hard": hard }))
}

fn skill_install_packages(
    call: &ToolCall,
    vfs: &mut Vfs,
    pm: &mut ProcessManager,
    console: &mut Console,
) -> crate::error::Result<Value> {
    let packages: Vec<String> = call.arguments["packages"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let dev = call.arguments["dev"].as_bool().unwrap_or(false);

    let manager = call.arguments["manager"]
        .as_str()
        .unwrap_or_else(|| detect_package_manager(vfs));

    let cmd_str = if packages.is_empty() {
        format!("{manager} install")
    } else {
        let dev_flag = if dev {
            match manager {
                "npm" => " --save-dev",
                "pnpm" | "yarn" => " -D",
                _ => " --dev",
            }
        } else {
            ""
        };
        format!("{manager} add{dev_flag} {}", packages.join(" "))
    };

    let cmd = Command::parse(&cmd_str)?;
    let runtime: Box<dyn Runtime> = match manager {
        "pnpm" => Box::new(PackageManagerRuntime::pnpm()),
        "yarn" => Box::new(PackageManagerRuntime::yarn()),
        "bun" => Box::new(BunRuntime),
        _ => Box::new(PackageManagerRuntime::npm()),
    };

    let output = runtime.exec(&cmd, vfs, pm)?;
    console.ingest_process(0, &output.stdout, &output.stderr);

    Ok(json!({
        "manager": manager,
        "packages": packages,
        "stdout": String::from_utf8_lossy(&output.stdout),
        "exit_code": output.exit_code,
    }))
}

fn skill_file_tree(call: &ToolCall, vfs: &Vfs) -> crate::error::Result<Value> {
    let root = call.arguments["path"].as_str().unwrap_or("/");
    let depth = call.arguments["depth"].as_u64().unwrap_or(5) as usize;
    Ok(build_tree(vfs, root, depth))
}

fn build_tree(vfs: &Vfs, path: &str, depth: usize) -> Value {
    if depth == 0 {
        return json!(null);
    }

    let entries = match vfs.list(path) {
        Ok(e) => e,
        Err(_) => return json!({ "path": path, "type": "file" }),
    };

    let children: Vec<Value> = entries
        .iter()
        .map(|name| {
            let full = if path == "/" {
                format!("/{name}")
            } else {
                format!("{path}/{name}")
            };
            if vfs.read(&full).is_ok() {
                json!({ "name": name, "path": full, "type": "file" })
            } else {
                let mut node = json!({ "name": name, "path": full, "type": "dir" });
                node["children"] = build_tree(vfs, &full, depth - 1);
                node
            }
        })
        .collect();

    json!({ "path": path, "type": "dir", "children": children })
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn str_arg<'a>(args: &'a Value, key: &str) -> crate::error::Result<&'a str> {
    args[key]
        .as_str()
        .ok_or_else(|| RunboxError::Runtime(format!("missing argument: {key}")))
}

fn detect_package_manager(vfs: &Vfs) -> &'static str {
    if vfs.exists("/bun.lockb") {
        return "bun";
    }
    if vfs.exists("/pnpm-lock.yaml") {
        return "pnpm";
    }
    if vfs.exists("/yarn.lock") {
        return "yarn";
    }
    "npm"
}

// ── Preview skills ───────────────────────────────────────────────────────────

fn skill_preview_start(
    call: &ToolCall,
    preview: Option<&mut PreviewManager>,
) -> crate::error::Result<Value> {
    let preview = preview.ok_or_else(|| {
        RunboxError::Runtime("preview not available in this context".into())
    })?;

    let mut config = crate::preview::PreviewConfig::default();

    // Apply optional overrides from arguments
    if let Some(domain) = call.arguments["domain"].as_str() {
        config.domain = Some(domain.to_string());
    }
    if let Some(port) = call.arguments["port"].as_u64() {
        config.port = port as u16;
    }
    if let Some(base) = call.arguments["base_path"].as_str() {
        config.base_path = base.to_string();
    }
    if let Some(https) = call.arguments["https"].as_bool() {
        config.https = https;
    }
    if let Some(spa) = call.arguments["spa"].as_bool() {
        config.spa = spa;
    }
    if let Some(lr) = call.arguments["live_reload"].as_bool() {
        config.live_reload = lr;
    }
    if let Some(title) = call.arguments["title"].as_str() {
        config.metadata.title = title.to_string();
    }
    if let Some(desc) = call.arguments["description"].as_str() {
        config.metadata.description = desc.to_string();
    }

    // Use current timestamp (0 as fallback — the WASM layer provides real time)
    let session = preview.start(config, 0);
    Ok(json!({
        "session_id": session.id,
        "status": "running",
        "url": session.base_url(),
        "share_url": session.share_url(),
    }))
}

fn skill_preview_stop(
    preview: Option<&mut PreviewManager>,
) -> crate::error::Result<Value> {
    let preview = preview.ok_or_else(|| {
        RunboxError::Runtime("preview not available in this context".into())
    })?;

    preview.stop()?;
    Ok(json!({ "stopped": true }))
}

fn skill_preview_configure(
    call: &ToolCall,
    preview: Option<&mut PreviewManager>,
) -> crate::error::Result<Value> {
    let preview = preview.ok_or_else(|| {
        RunboxError::Runtime("preview not available in this context".into())
    })?;

    let session = preview.current_mut().ok_or_else(|| {
        RunboxError::Runtime("no active preview session".into())
    })?;

    // Apply configuration updates
    if let Some(domain) = call.arguments["domain"].as_str() {
        session.config.domain = Some(domain.to_string());
    }
    if let Some(title) = call.arguments["title"].as_str() {
        session.config.metadata.title = title.to_string();
    }
    if let Some(desc) = call.arguments["description"].as_str() {
        session.config.metadata.description = desc.to_string();
    }
    if let Some(image) = call.arguments["image"].as_str() {
        session.config.metadata.image = image.to_string();
    }
    if let Some(favicon) = call.arguments["favicon"].as_str() {
        session.config.metadata.favicon = favicon.to_string();
    }
    if let Some(origins) = call.arguments["cors_origins"].as_array() {
        session.config.cors.allowed_origins = origins
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();
    }
    if let Some(spa) = call.arguments["spa"].as_bool() {
        session.config.spa = spa;
    }
    if let Some(lr) = call.arguments["live_reload"].as_bool() {
        session.config.live_reload = lr;
    }

    Ok(json!({
        "configured": true,
        "url": session.base_url(),
        "domain": session.config.domain,
    }))
}

fn skill_preview_share(
    preview: Option<&mut PreviewManager>,
) -> crate::error::Result<Value> {
    let preview = preview.ok_or_else(|| {
        RunboxError::Runtime("preview not available in this context".into())
    })?;

    let share_url = preview.share()?;
    let session = preview.current().ok_or_else(|| {
        RunboxError::Runtime("no active preview session".into())
    })?;

    Ok(json!({
        "share_url": share_url,
        "session_id": session.id,
        "domain": session.config.domain,
    }))
}
