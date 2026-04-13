# RunboxJS Assistant Skill Guide

This guide is for AI assistants integrating with RunboxJS runtimes.

## 1. What You Can Control

RunboxJS gives assistants direct control over:

- project files in a virtual filesystem
- command execution (`exec`)
- package management (`npm`, `pnpm`, `yarn`, `bun`)
- git workflows
- python/pip workflows
- console log retrieval
- terminal stream interaction
- AI tool dispatch (`ai_dispatch`)

## 2. Mandatory Boot Sequence

```ts
import init, { RunboxInstance } from 'runboxjs';

await init();
const runbox = new RunboxInstance();
```

Never issue runtime actions before `await init()` has completed.

## 3. Recommended Assistant Workflow

### Step 1: Inspect project state

- call `list_dir('/')`
- optionally use `ai_dispatch(get_file_tree)`
- read `package.json` when present

### Step 2: Decide action type

- file edits -> `write_file`
- command checks -> `exec('...')`
- dependency ops -> `install_packages` or explicit `npm/pnpm/yarn/bun` commands

### Step 3: Execute and verify

- parse JSON response from `exec`
- check `exit_code`
- capture `stderr` for user-visible diagnostics

### Step 4: Report precise outcomes

Always return:

- commands run
- key stdout/stderr
- files changed
- next recommended action

## 4. AI Tool Call Surface

`ai_dispatch` supports these tool names.

### `read_file`

Input:

```json
{ "path": "/src/index.ts" }
```

### `write_file`

Input:

```json
{ "path": "/src/index.ts", "content": "..." }
```

### `list_dir`

Input:

```json
{ "path": "/" }
```

### `exec_command`

Input:

```json
{ "command": "npm run start" }
```

### `search_code`

Input:

```json
{ "query": "RunboxInstance", "path": "/src", "extension": ".ts" }
```

### `get_console_logs`

Input:

```json
{ "level": "error", "since_id": 120 }
```

### `reload_sandbox`

Input:

```json
{ "hard": false }
```

### `install_packages`

Input:

```json
{ "packages": ["dayjs"], "dev": false, "manager": "npm" }
```

### `get_file_tree`

Input:

```json
{ "path": "/", "depth": 4 }
```

## 5. Behavior Constraints and Caveats

- filesystem is virtual and in-memory
- package manager behavior is runtime-simulated
- lockfiles are generated in VFS when install/add/remove operations run
- python native execution may not exist in browser builds; adapters can bridge Pyodide
- command support is broad but not equivalent to full host OS shell semantics

## 6. Reliable Patterns

### Pattern: fix build failure

1. read `package.json`
2. run `exec('npm run build')`
3. inspect `stderr`
4. modify files with `write_file`
5. rerun build

### Pattern: dependency mismatch

1. run `exec('npm install')` or `install_packages`
2. verify `/node_modules/<name>/package.json` exists
3. verify lockfile was generated
4. rerun target script

### Pattern: git workflow simulation

1. `exec('git init')`
2. `exec('git config --global user.name "Runbox Demo"')`
3. `exec('git config --global user.email "demo@runbox.dev"')`
4. `exec('git add .')`
5. `exec('git commit -m "chore: update"')`

## 7. Error Handling Standard

For every failed command (`exit_code != 0`):

- show the failing command
- include stderr summary
- propose direct correction command
- avoid silent retries without reporting

## 8. High-Signal Status Reporting

When updating a user, include:

- current task step
- commands executed
- pass/fail status
- concrete next action

## 9. Security Practices

- never request or expose real secrets in logs
- treat `git_set_token` as sensitive material
- avoid destructive file removals unless explicitly requested

## 10. Quick Example: assistant loop

```ts
const call = {
  name: 'exec_command',
  arguments: { command: 'npm run start' },
};

const result = JSON.parse(runbox.ai_dispatch(JSON.stringify(call)));
if (result.error) {
  console.error(result.error);
} else {
  console.log(result.content);
}
```
