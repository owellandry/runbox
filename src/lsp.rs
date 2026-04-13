use serde::{Deserialize, Serialize};
use crate::vfs::Vfs;
use crate::error::Result;

// ── 6.1 Language Server y Soporte de Editor (Monaco) ──────────────────────────

/// Rango en un archivo fuente (0-indexed). Compatible con Mónaco/LSP.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Range {
    pub start_line: u32,
    pub start_column: u32,
    pub end_line: u32,
    pub end_column: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Information,
    Hint,
}

/// Diagnóstico de código, devuelto al editor para renderizado inline (squiggles).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub range: Range,
    pub source: Option<String>,
}

/// Sugerencia de autocompletado (IntelliSense)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionItem {
    pub label: String,
    pub detail: Option<String>,
    pub insert_text: String,
    pub kind: CompletionKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompletionKind {
    Keyword,
    Function,
    Variable,
    Class,
    Module,
    Property,
}

/// Puente de backend para la inteligencia de Monaco Editor
pub struct LspServer;

impl LspServer {
    /// Obtiene auto-completado sintético para un punto en el documento
    pub fn get_completions(_vfs: &Vfs, path: &str, _line: u32, _col: u32) -> Result<Vec<CompletionItem>> {
        let mut completions = Vec::new();
        if path.ends_with(".ts") || path.ends_with(".tsx") {
            // Simulated TS completions
            completions.push(CompletionItem {
                label: "console.log".into(),
                detail: Some("Log to standard output".into()),
                insert_text: "console.log($1);".into(),
                kind: CompletionKind::Function,
            });
            completions.push(CompletionItem {
                label: "export const".into(),
                detail: None,
                insert_text: "export const ".into(),
                kind: CompletionKind::Keyword,
            });
        }
        Ok(completions)
    }

    /// Analiza un archivo de código y emite advertencias / errores.
    pub fn get_diagnostics(vfs: &Vfs, path: &str) -> Result<Vec<Diagnostic>> {
        let mut diags = Vec::new();
        if let Ok(content) = vfs.read_string(path) {
            // Simulated static checking for basic JS/TS errors
            for (i, line) in content.lines().enumerate() {
                if line.contains("debugger;") {
                    diags.push(Diagnostic {
                        severity: DiagnosticSeverity::Warning,
                        message: "Unexpected 'debugger' statement".into(),
                        range: Range {
                            start_line: i as u32,
                            start_column: 0,
                            end_line: i as u32,
                            end_column: line.len() as u32,
                        },
                        source: Some("eslint".into()),
                    });
                }
            }
        }
        Ok(diags)
    }
}
