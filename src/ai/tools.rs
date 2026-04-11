/// Definiciones de tools en formato JSON Schema.
/// Compatible con OpenAI (function calling), Anthropic (tool use),
/// Google Gemini, Mistral, Cohere, y cualquier proveedor que siga el estándar.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Descripción de un tool que el AI puede invocar.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDef {
    pub name: &'static str,
    pub description: &'static str,
    pub parameters: Value,
}

/// Llamada a tool que el AI solicita ejecutar.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub arguments: Value,
}

/// Resultado de ejecutar un tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub name: String,
    pub content: Value,
    pub error: Option<String>,
}

// ── Definiciones de todos los tools disponibles ──────────────────────────────

pub fn all_tools() -> Vec<ToolDef> {
    vec![
        read_file(),
        write_file(),
        list_dir(),
        exec_command(),
        search_code(),
        get_console_logs(),
        reload_sandbox(),
        install_packages(),
        get_file_tree(),
    ]
}

/// Serializa los tools al formato OpenAI (functions array).
pub fn to_openai_format(tools: &[ToolDef]) -> Value {
    json!(tools.iter().map(|t| json!({
        "type": "function",
        "function": {
            "name": t.name,
            "description": t.description,
            "parameters": t.parameters,
        }
    })).collect::<Vec<_>>())
}

/// Serializa los tools al formato Anthropic (tools array).
pub fn to_anthropic_format(tools: &[ToolDef]) -> Value {
    json!(tools.iter().map(|t| json!({
        "name": t.name,
        "description": t.description,
        "input_schema": t.parameters,
    })).collect::<Vec<_>>())
}

/// Serializa los tools al formato Gemini (function_declarations).
pub fn to_gemini_format(tools: &[ToolDef]) -> Value {
    json!([{
        "function_declarations": tools.iter().map(|t| json!({
            "name": t.name,
            "description": t.description,
            "parameters": t.parameters,
        })).collect::<Vec<_>>()
    }])
}

// ── Tool definitions ──────────────────────────────────────────────────────────

fn read_file() -> ToolDef {
    ToolDef {
        name: "read_file",
        description: "Lee el contenido de un archivo del proyecto.",
        parameters: json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Ruta del archivo a leer (ej: /src/index.ts)"
                }
            },
            "required": ["path"]
        }),
    }
}

fn write_file() -> ToolDef {
    ToolDef {
        name: "write_file",
        description: "Escribe o crea un archivo en el proyecto.",
        parameters: json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Ruta del archivo a escribir"
                },
                "content": {
                    "type": "string",
                    "description": "Contenido del archivo"
                }
            },
            "required": ["path", "content"]
        }),
    }
}

fn list_dir() -> ToolDef {
    ToolDef {
        name: "list_dir",
        description: "Lista los archivos y carpetas en un directorio.",
        parameters: json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Directorio a listar (default: /)"
                }
            },
            "required": []
        }),
    }
}

fn exec_command() -> ToolDef {
    ToolDef {
        name: "exec_command",
        description: "Ejecuta un comando de shell en el sandbox (bun, python, git, npm, etc.).",
        parameters: json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "Comando a ejecutar (ej: bun run index.ts)"
                }
            },
            "required": ["command"]
        }),
    }
}

fn search_code() -> ToolDef {
    ToolDef {
        name: "search_code",
        description: "Busca un patrón de texto en todos los archivos del proyecto.",
        parameters: json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Texto o patrón a buscar"
                },
                "path": {
                    "type": "string",
                    "description": "Directorio donde buscar (opcional, default: /)"
                },
                "extension": {
                    "type": "string",
                    "description": "Filtrar por extensión (ej: .ts, .py)"
                }
            },
            "required": ["query"]
        }),
    }
}

fn get_console_logs() -> ToolDef {
    ToolDef {
        name: "get_console_logs",
        description: "Obtiene los logs de la consola del sandbox.",
        parameters: json!({
            "type": "object",
            "properties": {
                "level": {
                    "type": "string",
                    "enum": ["log", "info", "warn", "error", "debug"],
                    "description": "Filtrar por nivel (opcional)"
                },
                "since_id": {
                    "type": "number",
                    "description": "Solo entradas con ID mayor a este valor"
                }
            },
            "required": []
        }),
    }
}

fn reload_sandbox() -> ToolDef {
    ToolDef {
        name: "reload_sandbox",
        description: "Recarga el sandbox (reinicia el proceso o el iframe del browser).",
        parameters: json!({
            "type": "object",
            "properties": {
                "hard": {
                    "type": "boolean",
                    "description": "true = recarga completa, false = hot reload"
                }
            },
            "required": []
        }),
    }
}

fn install_packages() -> ToolDef {
    ToolDef {
        name: "install_packages",
        description: "Instala dependencias del proyecto usando el package manager detectado (bun, npm, pnpm, yarn).",
        parameters: json!({
            "type": "object",
            "properties": {
                "packages": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Lista de paquetes a instalar (vacío = instalar package.json)"
                },
                "dev": {
                    "type": "boolean",
                    "description": "Instalar como devDependency"
                },
                "manager": {
                    "type": "string",
                    "enum": ["bun", "npm", "pnpm", "yarn"],
                    "description": "Package manager a usar (auto-detectado si no se especifica)"
                }
            },
            "required": []
        }),
    }
}

fn get_file_tree() -> ToolDef {
    ToolDef {
        name: "get_file_tree",
        description: "Obtiene el árbol completo de archivos del proyecto en formato JSON.",
        parameters: json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Directorio raíz (default: /)"
                },
                "depth": {
                    "type": "number",
                    "description": "Profundidad máxima (default: 5)"
                }
            },
            "required": []
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_tools_serialize() {
        let tools = all_tools();
        assert!(!tools.is_empty());
        let openai = to_openai_format(&tools);
        assert!(openai.is_array());
        let anthropic = to_anthropic_format(&tools);
        assert!(anthropic.is_array());
        let gemini = to_gemini_format(&tools);
        assert!(gemini.is_array());
    }
}
