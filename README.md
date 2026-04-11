# RunBoxJS

A powerful WebAssembly-powered sandbox runtime for executing JavaScript, managing files, and running commands in completely isolated environments directly in the browser.

**npm**: [@runboxjs](https://www.npmjs.com/package/runboxjs)

---

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [API Reference](#api-reference)
- [Examples](#examples)
- [Use Cases](#use-cases)
- [Troubleshooting](#troubleshooting)

---

## Features

- **🔒 Secure Sandboxing** - Execute untrusted code safely in WebAssembly
- **📦 Virtual Filesystem** - In-memory VFS with complete file system operations
- **⚡ WebAssembly Runtime** - Near-native performance with WASM
- **🔧 Process Management** - Execute commands and manage subprocesses
- **📝 Git Integration** - Built-in Git operations (clone, commit, push, pull)
- **🎯 Package Management** - Full npm, yarn, and pnpm support
- **💻 Terminal Emulation** - Full xterm.js compatible terminal
- **🔄 Hot Reload** - Real-time file change detection and reload strategies
- **🔍 DOM Inspector** - Inspect and debug DOM elements
- **🤖 AI Integration** - Built-in tools for OpenAI, Anthropic, and Gemini
- **🌐 Browser-Based** - Runs entirely in the browser, no backend needed

---

## Installation

```bash
npm install runboxjs
```

or with yarn:

```bash
yarn add runboxjs
```

or with pnpm:

```bash
pnpm add runboxjs
```

### Installation with Vite

If you're using Vite 8.0+, you need to configure WASM support:

**1. Install the WASM plugin:**

```bash
npm install --save-dev vite-plugin-wasm
```

**2. Update `vite.config.ts`:**

```typescript
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import wasm from 'vite-plugin-wasm'

export default defineConfig({
  plugins: [wasm(), react()],
  optimizeDeps: {
    exclude: ['runboxjs'],
  },
})
```

**Key points:**
- `wasm()` must be the **first plugin**
- `exclude: ['runboxjs']` prevents Vite from pre-bundling the WASM module

See [WASM_SETUP.md](./WASM_SETUP.md) for more details and troubleshooting.

---

## Quick Start

```javascript
import init, { RunboxInstance } from 'runboxjs';

// Initialize the WASM module
await init();

// Create a RunBox instance
const runbox = new RunboxInstance();

// Write a file
runbox.write_file('index.html', new TextEncoder().encode('<h1>Hello World</h1>'));

// Read a file
const content = runbox.read_file('index.html');
console.log(new TextDecoder().decode(content));

// Execute a command
const result = runbox.exec('ls');
console.log(JSON.parse(result));
// Output: { stdout: "index.html\n", stderr: "", exit_code: 0 }
```

---

## API Reference

### Creating an Instance

```javascript
const runbox = new RunboxInstance();
```

Creates a new isolated RunBox sandbox with its own VFS, process manager, console, and terminal.

---

### File System Operations

#### `write_file(path: string, content: Uint8Array): void`

Writes binary content to a file. Creates the file if it doesn't exist.

```javascript
const content = new TextEncoder().encode('Hello World');
runbox.write_file('/index.html', content);
```

---

#### `read_file(path: string): Uint8Array`

Reads binary content from a file.

```javascript
const bytes = runbox.read_file('/index.html');
const text = new TextDecoder().decode(bytes);
console.log(text);
```

---

#### `list_dir(path: string): string`

Lists all entries in a directory (returns JSON string).

```javascript
const json = runbox.list_dir('/');
const entries = JSON.parse(json);
// [ { name: "index.html", type: "file", size: 45 }, { name: "src", type: "dir", size: 0 } ]
```

---

#### `file_exists(path: string): boolean`

Checks if a file or directory exists.

```javascript
if (runbox.file_exists('/index.html')) {
  console.log('File exists');
}
```

---

#### `remove_file(path: string): void`

Deletes a file or directory (recursively).

```javascript
runbox.remove_file('/index.html');
```

---

### Command Execution

#### `exec(line: string): string`

Executes a command and returns JSON result with stdout, stderr, and exit code.

```javascript
const result = runbox.exec('npm install express');
const output = JSON.parse(result);

console.log(output.stdout);    // Standard output
console.log(output.stderr);    // Standard error
console.log(output.exit_code); // Exit code (0 = success)
```

**Supported Commands:**
- `npm`, `yarn`, `pnpm` - Package managers
- `bun` - JavaScript runtime & bundler
- `git` - Version control
- `python` - Python interpreter
- Shell builtins: `ls`, `cat`, `echo`, `pwd`, `mkdir`, `rm`, `cp`, `mv`, etc.

---

### Package Management

#### `npm_packages_needed(): string`

Returns packages from package.json that need to be installed (JSON array).

```javascript
const needed = JSON.parse(runbox.npm_packages_needed());
console.log(needed);
// [ { name: "express", version: "^4.18.0", type: "production" }, ... ]
```

---

#### `npm_process_tarball(name: string, version: string, bytes: Uint8Array): string`

Installs a package from a tarball (binary data, typically from npm registry).

```javascript
// Fetch a package tarball
const response = await fetch('https://registry.npmjs.org/lodash/-/lodash-4.17.21.tgz');
const bytes = new Uint8Array(await response.arrayBuffer());

// Install it
const result = runbox.npm_process_tarball('lodash', '4.17.21', bytes);
const output = JSON.parse(result);
console.log(output.ok); // true or false
console.log(output.error); // error message if failed
```

---

### Git Configuration

#### `git_set_token(token: string): void`

Sets a GitHub/GitLab token for authentication in git operations (clone, push, pull).

```javascript
runbox.git_set_token('ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx');
```

---

#### `git_set_user(name: string, email: string): void`

Sets git user name and email (required for commits).

```javascript
runbox.git_set_user('John Doe', 'john@example.com');
```

---

### Console & Logging

#### `console_push(level: string, message: string, source: string): number`

Adds a log entry to the RunBox console.

```javascript
runbox.console_push('info', 'App started', 'main');
runbox.console_push('error', 'Something failed', 'handler');
```

**Levels:** `"log"`, `"info"`, `"warn"`, `"error"`, `"debug"`

---

#### `console_all(): string`

Returns all console entries as JSON string.

```javascript
const json = runbox.console_all();
const entries = JSON.parse(json);
entries.forEach(e => {
  console.log(`[${e.level}] ${e.message}`);
});
```

---

#### `console_since(id: number): string`

Returns only new log entries since a given ID (useful for polling).

```javascript
const newEntries = JSON.parse(runbox.console_since(lastId));
```

---

#### `console_clear(): void`

Clears all console entries.

```javascript
runbox.console_clear();
```

---

### Terminal Operations

#### `terminal_input(data: string, pid?: number): void`

Sends input data to the terminal. If data ends with `\r` or `\n`, it executes as a command.

```javascript
// Simulate user typing "ls" and pressing Enter
runbox.terminal_input('l', undefined);
runbox.terminal_input('s', undefined);
runbox.terminal_input('\r', undefined); // Execute
```

---

#### `terminal_drain(): string`

Returns pending terminal output (JSON array) and clears the buffer.

Call regularly (e.g., in animation frame) to get updates.

```javascript
function updateTerminal() {
  const json = runbox.terminal_drain();
  const chunks = JSON.parse(json);
  chunks.forEach(chunk => {
    console.log(chunk.data);
  });
  requestAnimationFrame(updateTerminal);
}
updateTerminal();
```

---

#### `terminal_resize(cols: number, rows: number): void`

Resizes the terminal.

```javascript
runbox.terminal_resize(80, 24); // 80 columns, 24 rows
```

---

#### `terminal_size(): string`

Returns current terminal size as JSON string.

```javascript
const json = runbox.terminal_size();
const { cols, rows } = JSON.parse(json);
console.log(`Terminal: ${cols}x${rows}`);
```

---

#### `terminal_clear(): void`

Clears the terminal buffer.

```javascript
runbox.terminal_clear();
```

---

### Hot Reload

#### `hot_tick(now_ms: number): string`

Feeds file changes to the hot reload system. Call regularly with current timestamp.

Returns reload action or null.

```javascript
function animationFrame(now) {
  const json = runbox.hot_tick(now);
  const action = JSON.parse(json);
  
  if (action) {
    if (action.type === 'inject_css') {
      updateCSS(action.paths);
    } else if (action.type === 'hmr') {
      reloadModules(action.paths);
    } else if (action.type === 'full_reload') {
      location.reload();
    }
  }
  
  requestAnimationFrame(animationFrame);
}
requestAnimationFrame(animationFrame);
```

**Action object:**
```javascript
{
  type: "inject_css" | "hmr" | "full_reload",
  paths?: string[]  // Changed file paths
}
```

---

#### `hot_flush(): string`

Forces immediate hot reload flush, ignoring debounce.

```javascript
runbox.hot_flush();
```

---

### DOM Inspector

#### `inspector_activate(): void`

Activates DOM inspector mode. Browser should start capturing click and hover events.

```javascript
runbox.inspector_activate();
```

---

#### `inspector_deactivate(): void`

Deactivates DOM inspector.

```javascript
runbox.inspector_deactivate();
```

---

#### `inspector_is_active(): boolean`

Checks if the inspector is currently active.

```javascript
if (runbox.inspector_is_active()) {
  console.log('Inspector is active');
}
```

---

#### `inspector_set_node(node_json: string): void`

Sets the currently inspected DOM node (from browser/iframe).

```javascript
const nodeData = {
  tag: 'div',
  id: 'app',
  classes: ['container'],
  attributes: { role: 'main' }
};
runbox.inspector_set_node(JSON.stringify(nodeData));
```

---

#### `inspector_selected(): string`

Returns the currently selected node as JSON string.

```javascript
const json = runbox.inspector_selected();
const node = JSON.parse(json);
console.log(node.tag, node.id);
```

---

#### `inspector_overlay(): string`

Returns the overlay highlight instructions as JSON.

Used to draw visual overlay on inspected elements.

---

#### `inspector_history(limit: number): string`

Returns the last N inspected nodes as JSON array.

```javascript
const json = runbox.inspector_history(10);
const nodes = JSON.parse(json);
```

---

### AI Tools Integration

#### `ai_tools(provider: string): string`

Returns tool definitions in the format expected by the specified AI provider.

```javascript
// For Anthropic Claude
const anthropicTools = runbox.ai_tools('anthropic');

// For OpenAI
const openaiTools = runbox.ai_tools('openai');

// For Google Gemini
const geminiTools = runbox.ai_tools('gemini');

// Raw format
const rawTools = runbox.ai_tools('raw');
```

**Providers:** `"anthropic"`, `"openai"`, `"gemini"`, `"raw"`

---

#### `ai_dispatch(call_json: string): string`

Executes a tool call from an AI model.

```javascript
const toolCall = {
  name: 'execute_command',
  arguments: {
    command: 'npm install express'
  }
};

const resultJson = runbox.ai_dispatch(JSON.stringify(toolCall));
const result = JSON.parse(resultJson);

console.log(result.name);     // Tool name
console.log(result.content);  // Tool output
console.log(result.error);    // Error if any
```

---

### Sandbox Commands

#### `sandbox_command(cmd_json: string): string`

High-level command dispatcher for sandbox operations.

```javascript
// Execute command
runbox.sandbox_command(JSON.stringify({
  type: 'Exec',
  line: 'npm install'
}));

// Write file
runbox.sandbox_command(JSON.stringify({
  type: 'WriteFile',
  path: '/app/index.js',
  content: 'console.log("hello");'
}));

// Read file
runbox.sandbox_command(JSON.stringify({
  type: 'ReadFile',
  path: '/app/index.js'
}));

// List directory
runbox.sandbox_command(JSON.stringify({
  type: 'ListDir',
  path: '/'
}));

// Kill process
runbox.sandbox_command(JSON.stringify({
  type: 'Kill',
  pid: 123
}));
```

---

## Examples

### Example 1: Web IDE with Editor & Preview

```javascript
import init, { RunboxInstance } from 'runboxjs';

let runbox;

async function setupIDE() {
  await init();
  runbox = new RunboxInstance();

  // Create initial files
  runbox.write_file('/index.html', new TextEncoder().encode(`
    <!DOCTYPE html>
    <html>
    <head>
      <title>My App</title>
    </head>
    <body>
      <div id="app"></div>
      <script src="app.js"><\/script>
    </body>
    </html>
  `));

  runbox.write_file('/app.js', new TextEncoder().encode(`
    document.getElementById('app').innerHTML = '<h1>Hello World</h1>';
  `));
}

function updatePreview() {
  const html = runbox.read_file('/index.html');
  const js = runbox.read_file('/app.js');

  const htmlStr = new TextDecoder().decode(html);
  const jsStr = new TextDecoder().decode(js);

  const fullHTML = htmlStr.replace('</body>', `<script>${jsStr}</script></body>`);
  
  document.getElementById('preview').srcdoc = fullHTML;
}

setupIDE();
```

---

### Example 2: Interactive Terminal

```javascript
import init, { RunboxInstance } from 'runboxjs';

let runbox;

async function initTerminal() {
  await init();
  runbox = new RunboxInstance();

  const input = document.getElementById('terminal-input');
  const output = document.getElementById('terminal-output');

  input.addEventListener('keypress', (e) => {
    if (e.key === 'Enter') {
      const command = input.value;
      input.value = '';

      const result = runbox.exec(command);
      const parsed = JSON.parse(result);

      output.textContent += `$ ${command}\n`;
      if (parsed.stdout) output.textContent += parsed.stdout;
      if (parsed.stderr) output.textContent += `Error: ${parsed.stderr}\n`;

      output.scrollTop = output.scrollHeight;
    }
  });

  input.focus();
}

initTerminal();
```

---

### Example 3: NPM Package Manager UI

```javascript
import init, { RunboxInstance } from 'runboxjs';

let runbox;

async function initPackageManager() {
  await init();
  runbox = new RunboxInstance();

  const pkg = {
    name: "my-app",
    version: "1.0.0",
    dependencies: {}
  };

  runbox.write_file(
    '/package.json',
    new TextEncoder().encode(JSON.stringify(pkg, null, 2))
  );
}

function addPackage(name) {
  const result = runbox.exec(`npm add ${name}`);
  return JSON.parse(result);
}

function listPackages() {
  const result = runbox.exec('npm list');
  return JSON.parse(result).stdout;
}

initPackageManager();
```

---

## Use Cases

1. **Browser-Based IDE** - Full code editor with npm support and live preview
2. **Documentation Playgrounds** - Executable code examples in documentation
3. **Online Learning** - Interactive coding courses and tutorials
4. **AI Code Generation** - Generate and automatically test code with Claude/GPT
5. **Code Testing** - Validate and test user submissions
6. **Collaborative Development** - Real-time multi-user code editing

---

## Troubleshooting

### "Cannot find module 'runboxjs'"

Make sure you've installed the package:

```bash
npm install runboxjs
```

And import it correctly:

```javascript
import init, { RunboxInstance } from 'runboxjs';
```

---

### WASM initialization fails

Ensure:
- You're calling `await init()` before creating instances
- The `.wasm` file is served with `Content-Type: application/wasm`
- You're in a browser context (not Node.js)

---

### Out of memory

The sandbox has limited memory. If you're experiencing issues:
- Remove unused files with `remove_file()`
- Create new instances for separate projects
- Break work into smaller operations

---

### File operations failing

Check:
- File paths are absolute (start with `/`)
- Parent directories exist before writing files
- You're using binary data (Uint8Array) for write operations

---

### Commands not found

Some runtimes (Python, Bun) may not be available in all browsers. Always check the exit code:

```javascript
const result = runbox.exec('command');
const output = JSON.parse(result);
if (output.exit_code !== 0) {
  console.error('Command failed:', output.stderr);
}
```

---

### Git operations requiring authentication

Set credentials before git operations:

```javascript
runbox.git_set_user('Your Name', 'email@example.com');
runbox.git_set_token('your_github_token');
runbox.exec('git clone https://github.com/user/repo.git');
```

---

## Performance Tips

1. **Batch file operations** - Group multiple writes before hot reload checks
2. **Use appropriate chunk sizes** - Avoid very large file writes
3. **Monitor console size** - Clear console periodically with `console_clear()`
4. **Reuse instances** - Create instances once and reuse them
5. **Terminal polling** - Call `terminal_drain()` only when needed

---

## Browser Support

- Chrome/Edge 57+
- Firefox 52+
- Safari 14.1+
- Mobile Safari 14.5+

---

## License

MIT

---

**Made with ❤️ for developers everywhere**
