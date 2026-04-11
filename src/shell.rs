/// Shell emulator — parsea y despacha comandos al runtime correcto.
use crate::error::{Result, RunboxError};

#[derive(Debug, Clone)]
pub struct Command {
    pub program: String,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
}

impl Command {
    /// Parsea una línea de texto: `bun run index.ts` → Command
    pub fn parse(line: &str) -> Result<Self> {
        let mut tokens = tokenize(line);
        if tokens.is_empty() {
            return Err(RunboxError::Shell("empty command".into()));
        }

        // Variables de entorno al inicio: KEY=value cmd ...
        let mut env = vec![];
        while let Some(token) = tokens.first() {
            if let Some((k, v)) = token.split_once('=') {
                env.push((k.to_string(), v.to_string()));
                tokens.remove(0);
            } else {
                break;
            }
        }

        if tokens.is_empty() {
            return Err(RunboxError::Shell("no command after env vars".into()));
        }

        let program = tokens.remove(0);
        Ok(Self { program, args: tokens, env })
    }
}

/// Determina qué runtime debe manejar este comando.
#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeTarget {
    Bun,
    Python,
    Git,
    Curl,
    Npm,
    Pnpm,
    Yarn,
    Shell,   // builtins: cd, ls, echo, cat, ...
    Unknown,
}

impl RuntimeTarget {
    pub fn detect(cmd: &Command) -> Self {
        match cmd.program.as_str() {
            "bun" | "bunx"                              => Self::Bun,
            "python" | "python3" | "pip" | "pip3"      => Self::Python,
            "git"                                       => Self::Git,
            "curl" | "wget"                             => Self::Curl,
            "npm" | "npx"                               => Self::Npm,
            "pnpm" | "pnpx"                             => Self::Pnpm,
            "yarn"                                      => Self::Yarn,
            "cd" | "ls" | "echo" | "cat" | "pwd"
            | "mkdir" | "rm" | "cp" | "mv" | "touch"   => Self::Shell,
            _                                           => Self::Unknown,
        }
    }
}

fn tokenize(line: &str) -> Vec<String> {
    let mut tokens = vec![];
    let mut current = String::new();
    let mut in_quotes = false;
    let mut quote_char = ' ';

    for ch in line.chars() {
        match ch {
            '"' | '\'' if !in_quotes => {
                in_quotes = true;
                quote_char = ch;
            }
            c if in_quotes && c == quote_char => {
                in_quotes = false;
            }
            ' ' | '\t' if !in_quotes => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
            }
            _ => current.push(ch),
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_basic() {
        let cmd = Command::parse("bun run index.ts").unwrap();
        assert_eq!(cmd.program, "bun");
        assert_eq!(cmd.args, vec!["run", "index.ts"]);
    }

    #[test]
    fn parse_with_env() {
        let cmd = Command::parse("NODE_ENV=production bun run build.ts").unwrap();
        assert_eq!(cmd.env, vec![("NODE_ENV".to_string(), "production".to_string())]);
        assert_eq!(cmd.program, "bun");
    }

    #[test]
    fn detect_runtime() {
        let cmd = Command::parse("python3 main.py").unwrap();
        assert_eq!(RuntimeTarget::detect(&cmd), RuntimeTarget::Python);

        let cmd = Command::parse("git clone https://github.com/foo/bar").unwrap();
        assert_eq!(RuntimeTarget::detect(&cmd), RuntimeTarget::Git);
    }
}
