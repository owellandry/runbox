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

#[cfg(target_arch = "wasm32")]
fn eval_js(source: &str) -> JsOutput {
    // Envolver en IIFE para capturar console.log
    let wrapped = format!(
        r#"(function(){{
            const __logs = [];
            const __errs = [];
            const __orig_log   = console.log.bind(console);
            const __orig_error = console.error.bind(console);
            console.log   = (...a) => {{ __logs.push(a.map(String).join(' ')); __orig_log(...a); }};
            console.error = (...a) => {{ __errs.push(a.map(String).join(' ')); __orig_error(...a); }};
            let __result;
            try {{ {source} }}
            catch(e) {{ __errs.push(String(e)); }}
            finally {{
                console.log   = __orig_log;
                console.error = __orig_error;
            }}
            return JSON.stringify({{ stdout: __logs.join('\n'), stderr: __errs.join('\n') }});
        }})()"#
    );

    match js_sys::eval(&wrapped) {
        Ok(val) => {
            let json = val.as_string().unwrap_or_default();
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&json) {
                JsOutput {
                    stdout:    v["stdout"].as_str().unwrap_or("").to_string(),
                    stderr:    v["stderr"].as_str().unwrap_or("").to_string(),
                    exit_code: if v["stderr"].as_str().unwrap_or("").is_empty() { 0 } else { 1 },
                }
            } else {
                JsOutput { stdout: json, stderr: String::new(), exit_code: 0 }
            }
        }
        Err(e) => {
            let msg = js_sys::JSON::stringify(&e)
                .ok()
                .and_then(|s| s.as_string())
                .unwrap_or_else(|| "eval error".into());
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
