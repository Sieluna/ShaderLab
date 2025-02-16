import process from "node:process";
import path from "node:path";
import { defineConfig, loadEnv } from 'vite';
import postcssNesting from 'postcss-nesting';

/** @type {import('vite').UserConfig} */
export default defineConfig(({ mode }) => {
    const env = { ...process.env, ...loadEnv(mode, process.cwd()) };

    return {
        base: "/ShaderLab/",
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
        resolve: {
            alias: {
                'senra_api': path.resolve(__dirname, './pkg')
            }
        },
    }
});
