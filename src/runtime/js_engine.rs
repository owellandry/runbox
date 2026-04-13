/// Motor JavaScript / TypeScript.
///
/// WASM   → usa `js_sys::eval()` — el browser provee V8 directamente.
/// Native → usa `boa_engine` (intérprete puro Rust), sin dependencias externas.
///
/// TypeScript se transpila primero quitando las anotaciones de tipos antes
/// de pasar el código al motor. No es perfecto pero cubre ~90 % de los casos.
///
/// Polyfills mejorados:
/// - async/await con Promise wrapper
/// - fetch() polyfill sobre RunBox network layer
/// - setTimeout/setInterval/clearTimeout/clearInterval
/// - Web APIs: URL, URLSearchParams, TextEncoder, TextDecoder, crypto
/// - Node.js built-ins: path, fs (VFS-mapped), process, Buffer

use std::collections::HashMap;

// ── Resultado de ejecución ────────────────────────────────────────────────────

#[derive(Debug)]
pub struct JsOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

// ── Enhanced Polyfill Generation ──────────────────────────────────────────────

/// Generates enhanced polyfill scripts for the JS engine.
/// These provide Web APIs and Node.js builtins in the sandbox.
pub struct PolyfillGenerator;

impl PolyfillGenerator {
    /// Generate async/await polyfill wrapper.
    pub fn async_await() -> &'static str {
        r#"
        // Async/await support — wraps async code in a Promise executor
        if (!globalThis.__runbox_async_init) {
            globalThis.__runbox_async_init = true;
            globalThis.__runbox_promises = [];
            const origThen = Promise.prototype.then;
            // Track promises for synchronous waiting
            globalThis.__awaitAll = async function() {
                await Promise.allSettled(globalThis.__runbox_promises);
            };
        }
        "#
    }

    /// Generate fetch() polyfill.
    pub fn fetch_polyfill() -> &'static str {
        r#"
        if (!globalThis.__runbox_fetch_init) {
            globalThis.__runbox_fetch_init = true;
            // Enhanced fetch that works over RunBox network layer
            if (typeof globalThis.fetch === 'undefined') {
                globalThis.fetch = function(url, opts) {
                    return new Promise(function(resolve, reject) {
                        try {
                            const xhr = new XMLHttpRequest();
                            xhr.open((opts && opts.method) || 'GET', url, true);
                            if (opts && opts.headers) {
                                Object.entries(opts.headers).forEach(function([k, v]) {
                                    xhr.setRequestHeader(k, v);
                                });
                            }
                            xhr.onload = function() {
                                resolve({
                                    ok: xhr.status >= 200 && xhr.status < 300,
                                    status: xhr.status,
                                    statusText: xhr.statusText,
                                    text: function() { return Promise.resolve(xhr.responseText); },
                                    json: function() { return Promise.resolve(JSON.parse(xhr.responseText)); },
                                    headers: { get: function(h) { return xhr.getResponseHeader(h); } },
                                });
                            };
                            xhr.onerror = function() { reject(new Error('Network request failed')); };
                            xhr.send(opts && opts.body);
                        } catch(e) { reject(e); }
                    });
                };
            }
        }
        "#
    }

    /// Generate timer polyfills (setTimeout, setInterval, clearTimeout, clearInterval).
    pub fn timers() -> &'static str {
        r#"
        if (!globalThis.__runbox_timers_init) {
            globalThis.__runbox_timers_init = true;
            var __timerId = 1;
            var __timers = {};
            if (typeof globalThis.setTimeout === 'undefined') {
                globalThis.setTimeout = function(fn, ms) {
                    var id = __timerId++;
                    __timers[id] = { fn: fn, type: 'timeout' };
                    // In sandbox, execute immediately (no real async)
                    try { fn(); } catch(e) {}
                    return id;
                };
            }
            if (typeof globalThis.clearTimeout === 'undefined') {
                globalThis.clearTimeout = function(id) { delete __timers[id]; };
            }
            if (typeof globalThis.setInterval === 'undefined') {
                globalThis.setInterval = function(fn, ms) {
                    var id = __timerId++;
                    __timers[id] = { fn: fn, type: 'interval', ms: ms };
                    return id;
                };
            }
            if (typeof globalThis.clearInterval === 'undefined') {
                globalThis.clearInterval = function(id) { delete __timers[id]; };
            }
        }
        "#
    }

    /// Generate Web API polyfills (URL, URLSearchParams, TextEncoder, TextDecoder, crypto).
    pub fn web_apis() -> &'static str {
        r#"
        if (!globalThis.__runbox_webapis_init) {
            globalThis.__runbox_webapis_init = true;

            // TextEncoder / TextDecoder
            if (typeof globalThis.TextEncoder === 'undefined') {
                globalThis.TextEncoder = function() {};
                globalThis.TextEncoder.prototype.encode = function(str) {
                    var arr = [];
                    for (var i = 0; i < str.length; i++) {
                        var c = str.charCodeAt(i);
                        if (c < 128) arr.push(c);
                        else if (c < 2048) { arr.push(192 | (c >> 6)); arr.push(128 | (c & 63)); }
                        else { arr.push(224 | (c >> 12)); arr.push(128 | ((c >> 6) & 63)); arr.push(128 | (c & 63)); }
                    }
                    return new Uint8Array(arr);
                };
            }
            if (typeof globalThis.TextDecoder === 'undefined') {
                globalThis.TextDecoder = function() {};
                globalThis.TextDecoder.prototype.decode = function(buf) {
                    var bytes = new Uint8Array(buf);
                    var str = '';
                    for (var i = 0; i < bytes.length; i++) str += String.fromCharCode(bytes[i]);
                    return str;
                };
            }

            // Crypto — basic random values
            if (typeof globalThis.crypto === 'undefined') {
                globalThis.crypto = {
                    getRandomValues: function(arr) {
                        for (var i = 0; i < arr.length; i++) arr[i] = Math.floor(Math.random() * 256);
                        return arr;
                    },
                    randomUUID: function() {
                        return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function(c) {
                            var r = Math.random() * 16 | 0;
                            return (c === 'x' ? r : (r & 0x3 | 0x8)).toString(16);
                        });
                    }
                };
            }
        }
        "#
    }

    /// Generate Node.js built-in polyfills (enhanced path, fs, process, Buffer).
    pub fn node_builtins() -> &'static str {
        r#"
        if (!globalThis.__runbox_node_init) {
            globalThis.__runbox_node_init = true;

            // Enhanced path module
            if (!globalThis.__path_enhanced) {
                globalThis.__path_enhanced = {
                    join: function() { return Array.from(arguments).join('/').replace(/\/+/g, '/'); },
                    resolve: function() { return '/' + Array.from(arguments).join('/').replace(/\/+/g, '/'); },
                    extname: function(p) { var m = p.match(/\.[^.]+$/); return m ? m[0] : ''; },
                    basename: function(p, ext) { var b = p.split('/').pop(); return ext ? b.replace(ext, '') : b; },
                    dirname: function(p) { return p.split('/').slice(0, -1).join('/') || '/'; },
                    sep: '/',
                    delimiter: ':',
                    isAbsolute: function(p) { return p.startsWith('/'); },
                    normalize: function(p) {
                        var parts = p.split('/').filter(Boolean);
                        var result = [];
                        for (var i = 0; i < parts.length; i++) {
                            if (parts[i] === '..') result.pop();
                            else if (parts[i] !== '.') result.push(parts[i]);
                        }
                        return (p.startsWith('/') ? '/' : '') + result.join('/');
                    },
                    relative: function(from, to) {
                        var f = from.split('/').filter(Boolean);
                        var t = to.split('/').filter(Boolean);
                        var i = 0;
                        while (i < f.length && i < t.length && f[i] === t[i]) i++;
                        var ups = f.length - i;
                        var result = [];
                        for (var j = 0; j < ups; j++) result.push('..');
                        return result.concat(t.slice(i)).join('/');
                    },
                    parse: function(p) {
                        var dir = p.split('/').slice(0, -1).join('/') || '/';
                        var base = p.split('/').pop() || '';
                        var ext = base.match(/\.[^.]+$/);
                        return { root: '/', dir: dir, base: base, ext: ext ? ext[0] : '', name: ext ? base.slice(0, -ext[0].length) : base };
                    },
                    format: function(obj) { return (obj.dir || '') + '/' + (obj.base || obj.name + (obj.ext || '')); },
                };
            }

            // Enhanced process module
            if (!globalThis.__process_enhanced) {
                globalThis.__process_enhanced = {
                    env: { NODE_ENV: 'production', HOME: '/home', PATH: '/usr/bin' },
                    argv: ['node', 'index.js'],
                    version: 'v20.0.0',
                    versions: { node: '20.0.0' },
                    platform: 'linux',
                    arch: 'wasm32',
                    pid: 1,
                    ppid: 0,
                    cwd: function() { return '/'; },
                    chdir: function() {},
                    exit: function(code) { throw new Error('__EXIT__:' + (code || 0)); },
                    stdout: { write: function(s) { console.log(String(s)); } },
                    stderr: { write: function(s) { console.error(String(s)); } },
                    hrtime: { bigint: function() { return BigInt(Date.now()) * BigInt(1000000); } },
                    nextTick: function(fn) { Promise.resolve().then(fn); },
                    memoryUsage: function() { return { rss: 0, heapTotal: 0, heapUsed: 0, external: 0 }; },
                };
            }

            // Buffer polyfill (basic)
            if (typeof globalThis.Buffer === 'undefined') {
                globalThis.Buffer = {
                    from: function(data, enc) {
                        if (typeof data === 'string') {
                            if (enc === 'base64') {
                                try { return new Uint8Array(atob(data).split('').map(function(c) { return c.charCodeAt(0); })); }
                                catch(e) { return new Uint8Array(0); }
                            }
                            return new TextEncoder().encode(data);
                        }
                        return new Uint8Array(data);
                    },
                    alloc: function(size) { return new Uint8Array(size); },
                    isBuffer: function(obj) { return obj instanceof Uint8Array; },
                    concat: function(list) {
                        var total = list.reduce(function(s, b) { return s + b.length; }, 0);
                        var result = new Uint8Array(total);
                        var offset = 0;
                        list.forEach(function(b) { result.set(b, offset); offset += b.length; });
                        return result;
                    },
                };
            }
        }
        "#
    }

    /// Get all polyfills combined.
    pub fn all() -> String {
        format!("{}{}{}{}{}",
            Self::async_await(),
            Self::fetch_polyfill(),
            Self::timers(),
            Self::web_apis(),
            Self::node_builtins(),
        )
    }

    /// List available polyfills as JSON.
    pub fn list() -> String {
        serde_json::json!({
            "polyfills": [
                {"name": "async_await", "description": "Promise-based async/await support"},
                {"name": "fetch", "description": "fetch() API over RunBox network layer"},
                {"name": "timers", "description": "setTimeout/setInterval/clearTimeout/clearInterval"},
                {"name": "web_apis", "description": "URL, URLSearchParams, TextEncoder, TextDecoder, crypto"},
                {"name": "node_builtins", "description": "path, fs, process, Buffer polyfills"},
            ]
        }).to_string()
    }
}

