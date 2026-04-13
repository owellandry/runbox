# RunboxJS

RunboxJS is a WebAssembly sandbox runtime that executes project workflows directly in the browser.
It includes a virtual filesystem, command runtimes (bun/node/npm/pnpm/yarn/git/python/pip), terminal streams, hot reload signals, and AI tool dispatch.

- npm package: `runboxjs`
- Rust crate source: `runbox`
- Current Rust crate version: `0.3.6`

## Why RunboxJS

- Isolated execution in browser memory (no host filesystem access)
- Virtual filesystem API for files and directories
- Shell-style command execution with structured JSON output
- Package manager simulation with lockfile generation
- Git workflow simulation (init/add/commit/branch/checkout/merge/push/pull)
- Python and pip workflow simulation (plus native fallback on non-wasm targets)
- AI tool bridge (`ai_tools` + `ai_dispatch`) for assistant orchestration
- Terminal input/output streams for xterm-like integrations

## Install

```bash
npm install runboxjs
```

## Quick Start

```ts
import init, { RunboxInstance } from 'runboxjs';

await init();

const runbox = new RunboxInstance();

runbox.write_file('/index.js', new TextEncoder().encode("console.log('hello');"));

const result = JSON.parse(runbox.exec('node /index.js'));
console.log(result.stdout);
```

## Vite (zero extra config)

`runboxjs` is built with `wasm-pack --target web`, so Vite can bundle it directly.
In most projects you only need:

```bash
npm install runboxjs
```

and:

```ts
import init, { RunboxInstance } from 'runboxjs';
```

No `vite-plugin-wasm` setup is required for standard client-side Vite apps.
See [WASM_SETUP.md](./WASM_SETUP.md) for edge cases and troubleshooting.

## Runtime Command Support

`runbox.exec(line)` routes commands by program name.

- JS runtime: `bun`, `node`, `nodejs`, `tsx`, `ts-node`
- Package managers: `npm`, `npx`, `pnpm`, `pnpx`, `yarn`, `bun install/add`
- Git: `git`
- Python: `python`, `python3`, `pip`, `pip3`
- Shell builtins: `cd`, `ls`, `echo`, `cat`, `pwd`, `mkdir`, `rm`, `cp`, `mv`, `touch`

Response shape:

```json
{
  "stdout": "...",
  "stderr": "...",
  "exit_code": 0
}
```

## API Reference

All methods belong to `RunboxInstance`.

### Core Filesystem

- `write_file(path: string, content: Uint8Array): void`
- `read_file(path: string): Uint8Array`
- `list_dir(path: string): string` (JSON array)
- `file_exists(path: string): boolean`
- `remove_file(path: string): void`

### Command and Runtime

- `exec(line: string): string` (JSON with `stdout`, `stderr`, `exit_code`)

### npm Tarball Flow (WASM)

- `npm_packages_needed(): string`
- `npm_process_tarball(name: string, version: string, bytes: Uint8Array): string`

This supports browser-side fetch/install pipelines when direct registry resolution is not available.

### Git Helpers

- `git_set_user(name: string, email: string): void`
- `git_set_token(token: string): void`

### Console Stream

- `console_push(level: string, message: string, source: string): number`
- `console_all(): string`
- `console_since(id: number): string`
- `console_clear(): void`

Valid levels: `log | info | warn | error | debug`

### Terminal Stream

- `terminal_input(data: string, pid?: number): void`
- `terminal_drain(): string`
- `terminal_resize(cols: number, rows: number): void`
- `terminal_size(): string`
- `terminal_clear(): void`

### Hot Reload Signaling

- `hot_tick(now_ms: number): string`
- `hot_flush(): string`

Returns either `null` or an action:

```json
{ "type": "inject_css|hmr|full_reload", "paths": ["/src/app.css"] }
```

### Inspector Bridge

- `inspector_activate(): void`
- `inspector_deactivate(): void`
- `inspector_is_active(): boolean`
- `inspector_set_node(node_json: string): void`
- `inspector_selected(): string`
- `inspector_overlay(): string`
- `inspector_history(limit: number): string`
- `inspector_request(target: string): string`

`target` examples:
- `point:120,240`
- `selector:.card`
- `dismiss`

### Sandbox Command/Event Bridge

- `sandbox_command(cmd_json: string): string`
- `sandbox_event(event_json: string): string`

### HTTP and Service Worker Bridge

- `http_handle_request(request_json: string): string`
- `sw_handle_request(request_json: string): string`

These are used by browser adapters to proxy localhost-like requests and SW fetch interception into the virtual runtime.

### AI Tooling

- `ai_tools(provider: string): string`
- `ai_dispatch(call_json: string): string`

Providers:
- `openai`
- `anthropic`
- `gemini`
- `raw`

## AI Skill Names

`ai_dispatch` currently supports these tool names:

- `read_file`
- `write_file`
- `list_dir`
- `exec_command`
- `search_code`
- `get_console_logs`
- `reload_sandbox`
- `install_packages`
- `get_file_tree`

For assistant integration guidance, see [skills/AGENT_SKILL.md](./skills/AGENT_SKILL.md).

## Package Manager and Lockfile Behavior

When running package manager commands, RunboxJS can generate lockfiles in VFS:

- npm -> `/package-lock.json`
- pnpm -> `/pnpm-lock.yaml`
- yarn -> `/yarn.lock`
- bun -> `/bun.lock`

If your UI does not show these files, the issue is usually UI file tree refresh logic, not runtime generation.

## Local Development

```bash
# Rust checks
cargo check
cargo test
cargo bench

# WASM package build (writes pkg/package.json from template)
node build.mjs

# bump + build
node build.mjs --bump patch
```

## Publishing

Use the scripted flow (recommended):

```bash
node build.mjs --bump patch
node build.mjs --publish
```

Detailed publishing notes: [NPM_PUBLISH.md](./NPM_PUBLISH.md)

## Troubleshooting

### "crate-type must be cdylib"

`Cargo.toml` must include:

```toml
[lib]
crate-type = ["cdylib", "rlib"]
```

### Vite WASM import errors

RunboxJS no longer requires `vite-plugin-wasm`.
If you still hit a WASM error, clear caches and ensure your app imports `runboxjs` from the published package entry.

### Missing module after package install

Confirm both:
- dependency exists in `/package.json`
- corresponding package exists under `/node_modules/<name>/package.json`

### Python message "python3 not found"

Expected in environments without native Python. In browser WASM flows, host adapters should provide Pyodide integration.

## Repository Docs

- [TECHNICAL_DOCS.md](./TECHNICAL_DOCS.md)
- [WASM_SETUP.md](./WASM_SETUP.md)
- [NPM_PUBLISH.md](./NPM_PUBLISH.md)
- [skills/AGENT_SKILL.md](./skills/AGENT_SKILL.md)

## License

MIT
