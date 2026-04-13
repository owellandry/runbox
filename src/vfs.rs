use crate::error::{Result, RunboxError};
use serde::{Deserialize, Serialize};
/// Virtual Filesystem — sistema de archivos en memoria.
/// Todos los runtimes (Bun, Python, etc.) operan sobre este VFS.
/// Incluye tracking de cambios para hot-reload.
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Node {
    File(Vec<u8>),
    Dir(HashMap<String, Node>),
}

#[derive(Debug)]
pub struct Vfs {
    root: Node,
    /// Paths modificados desde el último drain_changes().
    pending_changes: Vec<FileChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub path: String,
    pub kind: ChangeKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChangeKind {
    Created,
    Modified,
    Deleted,
}

impl Vfs {
    pub fn new() -> Self {
        Self {
            root: Node::Dir(HashMap::new()),
            pending_changes: Vec::new(),
        }
    }

    /// Escribe un archivo. Crea directorios intermedios si no existen.
    pub fn write(&mut self, path: &str, content: Vec<u8>) -> Result<()> {
        let parts = split_path(path);
        let existed = get(&self.root, &parts).is_ok();
        insert(&mut self.root, &parts, content)?;
        // No emitir cambios en archivos internos de .git
        if !path.starts_with("/.git/") {
            self.pending_changes.push(FileChange {
                path: path.to_string(),
                kind: if existed {
                    ChangeKind::Modified
                } else {
                    ChangeKind::Created
                },
            });
        }
        Ok(())
    }

    /// Lee el contenido de un archivo.
    pub fn read(&self, path: &str) -> Result<&[u8]> {
        let parts = split_path(path);
        match get(&self.root, &parts)? {
            Node::File(bytes) => Ok(bytes),
            Node::Dir(_) => Err(RunboxError::Vfs(format!("{path} is a directory"))),
        }
    }

    /// Lista los entries de un directorio.
    pub fn list(&self, path: &str) -> Result<Vec<String>> {
        let parts = split_path(path);
        let node = if parts.is_empty() {
            &self.root
        } else {
            get(&self.root, &parts)?
        };
        match node {
            Node::Dir(entries) => Ok(entries.keys().cloned().collect()),
            Node::File(_) => Err(RunboxError::Vfs(format!("{path} is not a directory"))),
        }
    }

    /// Elimina un archivo o directorio.
    pub fn remove(&mut self, path: &str) -> Result<()> {
        let parts = split_path(path);
        remove(&mut self.root, &parts)?;
        if !path.starts_with("/.git/") {
            self.pending_changes.push(FileChange {
                path: path.to_string(),
                kind: ChangeKind::Deleted,
            });
        }
        Ok(())
    }

    /// Verifica si existe un path.
    pub fn exists(&self, path: &str) -> bool {
        let parts = split_path(path);
        get(&self.root, &parts).is_ok()
    }

    /// Retorna todos los cambios pendientes y limpia la cola.
    pub fn drain_changes(&mut self) -> Vec<FileChange> {
        std::mem::take(&mut self.pending_changes)
    }

    /// Retorna cambios sin limpiar la cola.
    pub fn peek_changes(&self) -> &[FileChange] {
        &self.pending_changes
    }
}

impl Default for Vfs {
    fn default() -> Self {
        Self::new()
    }
}

fn split_path(path: &str) -> Vec<String> {
    path.trim_matches('/')
        .split('/')
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}

fn get<'a>(node: &'a Node, parts: &[String]) -> Result<&'a Node> {
    if parts.is_empty() {
        return Ok(node);
    }
    match node {
        Node::Dir(entries) => {
            let child = entries
                .get(&parts[0])
                .ok_or_else(|| RunboxError::NotFound(parts[0].clone()))?;
            get(child, &parts[1..])
        }
        Node::File(_) => Err(RunboxError::Vfs("expected directory".into())),
    }
}

fn insert(node: &mut Node, parts: &[String], content: Vec<u8>) -> Result<()> {
    match node {
        Node::Dir(entries) => {
            if parts.len() == 1 {
                entries.insert(parts[0].clone(), Node::File(content));
                Ok(())
            } else {
                let child = entries
                    .entry(parts[0].clone())
                    .or_insert_with(|| Node::Dir(HashMap::new()));
                insert(child, &parts[1..], content)
            }
        }
        Node::File(_) => Err(RunboxError::Vfs("expected directory".into())),
    }
}

fn remove(node: &mut Node, parts: &[String]) -> Result<()> {
    match node {
        Node::Dir(entries) => {
            if parts.len() == 1 {
                entries
                    .remove(&parts[0])
                    .ok_or_else(|| RunboxError::NotFound(parts[0].clone()))?;
                Ok(())
            } else {
                let child = entries
                    .get_mut(&parts[0])
                    .ok_or_else(|| RunboxError::NotFound(parts[0].clone()))?;
                remove(child, &parts[1..])
            }
        }
        Node::File(_) => Err(RunboxError::Vfs("expected directory".into())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_and_read() {
        let mut vfs = Vfs::new();
        vfs.write("/src/main.ts", b"console.log('hi')".to_vec())
            .unwrap();
        assert_eq!(vfs.read("/src/main.ts").unwrap(), b"console.log('hi')");
    }

    #[test]
    fn list_dir() {
        let mut vfs = Vfs::new();
        vfs.write("/src/a.ts", vec![]).unwrap();
        vfs.write("/src/b.ts", vec![]).unwrap();
        let mut entries = vfs.list("/src").unwrap();
        entries.sort();
        assert_eq!(entries, vec!["a.ts", "b.ts"]);
    }

    #[test]
    fn not_found() {
        let vfs = Vfs::new();
        assert!(vfs.read("/nope.ts").is_err());
    }

    #[test]
    fn change_tracking() {
        let mut vfs = Vfs::new();
        vfs.write("/index.ts", b"v1".to_vec()).unwrap();
        vfs.write("/index.ts", b"v2".to_vec()).unwrap();
        vfs.write("/style.css", b"body{}".to_vec()).unwrap();

        let changes = vfs.drain_changes();
        assert_eq!(changes.len(), 3);
        assert_eq!(changes[0].kind, ChangeKind::Created);
        assert_eq!(changes[1].kind, ChangeKind::Modified);
        assert_eq!(changes[2].path, "/style.css");

        // Después del drain debe estar vacía
        assert!(vfs.drain_changes().is_empty());
    }

    #[test]
    fn git_changes_ignored() {
        let mut vfs = Vfs::new();
        vfs.write("/.git/HEAD", b"ref: refs/heads/main\n".to_vec())
            .unwrap();
        vfs.write("/src/app.ts", b"code".to_vec()).unwrap();

        let changes = vfs.drain_changes();
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].path, "/src/app.ts");
    }
}