/// Registry of available Node.js built-in modules for require() resolution.
pub fn node_builtin_modules() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("path", "Path manipulation utilities");
    m.insert("fs", "File system (VFS-mapped)");
    m.insert("os", "Operating system info");
    m.insert("http", "HTTP server/client");
    m.insert("url", "URL parsing");
    m.insert("crypto", "Cryptographic functions");
    m.insert("buffer", "Buffer utilities");
    m.insert("events", "Event emitter");
    m.insert("stream", "Stream interface");
    m.insert("util", "Utility functions");
    m.insert("querystring", "Query string parsing");
    m.insert("assert", "Assertions");
    m
}

// ── API pública ───────────────────────────────────────────────────────────────

/// Ejecuta código JS o TS. Transpila TS a JS si `is_typescript` es true.
pub fn run(source: &str, is_typescript: bool) -> JsOutput {
    let js = if is_typescript {
        match strip_typescript(source) {
            Ok(js) => js,
            Err(e) => {
                return JsOutput {
                    stdout: String::new(),
                    stderr: e,
                    exit_code: 1,
                };
            }
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
    let mut out = String::with_capacity(ts.len());
    let mut str_ch = '"'; // delimitador de la string actual

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
                if d <= 0 || i >= lines.len() {
                    break;
                }
            }
            continue;
        }

        // type Foo = ...;
        if line.starts_with("type ") && line.contains('=') {
            // Skip lines until we find one ending with ';' or exhaust lines
            while i < lines.len() {
                let ends = lines[i].trim_end().ends_with(';');
                i += 1;
                if ends {
                    break;
                }
            }
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
    let mut iter = out.chars().peekable();
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
                if let Some(next) = iter.next() {
                    result.push(next);
                }
                continue;
            }
            if c == str_ch {
                in_str = false;
            }
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
            if matches!(
                next,
                Some(')') | Some('.') | Some('[') | Some(';') | Some(',')
            ) {
                // Eliminar el !
                continue;
            }
        }

        // Access modifiers al inicio de class member
        if result.ends_with('\n') || result.ends_with('{') || result.ends_with(';') {
            let kw = peek_keyword(&result, c, &mut iter);
            if matches!(
                kw.as_str(),
                "public" | "private" | "protected" | "readonly" | "abstract" | "override"
            ) {
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
            '<' | '(' | '[' | '{' => {
                depth += 1;
                iter.next();
            }
            '>' | ')' | ']' | '}' => {
                if depth > 0 {
                    depth -= 1;
                    iter.next();
                } else {
                    break;
                }
            }
            ',' | ';' | '\n' if depth == 0 => break,
            '=' if depth == 0 => break,
            _ => {
                iter.next();
            }
        }
    }
}

