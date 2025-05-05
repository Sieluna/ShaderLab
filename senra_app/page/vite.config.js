import process from "node:process";
import path from "node:path";
import { lezer } from "@lezer/generator/rollup";
import { defineConfig, loadEnv } from 'vite';
import postcssNesting from 'postcss-nesting';

/** @type {import('vite').UserConfig} */
export default defineConfig(({ mode }) => {
    const env = { ...process.env, ...loadEnv(mode, process.cwd()) };

    return {
        base: "/ShaderLab/",
        clearScreen: false,
        css: {
            modules: {
                localsConvention: 'camelCaseOnly',
            },
            postcss: {
                plugins: [postcssNesting],
            },
        },
        define: {
            __APP_ENV__: JSON.stringify(env.APP_ENV ?? "development"),
            __APP_API_URL__: JSON.stringify(env.APP_API_URL ?? "http://localhost:3000"),
        },
        plugins: [lezer()],
        resolve: {
            alias: {
                'senra_client': path.resolve(__dirname, './pkg')
            }
        },
        server: {
            port: 1420,
            strictPort: true,
            host: env.TAURI_DEV_HOST ?? false,
            hmr: env.TAURI_DEV_HOST
              ? {
                  protocol: 'ws',
                  host: env.TAURI_DEV_HOST,
                  port: 1421,
                }
              : undefined,
        },
    }
});
