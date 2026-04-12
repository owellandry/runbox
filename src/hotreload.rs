use crate::vfs::{ChangeKind, FileChange};
/// Hot Reload — detecta cambios en el VFS y decide la estrategia de recarga.
use serde::{Deserialize, Serialize};

// ── Estrategia de recarga ─────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ReloadAction {
    /// Inyectar nuevas hojas de estilo sin recargar la página.
    InjectCss { paths: Vec<String> },
    /// Hot Module Replacement — actualizar módulos JS/TS sin perder estado.
    Hmr { paths: Vec<String> },
    /// Recarga completa del iframe.
    FullReload,
    /// No hacer nada (archivo irrelevante para el browser).
    None,
}

/// Clasifica el tipo de archivo y su impacto en el browser.
fn classify(path: &str) -> FileImpact {
    let ext = path.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "css" | "scss" | "sass" | "less" => FileImpact::Css,
        "ts" | "tsx" | "js" | "jsx" | "mjs" => FileImpact::Script,
        "html" | "htm" | "svelte" | "vue" => FileImpact::Markup,
        "json" if path.contains("package.json") => FileImpact::Config,
        "json" if path.contains("tsconfig") => FileImpact::Config,
        "toml" | "yaml" | "yml" | "env" => FileImpact::Config,
        _ => FileImpact::Asset,
    }
}

#[derive(Debug, PartialEq)]
enum FileImpact {
    Css,
    Script,
    Markup,
    Config,
    Asset,
}

// ── Debouncer ─────────────────────────────────────────────────────────────────

/// Acumula cambios en una ventana de tiempo y los retorna en batch.
#[derive(Debug)]
pub struct Debouncer {
    pending: Vec<FileChange>,
    window_ms: u64,
    last_change_ms: u64,
}

impl Debouncer {
    pub fn new(window_ms: u64) -> Self {
        Self {
            pending: Vec::new(),
            window_ms,
            last_change_ms: 0,
        }
    }

    /// Añade un cambio. Retorna true si el batch está listo para procesar.
    pub fn push(&mut self, change: FileChange, now_ms: u64) -> bool {
        self.pending.push(change);
        self.last_change_ms = now_ms;
        false // el caller decide cuándo flush
    }

    /// Retorna true si ha pasado la ventana de debounce desde el último cambio.
    pub fn ready(&self, now_ms: u64) -> bool {
        !self.pending.is_empty() && now_ms.saturating_sub(self.last_change_ms) >= self.window_ms
    }

    /// Retorna y limpia el batch de cambios pendientes.
    pub fn flush(&mut self) -> Vec<FileChange> {
        std::mem::take(&mut self.pending)
    }
}

// ── HotReloader ───────────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct HotReloader {
    debouncer: Debouncer,
}

impl HotReloader {
    /// `debounce_ms`: cuántos ms esperar después del último cambio antes de recargar.
    pub fn new(debounce_ms: u64) -> Self {
        Self {
            debouncer: Debouncer::new(debounce_ms),
        }
    }

    /// Alimenta los cambios del VFS. Retorna la acción si el debounce expiró.
    pub fn feed(&mut self, changes: Vec<FileChange>, now_ms: u64) -> Option<ReloadAction> {
        for change in changes {
            // Ignorar eliminaciones de archivos de build
            if change.path.contains("/node_modules/")
                || change.path.contains("/.git/")
                || change.path.ends_with(".map")
            {
                continue;
            }
            self.debouncer.push(change, now_ms);
        }

        if self.debouncer.ready(now_ms) {
            let batch = self.debouncer.flush();
            Some(decide_action(&batch))
        } else {
            None
        }
    }

    /// Fuerza un flush sin esperar el debounce.
    pub fn flush_now(&mut self) -> Option<ReloadAction> {
        if self.debouncer.pending.is_empty() {
            return None;
        }
        let batch = self.debouncer.flush();
        Some(decide_action(&batch))
    }
}

/// Decide la acción mínima necesaria para el batch de cambios.
fn decide_action(changes: &[FileChange]) -> ReloadAction {
    let mut css_paths = vec![];
    let mut hmr_paths = vec![];
    let mut needs_full = false;

    for change in changes {
        // Las eliminaciones siempre requieren recarga completa
        if change.kind == ChangeKind::Deleted {
            needs_full = true;
            break;
        }
        match classify(&change.path) {
            FileImpact::Css => css_paths.push(change.path.clone()),
            FileImpact::Script => hmr_paths.push(change.path.clone()),
            FileImpact::Markup | FileImpact::Config => {
                needs_full = true;
                break;
            }
            FileImpact::Asset => {} // ignorar
        }
    }

    if needs_full {
        return ReloadAction::FullReload;
    }
    if !hmr_paths.is_empty() {
        // Si hay tanto CSS como scripts, usar HMR (más invasivo que CSS inject pero menos que full)
        let mut all = hmr_paths;
        all.extend(css_paths);
        return ReloadAction::Hmr { paths: all };
    }
    if !css_paths.is_empty() {
        return ReloadAction::InjectCss { paths: css_paths };
    }
    ReloadAction::None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn change(path: &str, kind: ChangeKind) -> FileChange {
        FileChange {
            path: path.to_string(),
            kind,
        }
    }

    #[test]
    fn css_only_injects() {
        let action = decide_action(&[change("/style.css", ChangeKind::Modified)]);
        assert!(matches!(action, ReloadAction::InjectCss { .. }));
    }

    #[test]
    fn ts_does_hmr() {
        let action = decide_action(&[change("/index.ts", ChangeKind::Modified)]);
        assert!(matches!(action, ReloadAction::Hmr { .. }));
    }

    #[test]
    fn html_does_full_reload() {
        let action = decide_action(&[change("/index.html", ChangeKind::Modified)]);
        assert_eq!(action, ReloadAction::FullReload);
    }

    #[test]
    fn deletion_does_full_reload() {
        let action = decide_action(&[change("/index.ts", ChangeKind::Deleted)]);
        assert_eq!(action, ReloadAction::FullReload);
    }

    #[test]
    fn debounce_batches_changes() {
        let mut hr = HotReloader::new(50);

        // Cambios a t=0, debounce=50ms
        let r = hr.feed(vec![change("/a.css", ChangeKind::Modified)], 0);
        assert!(r.is_none(), "no debe recargar antes del debounce");

        // A t=60 (>50ms después) debe disparar
        let r = hr.feed(vec![], 60);
        assert!(matches!(r, Some(ReloadAction::InjectCss { .. })));
    }
}
