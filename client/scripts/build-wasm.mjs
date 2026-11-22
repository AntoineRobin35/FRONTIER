// Compile le moteur Rust en WebAssembly dans client/wasm-pkg.
// Trouve wasm-pack même si le terminal n'a pas encore le PATH de Rust.

import { spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import { homedir } from 'node:os';
import { delimiter, dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const clientDir = resolve(here, '..');
const crateDir = resolve(clientDir, '..', 'crates', 'wasm');
const outDir = resolve(clientDir, 'wasm-pkg');

const cargoBin = join(homedir(), '.cargo', 'bin');
const env = { ...process.env };
const pathKey = Object.keys(env).find((k) => k.toLowerCase() === 'path') ?? 'PATH';
env[pathKey] = `${cargoBin}${delimiter}${env[pathKey] ?? ''}`;

const exe = process.platform === 'win32' ? 'wasm-pack.exe' : 'wasm-pack';
const local = join(cargoBin, exe);
const wasmPack = existsSync(local) ? local : 'wasm-pack';

const result = spawnSync(
  wasmPack,
  ['build', crateDir, '--target', 'web', '--release', '--out-dir', outDir, '--out-name', 'frontier', '--no-pack'],
  { stdio: 'inherit', env },
);

if (result.error) {
  console.error("\nImpossible de lancer wasm-pack. Rust est-il installé ? (voir README)\n", result.error.message);
  process.exit(1);
}
process.exit(result.status ?? 1);
