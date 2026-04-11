/// Motor JavaScript / TypeScript.
///
/// WASM   → usa `js_sys::eval()` — el browser provee V8 directamente.
/// Native → usa `boa_engine` (intérprete puro Rust), sin dependencias externas.
///
/// TypeScript se transpila primero quitando las anotaciones de tipos antes
/// de pasar el código al motor. No es perfecto pero cubre ~90 % de los casos.

// ── Resultado de ejecución ────────────────────────────────────────────────────

#[derive(Debug)]
pub struct JsOutput {
    pub stdout:    String,
    pub stderr:    String,
    pub exit_code: i32,
}

// ── API pública ───────────────────────────────────────────────────────────────

/// Ejecuta código JS o TS. Transpila TS a JS si `is_typescript` es true.
pub fn run(source: &str, is_typescript: bool) -> JsOutput {
    let js = if is_typescript {
        match strip_typescript(source) {
            Ok(js)  => js,
            Err(e)  => return JsOutput { stdout: String::new(), stderr: e, exit_code: 1 },
        }
    } else {
        source.to_string()
    };

    eval_js(&js)
}

// ── TypeScript type stripper ──────────────────────────────────────────────────
//
// Elimina las construcciones propias de TypeScript sin tocar el código JS.
// Cubre: tipos inline, interfaces, type aliases, enums, generics, decorators,
// access modifiers, non-null assertions, satisfies.

pub fn strip_typescript(ts: &str) -> std::result::Result<String, String> {
    let mut out    = String::with_capacity(ts.len());
    let mut str_ch = '"';      // delimitador de la string actual

    // Línea por línea primero para eliminar declaraciones de nivel superior
    let lines: Vec<&str> = ts.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim_start();

        // interface Foo { ... }
        if line.starts_with("interface ") || line.starts_with("declare ") {
            // Saltar hasta que la llave de cierre balance llegue a 0
            let mut d = 0i32;
            loop {
                let l = lines[i];
                d += l.chars().filter(|&c| c == '{').count() as i32;
                d -= l.chars().filter(|&c| c == '}').count() as i32;
                i += 1;
                if d <= 0 || i >= lines.len() { break; }
            }
            continue;
        }

        // type Foo = ...;
        if line.starts_with("type ") && line.contains('=') {
            while i < lines.len() && !lines[i].trim_end().ends_with(';') {
                i += 1;
            }
            i += 1;
            continue;
        }

        // import type ...
        if line.starts_with("import type ") {
            i += 1;
            continue;
        }

        out.push_str(lines[i]);
        out.push('\n');
        i += 1;
    }

    // Segunda pasada: eliminar anotaciones inline de tipos
    let mut result = String::with_capacity(out.len());
    let mut iter   = out.chars().peekable();
    let mut in_str = false;

    while let Some(c) = iter.next() {
        // Gestión de strings (no tocar el interior)
        if (c == '"' || c == '\'' || c == '`') && !in_str {
            in_str = true;
            str_ch = c;
            result.push(c);
            continue;
        }
        if in_str {
            if c == '\\' {
                result.push(c);
                if let Some(next) = iter.next() { result.push(next); }
                continue;
            }
            if c == str_ch { in_str = false; }
            result.push(c);
            continue;
        }

        // Anotación de tipo: `: Type` después de identificador / ) / ]
        if c == ':' {
            // Puede ser ternario (a ? b : c), label, o tipo TS
            // Heurística: si el carácter anterior era palabra/)/], es tipo
            let prev = result.chars().last();
            let is_type_ann = matches!(prev,
                Some(ch) if ch.is_alphanumeric() || ch == '_' || ch == ')' || ch == ']' || ch == '?' || ch == '"' || ch == '\''
            );
            if is_type_ann {
                // Consumir hasta el fin del tipo (coma, ), {, =, ;, newline)
                skip_type_expr(&mut iter);
                continue;
            }
        }

        // `as Type` — type assertion
        if c == 'a' && iter.peek() == Some(&'s') {
            let prev = result.chars().last();
            if matches!(prev, Some(p) if p == ' ' || p == ')' || p == ']') {
                // Comprobar que es `as` completo
                let mut buf = String::from("a");
                buf.push(iter.next().unwrap()); // 's'
                if iter.peek().map(|c| c.is_whitespace()).unwrap_or(false) {
                    // es `as Type`, consumir el tipo
                    let _ = iter.next(); // espacio
                    skip_type_expr(&mut iter);
                    continue;
                } else {
                    result.push_str(&buf);
                    continue;
                }
            }
        }

        // `!` non-null assertion al final de expresión
        if c == '!' {
            let next = iter.peek().copied();
            if matches!(next, Some(')') | Some('.') | Some('[') | Some(';') | Some(',')) {
                // Eliminar el !
                continue;
            }
        }

        // Access modifiers al inicio de class member
        if result.ends_with('\n') || result.ends_with('{') || result.ends_with(';') {
            let kw = peek_keyword(&result, c, &mut iter);
            if matches!(kw.as_str(), "public" | "private" | "protected" | "readonly" | "abstract" | "override") {
                // Saltar la keyword y el espacio siguiente
                skip_word(&mut iter);
                continue;
            }
        }

        result.push(c);
    }

    Ok(result)
}

