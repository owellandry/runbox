#!/usr/bin/env node
/**
 * Build script para runboxjs.
 * Uso:  node build.mjs [--bump patch|minor|major]
 *
 * 1. Corre wasm-pack build
 * 2. Restaura pkg/package.json desde pkg-template.json
 * 3. Toma la versión de Cargo.toml (y opcionalmente la bumpa)
 * 4. (Opcional) hace npm publish si se pasa --publish
 */

import { execSync } from 'child_process';
import { readFileSync, writeFileSync } from 'fs';

// ── Leer versión de Cargo.toml ────────────────────────────────────────────────
const cargo = readFileSync('Cargo.toml', 'utf8');
const versionMatch = cargo.match(/^version\s*=\s*"([^"]+)"/m);
if (!versionMatch) { console.error('No version found in Cargo.toml'); process.exit(1); }
let [major, minor, patch] = versionMatch[1].split('.').map(Number);

// ── Bump opcional ─────────────────────────────────────────────────────────────
const bumpArg = process.argv.find(a => a.startsWith('--bump='))?.split('=')[1]
             || (process.argv.includes('--bump') && process.argv[process.argv.indexOf('--bump') + 1]);

if (bumpArg === 'major') { major++; minor = 0; patch = 0; }
else if (bumpArg === 'minor') { minor++; patch = 0; }
else if (bumpArg === 'patch' || bumpArg) { patch++; }

const version = `${major}.${minor}.${patch}`;

// ── Actualizar Cargo.toml con la nueva versión ────────────────────────────────
if (bumpArg) {
  const updated = cargo.replace(/^(version\s*=\s*)"[^"]+"/, `$1"${version}"`);
  writeFileSync('Cargo.toml', updated);
  console.log(`📦 Version bumped → ${version}`);
}

// ── wasm-pack build ───────────────────────────────────────────────────────────
console.log('🔨 Building WASM...');
execSync('wasm-pack build --target bundler --release', { stdio: 'inherit' });

// ── Restaurar pkg/package.json desde template ─────────────────────────────────
const template = JSON.parse(readFileSync('pkg-template.json', 'utf8'));
template.version = version;
writeFileSync('pkg/package.json', JSON.stringify(template, null, 2) + '\n');
console.log(`✅ pkg/package.json → runboxjs@${version}`);

// ── Publish opcional ──────────────────────────────────────────────────────────
if (process.argv.includes('--publish')) {
  console.log('🚀 Publishing to npm...');
  execSync('npm publish --access public', { stdio: 'inherit', cwd: 'pkg' });
  console.log(`✅ runboxjs@${version} published`);
}
