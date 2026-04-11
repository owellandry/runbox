/// Registry — gestiona múltiples servidores MCP conectados simultáneamente.
/// Permite descubrir y enrutar tool calls al servidor correcto.
use std::collections::HashMap;
use serde_json::Value;
use crate::error::{Result, RunboxError};
use super::client::{McpClient, McpServerConfig, ConnectionState};
use super::protocol::{McpTool, McpResource, ToolCallResult};

pub struct McpRegistry {
    clients: HashMap<String, McpClient>,
}

impl McpRegistry {
    pub fn new() -> Self {
        Self { clients: HashMap::new() }
    }

    /// Registra y conecta un servidor MCP.
    pub fn add(&mut self, config: McpServerConfig) -> Result<()> {
        let name = config.name.clone();
        let mut client = McpClient::new(config);
        client.initialize()?;
        self.clients.insert(name, client);
        Ok(())
    }

    /// Desconecta y elimina un servidor.
    pub fn remove(&mut self, name: &str) -> bool {
        self.clients.remove(name).is_some()
    }

    /// Lista todos los servidores registrados con su estado.
    pub fn list_servers(&self) -> Vec<ServerInfo> {
        self.clients.values().map(|c| ServerInfo {
            name: c.config.name.clone(),
            state: c.state.clone(),
            tool_count: c.caps.tools.len(),
            resource_count: c.caps.resources.len(),
        }).collect()
    }

    /// Agrega las tools de todos los servidores conectados.
    /// Cada tool lleva el prefijo `server_name/` para evitar colisiones.
    pub fn all_tools(&self) -> Vec<NamespacedTool> {
        self.clients.values()
            .filter(|c| c.state == ConnectionState::Connected)
            .flat_map(|c| c.caps.tools.iter().map(|t| NamespacedTool {
                server: c.config.name.clone(),
                tool: t.clone(),
                qualified_name: format!("{}/{}", c.config.name, t.name),
            }))
            .collect()
    }

    /// Agrega los resources de todos los servidores conectados.
    pub fn all_resources(&self) -> Vec<NamespacedResource> {
        self.clients.values()
            .filter(|c| c.state == ConnectionState::Connected)
            .flat_map(|c| c.caps.resources.iter().map(|r| NamespacedResource {
                server: c.config.name.clone(),
                resource: r.clone(),
            }))
            .collect()
    }

    /// Llama un tool en el servidor correcto.
    /// `qualified_name` puede ser "server_name/tool_name" o solo "tool_name"
    /// si es único entre todos los servidores.
    pub fn call_tool(&mut self, qualified_name: &str, arguments: Value) -> Result<ToolCallResult> {
        let (server_name, tool_name) = parse_qualified(qualified_name, &self.clients)?;

        let client = self.clients.get_mut(&server_name)
            .ok_or_else(|| RunboxError::Runtime(format!("server '{server_name}' not found")))?;

        client.call_tool(&tool_name, arguments)
    }

    /// Lee un resource del servidor que lo contiene (por URI).
    pub fn read_resource(&mut self, uri: &str) -> Result<String> {
        // Buscar qué servidor tiene este URI
        let server_name = self.clients.values()
            .find(|c| c.caps.resources.iter().any(|r| r.uri == uri))
            .map(|c| c.config.name.clone())
            .ok_or_else(|| RunboxError::NotFound(format!("resource '{uri}' not found in any server")))?;

        self.clients.get_mut(&server_name)
            .unwrap()
            .read_resource(uri)
    }

    pub fn is_empty(&self) -> bool {
        self.clients.is_empty()
    }
}

impl Default for McpRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct ServerInfo {
    pub name: String,
    pub state: ConnectionState,
    pub tool_count: usize,
    pub resource_count: usize,
}

#[derive(Debug, Clone)]
pub struct NamespacedTool {
    pub server: String,
    pub tool: McpTool,
    pub qualified_name: String,
}

#[derive(Debug, Clone)]
pub struct NamespacedResource {
    pub server: String,
    pub resource: McpResource,
}

fn parse_qualified(name: &str, clients: &HashMap<String, McpClient>) -> Result<(String, String)> {
    if let Some((server, tool)) = name.split_once('/') {
        return Ok((server.to_string(), tool.to_string()));
    }

    // Sin prefijo: buscar en todos los servidores
    let matches: Vec<_> = clients.values()
        .filter(|c| c.caps.tools.iter().any(|t| t.name == name))
        .map(|c| c.config.name.clone())
        .collect();

    match matches.len() {
        0 => Err(RunboxError::NotFound(format!("tool '{name}' not found in any server"))),
        1 => Ok((matches.into_iter().next().unwrap(), name.to_string())),
        _ => Err(RunboxError::Runtime(format!(
            "tool '{name}' is ambiguous — use server/tool format. Servers: {}", matches.join(", ")
        ))),
    }
}
