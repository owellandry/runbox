// AI tool definitions and provider-specific serializers.
// Compatible with OpenAI function calling, Anthropic tool use, and Gemini declarations.
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDef {
    pub name: &'static str,
    pub description: &'static str,
    pub parameters: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub arguments: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub name: String,
    pub content: Value,
    pub error: Option<String>,
}

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

pub fn to_openai_format(tools: &[ToolDef]) -> Value {
    json!(
        tools
            .iter()
            .map(|t| json!({
                "type": "function",
                "function": {
                    "name": t.name,
                    "description": t.description,
                    "parameters": t.parameters,
                }
            }))
            .collect::<Vec<_>>()
    )
}

pub fn to_anthropic_format(tools: &[ToolDef]) -> Value {
    json!(
        tools
            .iter()
            .map(|t| json!({
                "name": t.name,
                "description": t.description,
                "input_schema": t.parameters,
            }))
            .collect::<Vec<_>>()
    )
}

pub fn to_gemini_format(tools: &[ToolDef]) -> Value {
    json!([{
        "function_declarations": tools.iter().map(|t| json!({
            "name": t.name,
            "description": t.description,
            "parameters": t.parameters,
        })).collect::<Vec<_>>()
    }])
}

fn read_file() -> ToolDef {
    ToolDef {
        name: "read_file",
        description: "Read a file from the virtual project filesystem.",
        parameters: json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Absolute file path, for example /src/index.ts"
                }
            },
            "required": ["path"]
        }),
    }
}

fn write_file() -> ToolDef {
    ToolDef {
        name: "write_file",
        description: "Create or overwrite a file in the virtual project filesystem.",
        parameters: json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Absolute file path to write"
                },
                "content": {
                    "type": "string",
                    "description": "Full file content"
                }
            },
            "required": ["path", "content"]
        }),
    }
}

fn list_dir() -> ToolDef {
    ToolDef {
        name: "list_dir",
        description: "List files and directories inside a path.",
        parameters: json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Directory path. Defaults to /."
                }
            },
            "required": []
        }),
    }
}

fn exec_command() -> ToolDef {
    ToolDef {
        name: "exec_command",
        description: "Execute a shell command in the sandbox runtime (bun/node/npm/pnpm/yarn/git/python/pip).",
        parameters: json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "Command line to execute, for example npm run start"
                }
            },
            "required": ["command"]
        }),
    }
}

fn search_code() -> ToolDef {
    ToolDef {
        name: "search_code",
        description: "Search text across project files with optional directory and extension filters.",
        parameters: json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Text pattern to find"
                },
                "path": {
                    "type": "string",
                    "description": "Directory root for the search. Defaults to /."
                },
                "extension": {
                    "type": "string",
                    "description": "Optional extension filter, for example .ts or .py"
                }
            },
            "required": ["query"]
        }),
    }
}

fn get_console_logs() -> ToolDef {
    ToolDef {
        name: "get_console_logs",
        description: "Read console logs from the sandbox. Supports level and incremental filtering.",
        parameters: json!({
            "type": "object",
            "properties": {
                "level": {
                    "type": "string",
                    "enum": ["log", "info", "warn", "error", "debug"],
                    "description": "Optional log level filter"
                },
                "since_id": {
                    "type": "number",
                    "description": "Optional cursor. Return entries with id greater than this value"
                }
            },
            "required": []
        }),
    }
}

fn reload_sandbox() -> ToolDef {
    ToolDef {
        name: "reload_sandbox",
        description: "Request sandbox reload action metadata (hard or soft reload).",
        parameters: json!({
            "type": "object",
            "properties": {
                "hard": {
                    "type": "boolean",
                    "description": "true for full reload, false for soft reload signal"
                }
            },
            "required": []
        }),
    }
}

fn install_packages() -> ToolDef {
    ToolDef {
        name: "install_packages",
        description: "Install project dependencies with auto-detected or explicit package manager.",
        parameters: json!({
            "type": "object",
            "properties": {
                "packages": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Package list. Empty array means install from package.json"
                },
                "dev": {
                    "type": "boolean",
                    "description": "Install as development dependency"
                },
                "manager": {
                    "type": "string",
                    "enum": ["bun", "npm", "pnpm", "yarn"],
                    "description": "Optional package manager override"
                }
            },
            "required": []
        }),
    }
}

fn get_file_tree() -> ToolDef {
    ToolDef {
        name: "get_file_tree",
        description: "Return a recursive JSON tree of files and directories.",
        parameters: json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Root directory path. Defaults to /."
                },
                "depth": {
                    "type": "number",
                    "description": "Maximum recursion depth. Defaults to 5"
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