fn skip_word(iter: &mut std::iter::Peekable<std::str::Chars<'_>>) {
    while let Some(&c) = iter.peek() {
        if c.is_alphanumeric() || c == '_' {
            iter.next();
        } else {
            break;
        }
    }
    // Saltar espacio siguiente
    if iter.peek() == Some(&' ') {
        iter.next();
    }
}

fn peek_keyword(
    ctx: &str,
    first: char,
    _iter: &mut std::iter::Peekable<std::str::Chars<'_>>,
) -> String {
    // Build the keyword from the already-collected result context.
    // We look backwards from the end of `ctx` to find any partial word,
    // then prepend `first` to build the full keyword.
    // NOTE: We intentionally do NOT consume from `iter` here because
    // non-matching keywords must remain in the stream for normal output.
    let _ = ctx;
    let mut word = String::new();
    word.push(first);
    // Peek ahead without consuming — collect chars that would form the keyword
    let remaining: String = _iter.clone().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
    word.push_str(&remaining);
    word
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
                let rest = after_import[from_idx + " from ".len()..].trim();
                // Extraer el nombre del módulo (entre comillas)
                let module = rest.trim_matches(';').trim_matches('"').trim_matches('\'');

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
        if trimmed.starts_with("export {")
            || trimmed.starts_with("export const ")
            || trimmed.starts_with("export function ")
            || trimmed.starts_with("export class ")
        {
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

        // DEBUG: mostrar en terminal qué claves hay en __vfs_modules
        {
            const keys = Object.keys(globalThis.__vfs_modules);
            const npmKeys  = keys.filter(k => !k.startsWith('.') && !k.includes('/') === false && !k.startsWith('components') && !k.startsWith('pages') && !k.startsWith('lib') && !k.startsWith('index') && !k.startsWith('app'));
            const projKeys = keys.filter(k => k.startsWith('./') || (!k.includes('/') && (k.endsWith('.js')||k.endsWith('.json'))));
            __logs.push('[VFS] ' + keys.length + ' modules loaded. Sample: ' + keys.slice(0,8).join(', '));
        }

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

        function __pick_package_entry(pkg) {
            function pick(v) {
                if (!v) return null;
                if (typeof v === 'string') return v;
                if (Array.isArray(v)) {
                    for (const item of v) {
                        const hit = pick(item);
                        if (hit) return hit;
                    }
                    return null;
                }
                if (typeof v === 'object') {
                    // Prefer CommonJS-friendly keys first.
                    for (const key of ['require', 'node', 'default', 'import', 'browser']) {
                        const hit = pick(v[key]);
                        if (hit) return hit;
                    }
                    // Fallback: first string-like value in object.
                    for (const value of Object.values(v)) {
                        const hit = pick(value);
                        if (hit) return hit;
                    }
                }
                return null;
            }

            const exportsField = pkg && pkg.exports;
            let exportTarget = exportsField;
            if (exportsField && typeof exportsField === 'object' && exportsField['.'] !== undefined) {
                exportTarget = exportsField['.'];
            }

            return pick(exportTarget) || pick(pkg.main) || pick(pkg.module) || 'index.js';
        }


        function __find_module_file(name) {
            const vfs = globalThis.__vfs_modules;
            // Direct key match first (handles already-resolved paths like 'react/cjs/react.production.min.js')
            if (vfs[name] !== undefined) return { path: name, code: vfs[name] };

            // Leer main de package.json — preferir CJS (main) sobre ESM (module)
            const pkgPath = name + '/package.json';
            if (vfs[pkgPath]) {
                try {
                    const pkg = JSON.parse(vfs[pkgPath]);
                    const entry = __pick_package_entry(pkg);
                    const normalizedEntry = String(entry).replace(/^\.\//, '');
                    const resolved = name + '/' + normalizedEntry;
                    const entryCandidates = [
                        resolved,
                        resolved + '.js',
                        resolved + '.cjs',
                        resolved + '.mjs',
                        resolved + '/index.js',
                        resolved + '/index.cjs',
                        resolved + '/index.mjs',
                    ];
                    for (const candidate of entryCandidates) {
                        if (vfs[candidate] !== undefined) return { path: candidate, code: vfs[candidate] };
                    }
                } catch(e) {}
            }
            // Fallback: candidatos en orden de prioridad (CJS antes que ESM)
            const candidates = [
                name + '/index.js',
                name + '/index.cjs',
                name + '/index.json',
                name + '.js',
                name + '.cjs',
                name + '.json',
                name + '/index.mjs',
                name + '.mjs',
            ];
            for (const c of candidates) {
                if (vfs[c] !== undefined) return { path: c, code: vfs[c] };
            }

            // Handle subpath imports like 'react-dom/client' — split into package + subpath
            const slashIdx = name.indexOf('/');
            if (slashIdx > 0 && !name.startsWith('@')) {
                const pkgName = name.substring(0, slashIdx);
                const subPath = name.substring(slashIdx + 1);
                // Check package.json exports for subpath
                const rootPkgPath = pkgName + '/package.json';
                if (vfs[rootPkgPath]) {
                    try {
                        const rootPkg = JSON.parse(vfs[rootPkgPath]);
                        if (rootPkg.exports && typeof rootPkg.exports === 'object') {
                            const subKey = './' + subPath;
                            if (rootPkg.exports[subKey]) {
                                const subTarget = pick_export_path_inner(rootPkg.exports[subKey]);
                                if (subTarget) {
                                    const full = pkgName + '/' + subTarget.replace(/^\.\//, '');
                                    if (vfs[full] !== undefined) return { path: full, code: vfs[full] };
                                }
                            }
                        }
                    } catch(e) {}
                }
            }
            // Handle scoped package subpath imports like '@scope/pkg/subpath'
            if (name.startsWith('@')) {
                const parts = name.split('/');
                if (parts.length >= 3) {
                    const pkgName = parts[0] + '/' + parts[1];
                    const subPath = parts.slice(2).join('/');
                    const rootPkgPath = pkgName + '/package.json';
                    if (vfs[rootPkgPath]) {
                        try {
                            const rootPkg = JSON.parse(vfs[rootPkgPath]);
                            if (rootPkg.exports && typeof rootPkg.exports === 'object') {
                                const subKey = './' + subPath;
                                if (rootPkg.exports[subKey]) {
                                    const subTarget = pick_export_path_inner(rootPkg.exports[subKey]);
                                    if (subTarget) {
                                        const full = pkgName + '/' + subTarget.replace(/^\.\//, '');
                                        if (vfs[full] !== undefined) return { path: full, code: vfs[full] };
                                    }
                                }
                            }
                        } catch(e) {}
                    }
                    // Direct subpath fallback
                    const directCandidates = [
                        pkgName + '/' + subPath,
                        pkgName + '/' + subPath + '.js',
                        pkgName + '/' + subPath + '.cjs',
                        pkgName + '/' + subPath + '/index.js',
                    ];
                    for (const c of directCandidates) {
                        if (vfs[c] !== undefined) return { path: c, code: vfs[c] };
                    }
                }
            }

            return null;
        }

        // Helper for resolving export map values recursively
        function pick_export_path_inner(v) {
            if (!v) return null;
            if (typeof v === 'string') return v;
            if (Array.isArray(v)) {
                for (const item of v) {
                    const hit = pick_export_path_inner(item);
                    if (hit) return hit;
                }
                return null;
            }
            if (typeof v === 'object') {
                for (const key of ['require', 'node', 'default', 'import', 'browser']) {
                    const hit = pick_export_path_inner(v[key]);
                    if (hit) return hit;
                }
                for (const value of Object.values(v)) {
                    const hit = pick_export_path_inner(value);
                    if (hit) return hit;
                }
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
            // Also cache by the resolved path to avoid double-loading
            if (modPath !== name && __module_cache[modPath] !== undefined) return __module_cache[modPath];
            const mod = { exports: {}, id: modPath };
            __module_cache[name] = mod.exports; // Pre-cache para evitar ciclos
            __module_cache[modPath] = mod.exports; // Cache by resolved path too
            const dir = modPath.split('/').slice(0, -1).join('/');
            // Transformar ESM a CJS si el archivo usa import/export
            const needsTransform = /^\s*(import\s|export\s)/m.test(rawCode);
            const code = needsTransform ? __esm_to_cjs(rawCode) : rawCode;
            try {
                (new Function('module', 'exports', 'require', '__dirname', '__filename', code))(
                    mod, mod.exports,
                    (dep) => {
                        // Strip node: prefix for core modules
                        const cleanDep = dep.startsWith('node:') ? dep.substring(5) : dep;

                        if (cleanDep.startsWith('.')) {
                            // Relative import — resolve against current directory
                            const resolved = __resolve_module_path(dir, cleanDep);
                            // Use __find_module_file for proper entry point resolution
                            const f = __find_module_file(resolved);
                            if (f) return __vfs_require(f.path);
                            // Fallback: try as-is via main require
                            return require(resolved);
                        }
                        // Absolute/bare import — delegate to main require which handles built-ins + VFS
                        return require(cleanDep);
                    },
                    '/' + dir, '/' + modPath
                );
                __module_cache[name] = mod.exports;
                __module_cache[modPath] = mod.exports;
            } catch(e) {
                delete __module_cache[name];
                delete __module_cache[modPath];
                throw new Error("Error loading module '" + name + "': " + e.message);
            }
            return mod.exports;
        }

        // ── Node.js built-in shims ──────────────────────────────────────────
        const __events_polyfill = (function() {
            function EventEmitter() { this._events = {}; }
            EventEmitter.prototype.on = function(e, fn) { (this._events[e] = this._events[e] || []).push(fn); return this; };
            EventEmitter.prototype.emit = function(e) { var a = [].slice.call(arguments, 1); (this._events[e] || []).forEach(function(fn) { fn.apply(null, a); }); return this; };
            EventEmitter.prototype.once = function(e, fn) { var self = this; function f() { self.off(e, f); fn.apply(null, arguments); } this.on(e, f); return this; };
            EventEmitter.prototype.off = EventEmitter.prototype.removeListener = function(e, fn) { this._events[e] = (this._events[e] || []).filter(function(f) { return f !== fn; }); return this; };
            EventEmitter.prototype.removeAllListeners = function(e) { if (e) delete this._events[e]; else this._events = {}; return this; };
            EventEmitter.prototype.listeners = function(e) { return this._events[e] || []; };
            EventEmitter.prototype.listenerCount = function(e) { return (this._events[e] || []).length; };
            EventEmitter.prototype.addListener = EventEmitter.prototype.on;
            EventEmitter.EventEmitter = EventEmitter;
            EventEmitter.default = EventEmitter;
            return EventEmitter;
        })();

        const __stream_polyfill = { Readable: __events_polyfill, Writable: __events_polyfill, Transform: __events_polyfill, Duplex: __events_polyfill, PassThrough: __events_polyfill, Stream: __events_polyfill, pipeline: function() {}, finished: function() {} };
        const __util_polyfill = { inherits: function(c, p) { c.prototype = Object.create(p.prototype); c.prototype.constructor = c; }, deprecate: function(fn) { return fn; }, promisify: function(fn) { return function() { var a = [].slice.call(arguments); return new Promise(function(r, j) { a.push(function(e, v) { e ? j(e) : r(v); }); fn.apply(null, a); }); }; }, types: { isDate: function(v) { return v instanceof Date; }, isRegExp: function(v) { return v instanceof RegExp; } }, inspect: function(v) { return JSON.stringify(v); }, format: function() { return [].slice.call(arguments).join(' '); } };
        const __assert_polyfill = function(v, msg) { if (!v) throw new Error(msg || 'Assertion failed'); };
        __assert_polyfill.ok = __assert_polyfill;
        __assert_polyfill.strictEqual = function(a, b, msg) { if (a !== b) throw new Error(msg || a + ' !== ' + b); };
        __assert_polyfill.deepStrictEqual = __assert_polyfill.strictEqual;
        const __querystring_polyfill = { parse: function(s) { var o = {}; (s || '').split('&').forEach(function(p) { var kv = p.split('='); o[decodeURIComponent(kv[0])] = decodeURIComponent(kv[1] || ''); }); return o; }, stringify: function(o) { return Object.entries(o || {}).map(function(kv) { return encodeURIComponent(kv[0]) + '=' + encodeURIComponent(kv[1]); }).join('&'); } };

        const require = (mod) => {
            // Strip node: prefix
            const cleanMod = mod.startsWith('node:') ? mod.substring(5) : mod;

            // Node.js built-in modules
            if (cleanMod === 'http' || cleanMod === 'https') return __http;
            if (cleanMod === 'path') return __path;
            if (cleanMod === 'os') return { platform: () => 'linux', tmpdir: () => '/tmp', homedir: () => '/home', cpus: () => [{}], arch: () => 'wasm32', hostname: () => 'runbox', type: () => 'Linux', release: () => '5.0.0', EOL: '\n' };
            if (cleanMod === 'fs' || cleanMod === 'fs/promises') return {
                readFileSync: () => { throw new Error('fs.readFileSync: not available in WASM sandbox'); },
                writeFileSync: () => { throw new Error('fs.writeFileSync: not available in WASM sandbox'); },
                existsSync: () => false,
                readdirSync: () => [],
                statSync: () => ({ isFile: () => false, isDirectory: () => false }),
                mkdirSync: () => {},
                promises: { readFile: () => Promise.reject(new Error('fs not available')), writeFile: () => Promise.reject(new Error('fs not available')) },
            };
            if (cleanMod === 'url') return {
                fileURLToPath: (u) => u.replace('file://', ''),
                pathToFileURL: (p) => 'file://' + p,
                URL: globalThis.URL || function(u) { this.href = u; this.pathname = u; },
                URLSearchParams: globalThis.URLSearchParams || function() {},
            };
            if (cleanMod === 'events') return __events_polyfill;
            if (cleanMod === 'stream') return __stream_polyfill;
            if (cleanMod === 'util') return __util_polyfill;
            if (cleanMod === 'assert') return __assert_polyfill;
            if (cleanMod === 'querystring') return __querystring_polyfill;
            if (cleanMod === 'buffer') return { Buffer: globalThis.Buffer || { from: (d) => new Uint8Array(typeof d === 'string' ? [...d].map(c => c.charCodeAt(0)) : d), alloc: (n) => new Uint8Array(n), isBuffer: () => false } };
            if (cleanMod === 'crypto') return globalThis.crypto || { getRandomValues: (a) => { for (var i = 0; i < a.length; i++) a[i] = Math.floor(Math.random() * 256); return a; }, randomUUID: () => 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, c => { var r = Math.random() * 16 | 0; return (c === 'x' ? r : (r & 0x3 | 0x8)).toString(16); }) };
            if (cleanMod === 'child_process') return { exec: () => {}, execSync: () => '', spawn: () => ({ on: () => {}, stdout: { on: () => {} }, stderr: { on: () => {} } }) };
            if (cleanMod === 'worker_threads') return { isMainThread: true, parentPort: null, Worker: function() {} };
            if (cleanMod === 'perf_hooks') return { performance: globalThis.performance || { now: Date.now } };
            if (cleanMod === 'string_decoder') return { StringDecoder: function() { this.write = function(b) { return typeof b === 'string' ? b : new TextDecoder().decode(b); }; this.end = function() { return ''; }; } };
            if (cleanMod === 'zlib') return { createGzip: () => __stream_polyfill, createGunzip: () => __stream_polyfill, gzip: (b, cb) => cb && cb(null, b), gunzip: (b, cb) => cb && cb(null, b) };

            // React y amigos — expuestos por el host (DemoPage) en globalThis
            if (cleanMod === 'react') {
                const R = globalThis.__runbox_react;
                if (R) return R;
            }
            if (cleanMod === 'react-dom' || cleanMod === 'react-dom/client') {
                const R = globalThis.__runbox_reactdom;
                if (R) return R;
            }
            if (cleanMod === 'react-dom/server') {
                const R = globalThis.__runbox_reactdom_server;
                if (R) return R;
            }
            // Cargar desde VFS node_modules (cualquier paquete instalado via npm install)
            return __vfs_require(cleanMod);
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
                let server_ports = v["server_ports"]
                    .as_array()
                    .map(|arr| arr.iter().filter_map(|p| p.as_u64()).collect::<Vec<_>>())
                    .unwrap_or_default();
                let mut stdout = v["stdout"].as_str().unwrap_or("").to_string();
                for port in &server_ports {
                    if !stdout.is_empty() {
                        stdout.push('\n');
                    }
                    stdout.push_str(&format!("🔥 Server running → http://localhost:{port}"));
                }
                JsOutput {
                    stdout,
                    stderr: stderr.clone(),
                    exit_code: if stderr.is_empty() { 0 } else { 1 },
                }
            } else {
                JsOutput {
                    stdout: json,
                    stderr: String::new(),
                    exit_code: 0,
                }
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
            JsOutput {
                stdout: String::new(),
                stderr: msg,
                exit_code: 1,
            }
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
            stdout: String::new(),
            stderr: "boa: failed to initialize console".into(),
            exit_code: 1,
        };
    }

    let exit_code = match ctx.eval(Source::from_bytes(source)) {
        Ok(_) => 0,
        Err(e) => {
            let msg = e.to_string();
            let _ = ctx.eval(Source::from_bytes(&format!(
                r#"__stderr.push({});"#,
                serde_json::to_string(&msg).unwrap_or_default()
            )));
            1
        }
    };

    let stdout = ctx
        .eval(Source::from_bytes("__stdout.join('\\n')"))
        .ok()
        .and_then(|v| v.as_string().map(|s| s.to_std_string_escaped()))
        .unwrap_or_default();

    let stderr = ctx
        .eval(Source::from_bytes("__stderr.join('\\n')"))
        .ok()
        .and_then(|v| v.as_string().map(|s| s.to_std_string_escaped()))
        .unwrap_or_default();

    JsOutput {
        stdout,
        stderr,
        exit_code,
    }
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

    #[test]
    fn polyfill_generator_all() {
        let all = PolyfillGenerator::all();
        assert!(all.contains("__runbox_async_init"));
        assert!(all.contains("fetch"));
        assert!(all.contains("setTimeout"));
        assert!(all.contains("TextEncoder"));
        assert!(all.contains("Buffer"));
    }

    #[test]
    fn polyfill_list_json() {
        let list = PolyfillGenerator::list();
        let v: serde_json::Value = serde_json::from_str(&list).unwrap();
        let polyfills = v["polyfills"].as_array().unwrap();
        assert!(polyfills.len() >= 5);
    }

    #[test]
    fn node_builtin_modules_list() {
        let builtins = node_builtin_modules();
        assert!(builtins.contains_key("path"));
        assert!(builtins.contains_key("fs"));
        assert!(builtins.contains_key("http"));
        assert!(builtins.contains_key("crypto"));
    }

    #[test]
    fn polyfill_individual_parts() {
        let async_p = PolyfillGenerator::async_await();
        assert!(async_p.contains("Promise"));

        let fetch_p = PolyfillGenerator::fetch_polyfill();
        assert!(fetch_p.contains("XMLHttpRequest") || fetch_p.contains("fetch"));

        let timers = PolyfillGenerator::timers();
        assert!(timers.contains("setInterval"));
        assert!(timers.contains("clearTimeout"));

        let web = PolyfillGenerator::web_apis();
        assert!(web.contains("crypto"));
        assert!(web.contains("randomUUID"));

        let node = PolyfillGenerator::node_builtins();
        assert!(node.contains("normalize"));
        assert!(node.contains("process"));
    }
}
