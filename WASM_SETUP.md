# RunBoxJS WASM Configuration Guide

## Problem

When integrating RunBoxJS with Vite 8.0+, you might encounter:

```
Pre-transform error: "ESM integration proposal for Wasm" is not supported currently.
Use plugin-wasm or other community plugins to handle this.
```

## Root Cause

RunBoxJS is compiled using `wasm-pack --target bundler`, which generates ESM-style WASM imports:

```javascript
import * as wasm from "./runbox_bg.wasm";
```

Vite 8.0+ requires explicit configuration to handle this syntax.

## Solution: Configure Vite for WASM

### Step 1: Install the WASM Plugin

```bash
npm install --save-dev vite-plugin-wasm
# or
bun add -d vite-plugin-wasm
```

### Step 2: Update Your Vite Config

**vite.config.ts / vite.config.js:**

```typescript
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'
import wasm from 'vite-plugin-wasm'

export default defineConfig({
  plugins: [wasm(), react(), tailwindcss()],
  optimizeDeps: {
    exclude: ['runboxjs'],
  },
})
```

**Critical Configuration Details:**

| Setting | Purpose |
|---------|---------|
| `wasm()` first | Must be first plugin to intercept WASM imports |
| `exclude: ['runboxjs']` | Prevents Vite from pre-bundling the WASM module |

### Step 3: Use RunBoxJS in Your Code

Once configured, import and use normally:

```typescript
import init, { RunboxInstance } from 'runboxjs';

async function setupSandbox() {
  // Initialize the WASM module
  await init();
  
  // Create sandbox instance
  const runbox = new RunboxInstance();
  
  // Use RunBox API
  runbox.write_file('/test.js', new TextEncoder().encode('console.log("hello")'));
  const result = runbox.exec('node /test.js');
  console.log(JSON.parse(result));
}

setupSandbox();
```

## Why This Configuration Is Needed

### What `wasm-pack` Generates

```bash
wasm-pack build --target bundler --release
```

This creates:
- **runbox.js** - Wrapper that imports the WASM binary
- **runbox_bg.wasm** - The compiled WebAssembly module
- **runbox_bg.js** - WASM bindings

The wrapper uses ESM WASM imports:
```javascript
// runbox.js
import * as wasm from "./runbox_bg.wasm";
```

### What Vite Needs

Vite 8.0+ requires a plugin to understand this syntax because it's part of an upcoming standard. The `vite-plugin-wasm` plugin handles:

1. **WASM import interception** - Catches `import ... from "*.wasm"`
2. **Proper bundling** - Ensures the WASM binary is copied to output
3. **Runtime initialization** - Manages WASM module loading

## Troubleshooting

### Error: "vite-plugin-wasm not found"

**Solution:** Install the package

```bash
npm install --save-dev vite-plugin-wasm
npm install runboxjs
npm install
```

### Error: "Module not found: 'runboxjs'"

**Solution:** Make sure you've installed runboxjs

```bash
npm install runboxjs
```

### Still getting WASM errors after configuration?

**Clear caches and reinstall:**

```bash
# Clear all caches
rm -rf node_modules .next dist build
rm bun.lock package-lock.json

# Reinstall
bun install
# or
npm install
```

**Restart the dev server:**

```bash
bun run dev
# or
npm run dev
```

## Alternative Bundlers

If you're not using Vite, WASM support differs:

| Bundler | WASM Support | Notes |
|---------|--------------|-------|
| **Webpack 5+** | Native ✅ | No additional config needed |
| **esbuild** | Plugin required | Use esbuild-plugin-wasm |
| **Rollup** | Plugin required | Use @wasm-tool/rollup-plugin-wasm |
| **Vite 5.x** | Different config | Uses wasm-pack plugin |
| **Create React App** | Native ✅ | Works out of the box |

## Best Practices

1. **Always exclude runboxjs from optimization**
   ```typescript
   optimizeDeps: {
     exclude: ['runboxjs'],
   }
   ```

2. **Load WASM asynchronously**
   ```typescript
   async function initApp() {
     await init(); // Critical!
     const runbox = new RunboxInstance();
   }
   ```

3. **Handle initialization errors**
   ```typescript
   try {
     await init();
   } catch (err) {
     console.error('WASM initialization failed:', err);
   }
   ```

4. **Don't call code before initialization**
   ```typescript
   // ❌ Wrong - init happens in background
   const runbox = new RunboxInstance();
   init();

   // ✅ Correct - wait for init
   await init();
   const runbox = new RunboxInstance();
   ```

## Performance Notes

- WASM module is ~450KB (gzipped: ~100KB)
- First load is ~50-100ms
- Subsequent operations are near-native speed
- Runs entirely in the browser - no network latency

## Version Support

| Vite Version | Status | Config Needed |
|--------------|--------|---------------|
| 5.x | ✅ Working | Different plugin |
| 8.0 - 8.3 | ✅ Working | `vite-plugin-wasm` |
| 9.x+ | ✅ Working | Check plugin compatibility |

## Resources

- [RunBoxJS npm Package](https://www.npmjs.com/package/runboxjs)
- [vite-plugin-wasm GitHub](https://github.com/Svenum/vite-plugin-wasm)
- [Vite WASM Documentation](https://vitejs.dev/guide/features.html#webassembly)
- [WebAssembly ESM Integration](https://github.com/WebAssembly/esm-integration)

## Need Help?

If you encounter issues:

1. Check the [RunBoxJS documentation](https://www.npmjs.com/package/runboxjs)
2. Review this WASM_SETUP.md guide
3. Check Vite's official WASM guide
4. Open an issue on [GitHub](https://github.com/owellandry/runbox)
