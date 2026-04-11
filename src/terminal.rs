/// Terminal — integración con xterm.js.
/// Gestiona el buffer de salida, cola de entrada y redimensionado.
/// El lado JS (xterm.js) lee `output_drain()` y escribe con `input_push()`.
use std::collections::VecDeque;
use serde::{Deserialize, Serialize};
use crate::process::Pid;

// ── Tamaño del terminal ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TerminalSize {
    pub cols: u16,
    pub rows: u16,
}

impl Default for TerminalSize {
    fn default() -> Self {
        Self { cols: 80, rows: 24 }
    }
}

// ── Entrada del usuario ───────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct InputChunk {
    /// PID del proceso que debe recibir este input (None = proceso activo).
    pub pid:  Option<Pid>,
    pub data: String,
}

// ── Buffer de salida ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputChunk {
    /// PID que generó la salida.
    pub pid:    Pid,
    /// ANSI text tal cual — xterm.js lo renderiza directamente.
    pub data:   String,
    pub stream: Stream,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Stream { Stdout, Stderr }

// ── Terminal ──────────────────────────────────────────────────────────────────

pub struct Terminal {
    output_buf: VecDeque<OutputChunk>,
    input_buf:  VecDeque<InputChunk>,
    pub size:   TerminalSize,
    capacity:   usize,
}

impl Terminal {
    pub fn new(capacity: usize) -> Self {
        Self {
            output_buf: VecDeque::with_capacity(capacity),
            input_buf:  VecDeque::new(),
            size:       TerminalSize::default(),
            capacity,
        }
    }

    // ── Output (RunBox → xterm.js) ────────────────────────────────────────────

    pub fn write_stdout(&mut self, pid: Pid, data: impl Into<String>) {
        self.push_output(pid, data.into(), Stream::Stdout);
    }

    pub fn write_stderr(&mut self, pid: Pid, data: impl Into<String>) {
        // stderr en rojo via ANSI
        let raw = data.into();
        let colored = format!("\x1b[31m{raw}\x1b[0m");
        self.push_output(pid, colored, Stream::Stderr);
    }

    /// Ingesta stdout/stderr de un ExecOutput completo.
    pub fn ingest_output(&mut self, pid: Pid, stdout: &[u8], stderr: &[u8]) {
        if !stdout.is_empty() {
            self.write_stdout(pid, String::from_utf8_lossy(stdout));
        }
        if !stderr.is_empty() {
            self.write_stderr(pid, String::from_utf8_lossy(stderr));
        }
    }

    fn push_output(&mut self, pid: Pid, data: String, stream: Stream) {
        if self.output_buf.len() >= self.capacity {
            self.output_buf.pop_front();
        }
        self.output_buf.push_back(OutputChunk { pid, data, stream });
    }

    /// Devuelve y vacía todos los chunks de salida pendientes.
    /// xterm.js llama esto en un poll (requestAnimationFrame).
    pub fn output_drain(&mut self) -> Vec<OutputChunk> {
        self.output_buf.drain(..).collect()
    }

    pub fn output_drain_json(&mut self) -> String {
        serde_json::to_string(&self.output_drain()).unwrap_or_default()
    }

    /// Escribe el prompt inicial del shell.
    pub fn write_prompt(&mut self, cwd: &str) {
        // "user@runbox:/src $ " con color
        let prompt = format!("\x1b[32muser@runbox\x1b[0m:\x1b[34m{cwd}\x1b[0m$ ");
        self.push_output(0, prompt, Stream::Stdout);
    }

    pub fn write_banner(&mut self) {
        let banner = concat!(
            "\x1b[1;35m  RunBox\x1b[0m — sandbox de desarrollo\r\n",
            "  Runtimes: bun · node · python · git · npm · pnpm · yarn\r\n",
            "  Escribe un comando para empezar.\r\n\r\n",
        );
        self.push_output(0, banner.to_string(), Stream::Stdout);
    }

    // ── Input (xterm.js → RunBox) ─────────────────────────────────────────────

    /// xterm.js llama esto cuando el usuario escribe.
    pub fn input_push(&mut self, data: impl Into<String>, pid: Option<Pid>) {
        self.input_buf.push_back(InputChunk { pid, data: data.into() });
    }

    /// El process manager consume el input pendiente.
    pub fn input_pop(&mut self) -> Option<InputChunk> {
        self.input_buf.pop_front()
    }

    pub fn input_pending(&self) -> bool {
        !self.input_buf.is_empty()
    }

    // ── Resize ────────────────────────────────────────────────────────────────

    pub fn resize(&mut self, cols: u16, rows: u16) {
        self.size = TerminalSize { cols, rows };
    }

    pub fn size_json(&self) -> String {
        serde_json::to_string(&self.size).unwrap_or_default()
    }

    // ── Control sequences ─────────────────────────────────────────────────────

    pub fn clear(&mut self) {
        self.push_output(0, "\x1b[2J\x1b[H".to_string(), Stream::Stdout);
    }

    pub fn move_cursor(&mut self, row: u16, col: u16) {
        self.push_output(0, format!("\x1b[{row};{col}H"), Stream::Stdout);
    }
}

impl Default for Terminal {
    fn default() -> Self {
        Self::new(4096)
    }
}

// ── JavaScript glue (generado en runtime) ────────────────────────────────────
//
// El código JS que conecta RunBox WASM con xterm.js:
//
// ```js
// import { Terminal as XTerm } from 'xterm';
// import init, { RunboxInstance } from './runbox.js';
//
// const xterm = new XTerm({ cursorBlink: true, theme: { background: '#1a1b1e' } });
// xterm.open(document.getElementById('terminal'));
//
// const runbox = new RunboxInstance();
//
// // Poll de salida a 60fps
// function pollOutput() {
//   const chunks = JSON.parse(runbox.terminal_drain());
//   for (const chunk of chunks) xterm.write(chunk.data);
//   requestAnimationFrame(pollOutput);
// }
// requestAnimationFrame(pollOutput);
//
// // Input del usuario
// xterm.onData(data => runbox.terminal_input(data, null));
//
// // Resize
// const fit = new FitAddon();
// xterm.loadAddon(fit);
// fit.fit();
// new ResizeObserver(() => {
//   fit.fit();
//   runbox.terminal_resize(xterm.cols, xterm.rows);
// }).observe(document.getElementById('terminal'));
// ```
