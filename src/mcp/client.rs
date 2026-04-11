/// MCP Client — RunBox se conecta a servidores MCP externos.
/// Permite usar tools de cualquier servidor MCP (filesystem, databases, APIs, etc.)
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use crate::error::{Result, RunboxError};
use super::protocol::*;

/// Configuración de un servidor MCP externo.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    /// Nombre identificador (ej: "filesystem", "github", "postgres")
    pub name: String,
    /// Tipo de transporte
    pub transport: TransportConfig,
    /// Variables de entorno para el proceso del servidor
    #[serde(default)]
    pub env: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum TransportConfig {
    /// Lanza un proceso y comunica por stdin/stdout (el más común)
    Stdio {
        command: String,
        #[serde(default)]
        args: Vec<String>,
    },
    /// Servidor HTTP con Server-Sent Events
    Sse {
        url: String,
        #[serde(default)]
        headers: HashMap<String, String>,
    },
    /// WebSocket
    WebSocket {
        url: String,
    },
}

/// Estado de la conexión con un servidor MCP.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

/// Capacidades que un servidor MCP externo anunció.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RemoteCapabilities {
    pub tools:     Vec<McpTool>,
    pub resources: Vec<McpResource>,
    pub prompts:   Vec<McpPrompt>,
    pub server_info: Option<Implementation>,
}

/// Cliente de un servidor MCP individual.
pub struct McpClient {
    pub config: McpServerConfig,
    pub state:  ConnectionState,
    pub caps:   RemoteCapabilities,
    next_id:    i64,
}

impl McpClient {
    pub fn new(config: McpServerConfig) -> Self {
        Self {
            config,
            state:   ConnectionState::Disconnected,
            caps:    RemoteCapabilities::default(),
            next_id: 1,
        }
    }

    /// Simula el handshake initialize con el servidor remoto.
    /// En producción este método enviaría el mensaje por el transporte real.
    pub fn initialize(&mut self) -> Result<()> {
        self.state = ConnectionState::Connecting;

        let _req = self.build_request("initialize", json!({
            "protocolVersion": MCP_VERSION,
            "capabilities": { "roots": { "listChanged": false } },
            "clientInfo": { "name": "runbox", "version": env!("CARGO_PKG_VERSION") }
        }));

        // TODO: enviar _req por el transporte y parsear InitializeResult
        // Por ahora marcamos como conectado para poder registrar el servidor
        self.state = ConnectionState::Connected;
        Ok(())
    }

    /// Llama un tool del servidor remoto.
    pub fn call_tool(&mut self, name: &str, arguments: Value) -> Result<ToolCallResult> {
        if self.state != ConnectionState::Connected {
            return Err(RunboxError::Runtime(format!(
                "MCP server '{}' is not connected", self.config.name
            )));
        }

        let _req = self.build_request("tools/call", json!({
            "name": name,
            "arguments": arguments
        }));

        // TODO: enviar por transporte y deserializar respuesta
        // Stub que retorna un resultado placeholder
        Ok(ToolCallResult::ok(format!(
            "[stub] tool '{}' called on server '{}'", name, self.config.name
        )))
    }

    /// Lee un resource del servidor remoto.
    pub fn read_resource(&mut self, uri: &str) -> Result<String> {
        if self.state != ConnectionState::Connected {
            return Err(RunboxError::Runtime(format!(
                "MCP server '{}' is not connected", self.config.name
            )));
        }

        let _req = self.build_request("resources/read", json!({ "uri": uri }));
        // TODO: enviar por transporte
        Ok(format!("[stub] resource '{uri}' from server '{}'", self.config.name))
    }

    fn build_request(&mut self, method: &str, params: Value) -> RpcRequest {
        let id = self.next_id;
        self.next_id += 1;
        RpcRequest {
            jsonrpc: JSONRPC_VERSION.into(),
            id: Some(RequestId::Number(id)),
            method: method.into(),
            params,
        }
    }
}
