#!/usr/bin/env node
import { execSync } from 'child_process';
import path from 'path';
import { fileURLToPath } from 'url';

const projectPath = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const workspacePath = path.resolve(projectPath, '..');

console.log('projectPath', projectPath);
console.log('workspacePath', workspacePath);

const apiDir = path.join(workspacePath, 'senra_api');
const pkgDir = path.join(projectPath, 'pkg');

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
