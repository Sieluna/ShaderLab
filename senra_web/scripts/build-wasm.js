#!/usr/bin/env node
import { execSync } from 'node:child_process';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const projectPath = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const workspacePath = resolve(projectPath, '..');

const apiDir = join(workspacePath, 'senra_api');
const pkgDir = join(projectPath, 'pkg');

console.log('Building Rust WASM library...');

try {
    try {
        execSync('wasm-pack --version', { stdio: 'pipe' });
        console.log('wasm-pack detected');
    } catch (e) {
        console.log('wasm-pack not detected, installing...');
        execSync('cargo install wasm-pack', { stdio: 'inherit' });
    }

    console.log(`Compiling Rust library to WebAssembly...`);
    execSync(`cd ${apiDir} && wasm-pack build --target web --out-dir ${pkgDir}`, {
        stdio: 'inherit'
    });

    console.log('WASM build completed! ✨');
    console.log(`Output directory: ${pkgDir}`);
} catch (error) {
    console.error('Build WASM failed:', error.message);
    process.exit(1);
}
