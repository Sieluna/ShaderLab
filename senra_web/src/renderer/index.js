import { NotebookRendererManager } from './manager.js';

const activeRenderers = new Map();

export function createRenderer(containerId, notebook, options = {}) {
    const container = document.getElementById(containerId);
    if (!container) {
        console.error(`Container with id '${containerId}' not found`);
        return null;
    }

    if (!navigator.gpu) {
        console.error('WebGPU is not supported in this browser');
        container.innerHTML =
            '<div class="error-message">Your browser does not support WebGPU. Please use the latest version of Chrome or Edge.</div>';
        return null;
    }

    const manager = new NotebookRendererManager(container, notebook, options);

    manager
        .initialize()
        .then(() => {
            activeRenderers.set(containerId, manager);
        })
        .catch((error) => {
            console.error('Failed to initialize renderer:', error);
            container.innerHTML = `<div class="error-message">Renderer initialization failed: ${error.message}</div>`;
        });

    return {
        // Update rendering data
        update: (data) => manager.update(data),

        // Reset renderer
        reset: () => manager.reset(),

        // Destroy renderer and release resources
        destroy: () => {
            manager.destroy();
            activeRenderers.delete(containerId);
        },

        // Resize renderer
        resize: (width, height) => manager.resize(width, height),

        // Pause rendering
        pause: () => manager.pause(),

        // Resume rendering
        resume: () => manager.resume(),

        // Get renderer status
        getStatus: () => manager.getStatus(),
    };
}