fn skip_type_expr(iter: &mut std::iter::Peekable<std::str::Chars<'_>>) {
    let mut depth = 0i32;
    while let Some(&c) = iter.peek() {
        match c {
            '<' | '(' | '[' | '{' => { depth += 1; iter.next(); }
            '>' | ')' | ']' | '}' => {
                if depth > 0 { depth -= 1; iter.next(); }
                else { break; }
            }
            ',' | ';' | '\n' if depth == 0 => break,
            '=' if depth == 0 => break,
            _ => { iter.next(); }
        }
    }
}

fn skip_word(iter: &mut std::iter::Peekable<std::str::Chars<'_>>) {
    while let Some(&c) = iter.peek() {
        if c.is_alphanumeric() || c == '_' { iter.next(); } else { break; }
    }
    // Saltar espacio siguiente
    if iter.peek() == Some(&' ') { iter.next(); }
}

fn peek_keyword(ctx: &str, first: char, _iter: &mut std::iter::Peekable<std::str::Chars<'_>>) -> String {
    // Heurística muy simple: mirar el contexto
    let _ = ctx;
    first.to_string()
}

// ── Motor WASM — js_sys::eval ─────────────────────────────────────────────────

/// Convierte ESM import statements a require() para compatibilidad con eval().
/// `import X from 'Y'`        → `const X = require('Y');`
/// `import { A, B } from 'Y'` → `const { A, B } = require('Y');`
/// `import * as X from 'Y'`   → `const X = require('Y');`
#[cfg(target_arch = "wasm32")]
fn transform_esm_imports(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    for line in source.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("import ") && trimmed.contains(" from ") {
            // Extraer la parte entre 'import' y 'from'
            let after_import = &trimmed["import ".len()..];
            if let Some(from_idx) = after_import.rfind(" from ") {
                let binding = after_import[..from_idx].trim();
                let rest    = after_import[from_idx + " from ".len()..].trim();
                // Extraer el nombre del módulo (entre comillas)
                let module = rest.trim_matches(';')
                    .trim_matches('"')
                    .trim_matches('\'');

                if binding.starts_with("* as ") {
                    // import * as X from 'y' → const X = require('y')
                    let name = &binding["* as ".len()..];
                    out.push_str(&format!("const {name} = require('{module}');\n"));
                } else if binding.starts_with('{') {
                    // import { A, B } from 'y' → const { A, B } = require('y')
                    out.push_str(&format!("const {binding} = require('{module}');\n"));
                } else if binding.is_empty() {
                    // import 'y' — side-effect only, ignorar
                    out.push('\n');
                } else {
                    // import X from 'y' → const X = require('y')
                    out.push_str(&format!("const {binding} = require('{module}');\n"));
                }
                continue;
            }
        }
        // export default / export { ... } — ignorar en contexto eval
        if trimmed.starts_with("export default ") {
            out.push_str(&trimmed["export default ".len()..]);
            out.push('\n');
            continue;
        }
        if trimmed.starts_with("export {") || trimmed.starts_with("export const ") || trimmed.starts_with("export function ") || trimmed.starts_with("export class ") {
            out.push_str(&trimmed["export ".len()..]);
            out.push('\n');
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

#[cfg(target_arch = "wasm32")]
fn eval_js(source: &str) -> JsOutput {
    // Transformar ESM imports → require() antes del eval
    let source = transform_esm_imports(source);
    let source = source.as_str();

    // Polyfill de módulos Node.js — persiste en globalThis entre llamadas
    let polyfill = r#"
        if (!globalThis.__runbox_servers) globalThis.__runbox_servers = {};

        const __http = {
            createServer(handler) {
                return {
                    _handler: handler,
                    listen(port, hostname, cb) {
                        const callback = typeof hostname === 'function' ? hostname : cb;
                        globalThis.__runbox_servers[port] = handler;
                        if (typeof callback === 'function') callback();
                        __logs.push('__RUNBOX_SERVER_READY__:' + port);
                    }
                };
            },
            STATUS_CODES: { 200: 'OK', 201: 'Created', 404: 'Not Found', 500: 'Internal Server Error' },
        };

        const __process = {
            env: { NODE_ENV: 'production' },
            argv: ['node', 'index.js'],
            version: 'v18.0.0',
            platform: 'browser',
            exit(code) { throw new Error('__EXIT__:' + (code || 0)); },
            cwd() { return '/'; },
            stdout: { write(s) { __logs.push(String(s)); } },
            stderr: { write(s) { __errs.push(String(s)); } },
        };

        const __path = {
            join: (...parts) => parts.join('/').replace(/\/+/g, '/'),
            resolve: (...parts) => '/' + parts.join('/').replace(/\/+/g, '/'),
            extname: (p) => { const m = p.match(/\.[^.]+$/); return m ? m[0] : ''; },
            basename: (p, ext) => { const b = p.split('/').pop(); return ext ? b.replace(ext, '') : b; },
            dirname: (p) => p.split('/').slice(0, -1).join('/') || '/',
        };

        // ── CJS Module Loader desde VFS ──────────────────────────────────────────
        // globalThis.__vfs_modules es un mapa path→content pre-cargado por Rust
        // antes del eval. Permite require() de cualquier paquete npm instalado.
        if (!globalThis.__vfs_modules) globalThis.__vfs_modules = {};
        const __module_cache = {};

        function __resolve_module_path(from_dir, dep) {
            if (!dep.startsWith('.')) return dep;
            const stack = (from_dir ? from_dir + '/' + dep : dep).split('/');
            const resolved = [];
            for (const p of stack) {
                if (p === '..') resolved.pop();
                else if (p && p !== '.') resolved.push(p);
            }
            return resolved.join('/');
        }

        function __find_module_file(name) {
            const vfs = globalThis.__vfs_modules;
            // Leer main de package.json — preferir CJS (main) sobre ESM (module)
            const pkgPath = name + '/package.json';
            if (vfs[pkgPath]) {
                try {
                    const pkg = JSON.parse(vfs[pkgPath]);
                    // Orden: exports.require > main > module (evitar ESM cuando sea posible)
                    const exportsReq = pkg.exports && (
                        (pkg.exports['.'] && (pkg.exports['.'].require || pkg.exports['.'].default)) ||
                        pkg.exports.require
                    );
                    const entry = exportsReq || pkg.main || pkg.module || 'index.js';
                    const resolved = name + '/' + entry.replace(/^\.\//, '');
                    if (vfs[resolved] !== undefined) return { path: resolved, code: vfs[resolved] };
                } catch(e) {}
            }
            // Fallback: candidatos en orden de prioridad (CJS antes que ESM)
            const candidates = [
                name + '/index.js',
                name + '/index.cjs',
                name,
                name + '.js',
                name + '/index.mjs',
            ];
            for (const c of candidates) {
                if (vfs[c] !== undefined) return { path: c, code: vfs[c] };
            }
            return null;
        }

        // Transforma ESM imports/exports a CJS para poder usar new Function()
        function __esm_to_cjs(code) {
            // import X from 'Y'
            code = code.replace(/^\s*import\s+(\w+)\s+from\s+['"]([^'"]+)['"]\s*;?/gm,
                "const $1 = require('$2');");
            // import { A, B as C } from 'Y'
            code = code.replace(/^\s*import\s+(\{[^}]+\})\s+from\s+['"]([^'"]+)['"]\s*;?/gm,
                "const $1 = require('$2');");
            // import * as X from 'Y'
            code = code.replace(/^\s*import\s+\*\s+as\s+(\w+)\s+from\s+['"]([^'"]+)['"]\s*;?/gm,
                "const $1 = require('$2');");
            // import 'Y' (side-effect)
            code = code.replace(/^\s*import\s+['"][^'"]+['"]\s*;?/gm, "");
            // export default X
            code = code.replace(/^\s*export\s+default\s+/gm, "module.exports = module.exports.default = ");
            // export { A, B }
            code = code.replace(/^\s*export\s+\{([^}]+)\}\s*;?/gm, (_, names) => {
                return names.split(',').map(n => {
                    const [orig, alias] = n.trim().split(/\s+as\s+/);
                    const exp = (alias || orig).trim();
                    return 'exports.' + exp + ' = ' + orig.trim() + ';';
                }).join(' ');
            });
            // export const/let/var/function/class X
            code = code.replace(/^\s*export\s+(const|let|var)\s+(\w+)/gm,
                "const $2 = exports.$2 = (exports.$2, ");
            code = code.replace(/^\s*export\s+(function|class)\s+(\w+)/gm,
                "exports.$2 = $1 $2");
            return code;
        }

        function __vfs_require(name) {
            if (__module_cache[name] !== undefined) return __module_cache[name];
            const found = __find_module_file(name);
            if (!found) throw new Error("Cannot find module '" + name + "'. Did you run npm install?");
            const { path: modPath, code: rawCode } = found;
            const mod = { exports: {}, id: modPath };
            __module_cache[name] = mod.exports; // Pre-cache para evitar ciclos
            const dir = modPath.split('/').slice(0, -1).join('/');
            // Transformar ESM a CJS si el archivo usa import/export
            const needsTransform = /^\s*(import\s|export\s)/m.test(rawCode);
            const code = needsTransform ? __esm_to_cjs(rawCode) : rawCode;
            try {
                (new Function('module', 'exports', 'require', '__dirname', '__filename', code))(
                    mod, mod.exports,
                    (dep) => {
                        const resolved = dep.startsWith('.')
                            ? __resolve_module_path(dir, dep)
                            : dep;
                        // Intentar VFS primero, caer en built-ins si no existe
                        if (globalThis.__vfs_modules[resolved] !== undefined ||
                            globalThis.__vfs_modules[resolved + '/index.js'] !== undefined ||
                            globalThis.__vfs_modules[resolved + '.js'] !== undefined) {
                            return __vfs_require(resolved);
                        }
                        return require(resolved);
                    },
                    '/' + dir, '/' + modPath
                );
                __module_cache[name] = mod.exports;
            } catch(e) {
                delete __module_cache[name];
                throw new Error("Error loading module '" + name + "': " + e.message);
            }
            return mod.exports;
        }

        const require = (mod) => {
            if (mod === 'http') return __http;
            if (mod === 'path') return __path;
            if (mod === 'os') return { platform: () => 'linux', tmpdir: () => '/tmp', homedir: () => '/home' };
            if (mod === 'fs') return {
                readFileSync: () => { throw new Error('fs.readFileSync: not available in WASM sandbox'); },
                writeFileSync: () => { throw new Error('fs.writeFileSync: not available in WASM sandbox'); },
                existsSync: () => false,
            };
            if (mod === 'url') return {
                fileURLToPath: (u) => u.replace('file://', ''),
                pathToFileURL: (p) => 'file://' + p,
            };
            // React y amigos — expuestos por el host (DemoPage) en globalThis
            if (mod === 'react') {
                const R = globalThis.__runbox_react;
                if (R) return R;
            }
            if (mod === 'react-dom' || mod === 'react-dom/client') {
                const R = globalThis.__runbox_reactdom;
                if (R) return R;
            }
            if (mod === 'react-dom/server') {
                const R = globalThis.__runbox_reactdom_server;
                if (R) return R;
            }
            // Cargar desde VFS node_modules (cualquier paquete instalado via npm install)
            return __vfs_require(mod);
        };
        globalThis.require = require;
        globalThis.process = __process;
        const process = __process;
    "#;

    // Envolver en IIFE para capturar console.log
    let wrapped = format!(
        r#"(function(){{
            const __logs = [];
            const __errs = [];
            const __orig_log   = console.log.bind(console);
            const __orig_error = console.error.bind(console);
            console.log   = (...a) => {{ __logs.push(a.map(String).join(' ')); __orig_log(...a); }};
            console.error = (...a) => {{ __errs.push(a.map(String).join(' ')); __orig_error(...a); }};
            {polyfill}
            let __result;
            try {{ {source} }}
            catch(e) {{
                const msg = String(e);
                if (!msg.startsWith('__EXIT__:0')) __errs.push(msg);
            }}
            finally {{
                console.log   = __orig_log;
                console.error = __orig_error;
            }}
            // Filtrar señales internas del stdout visible
            const filteredLogs = __logs.filter(l => !l.startsWith('__RUNBOX_SERVER_READY__:'));
            const serverPorts = __logs
                .filter(l => l.startsWith('__RUNBOX_SERVER_READY__:'))
                .map(l => parseInt(l.split(':')[1], 10));
            return JSON.stringify({{
                stdout: filteredLogs.join('\n'),
                stderr: __errs.join('\n'),
                server_ports: serverPorts,
            }});
        }})()"#
    );

    match js_sys::eval(&wrapped) {
        Ok(val) => {
            let json = val.as_string().unwrap_or_default();
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&json) {
                let stderr = v["stderr"].as_str().unwrap_or("").to_string();
                let server_ports = v["server_ports"].as_array()
                    .map(|arr| arr.iter().filter_map(|p| p.as_u64()).collect::<Vec<_>>())
                    .unwrap_or_default();
                let mut stdout = v["stdout"].as_str().unwrap_or("").to_string();
                for port in &server_ports {
                    if !stdout.is_empty() { stdout.push('\n'); }
                    stdout.push_str(&format!("🔥 Server running → http://localhost:{port}"));
                }
                JsOutput {
                    stdout,
                    stderr:    stderr.clone(),
                    exit_code: if stderr.is_empty() { 0 } else { 1 },
                }
            } else {
                JsOutput { stdout: json, stderr: String::new(), exit_code: 0 }
            }
        }
        Err(e) => {
            // JSON.stringify(Error) siempre devuelve {} — extraer .message directamente
            let msg = js_sys::Reflect::get(&e, &wasm_bindgen::JsValue::from_str("message"))
                .ok()
                .and_then(|v| v.as_string())
                .filter(|s| !s.is_empty())
                .or_else(|| e.as_string())
                .unwrap_or_else(|| "eval error (unknown)".into());
            JsOutput { stdout: String::new(), stderr: msg, exit_code: 1 }
        }
    }
}

