import { defineConfig } from 'vite';
import postcssNesting from 'postcss-nesting';

/** @type {import('vite').UserConfig} */
export default defineConfig({
    css: {
        modules: {
            localsConvention: 'camelCaseOnly',
        },
        postcss: {
            plugins: [postcssNesting],
        },
    },
});