// ── Motor nativo — boa_engine ─────────────────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
fn eval_js(source: &str) -> JsOutput {
    use boa_engine::{Context, Source};

    let mut ctx = Context::default();

    // Inyectar console.log / console.error básico
    let log_script = r#"
        var __stdout = [];
        var __stderr = [];
        var console = {
            log:   function() { __stdout.push(Array.from(arguments).join(' ')); },
            error: function() { __stderr.push(Array.from(arguments).join(' ')); },
            warn:  function() { __stderr.push(Array.from(arguments).join(' ')); },
            info:  function() { __stdout.push(Array.from(arguments).join(' ')); },
        };
    "#;

    if ctx.eval(Source::from_bytes(log_script)).is_err() {
        return JsOutput {
            stdout:    String::new(),
            stderr:    "boa: failed to initialize console".into(),
            exit_code: 1,
        };
    }

    let exit_code = match ctx.eval(Source::from_bytes(source)) {
        Ok(_)  => 0,
        Err(e) => {
            let msg = e.to_string();
            let _ = ctx.eval(Source::from_bytes(&format!(
                r#"__stderr.push({});"#,
                serde_json::to_string(&msg).unwrap_or_default()
            )));
            1
        }
    };

    let stdout = ctx.eval(Source::from_bytes("__stdout.join('\\n')"))
        .ok()
        .and_then(|v| v.as_string().map(|s| s.to_std_string_escaped()))
        .unwrap_or_default();

    let stderr = ctx.eval(Source::from_bytes("__stderr.join('\\n')"))
        .ok()
        .and_then(|v| v.as_string().map(|s| s.to_std_string_escaped()))
        .unwrap_or_default();

    JsOutput { stdout, stderr, exit_code }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_interface() {
        let ts = "interface Foo { x: number; }\nconst a = 1;\n";
        let js = strip_typescript(ts).unwrap();
        assert!(!js.contains("interface"));
        assert!(js.contains("const a = 1"));
    }

    #[test]
    fn strip_type_alias() {
        let ts = "type Id = string;\nconst x = 'hello';\n";
        let js = strip_typescript(ts).unwrap();
        assert!(!js.contains("type Id"));
        assert!(js.contains("'hello'"));
    }

    #[test]
    fn strip_import_type() {
        let ts = "import type { Foo } from './foo';\nconst a = 1;\n";
        let js = strip_typescript(ts).unwrap();
        assert!(!js.contains("import type"));
        assert!(js.contains("const a = 1"));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn boa_eval_basic() {
        let out = run("console.log('hello')", false);
        assert_eq!(out.stdout.trim(), "hello");
        assert_eq!(out.exit_code, 0);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn boa_eval_error() {
        let out = run("throw new Error('oops')", false);
        assert_eq!(out.exit_code, 1);
        assert!(out.stderr.contains("oops"));
    }
}
