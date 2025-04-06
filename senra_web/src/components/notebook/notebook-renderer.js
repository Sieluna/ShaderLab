import { ResourceManager } from './resource-manager.js';
import { ShaderRenderer } from './shader-renderer.js';

const activeRenderers = new Map();

class NotebookRendererManager {
    container;
    notebook;
    options;
    renderers = new Map();
    resourceManager;
    isRunning = false;
    adapter = null;
    device = null;
    status = 'created';
    resizeObserver = null;
    _animationFrame = null;

    constructor(container, notebook, options = {}) {
        this.container = container;
        this.notebook = notebook;
        this.options = options;
        this.resourceManager = new ResourceManager();
    }

    async initialize() {
        try {
            this.status = 'initializing';

            await this._initWebGPU();
            await this._loadResources();
            this._createRenderers();
            this._setupResizeObserver();

            this.isRunning = true;
            this.status = 'running';
            this._startRenderLoop();

            return true;
        } catch (error) {
            this.status = 'error';
            console.error('Renderer initialization failed:', error);
            throw error;
        }
    }

    async _initWebGPU() {
        this.adapter = await navigator.gpu.requestAdapter({
            powerPreference: this.options.powerPreference ?? 'high-performance',
        });

        if (!this.adapter) {
            throw new Error('WebGPU adapter not available');
        }

        this.device = await this.adapter.requestDevice();
        this.device.lost.then((info) => {
            console.error('WebGPU device lost:', info);
            this.status = 'device-lost';
            if (this.isRunning) {
                this.initialize().catch(console.error);
            }
        });
    }

    async _loadResources() {
        if (!this.notebook.resources || !this.notebook.shaders) {
            console.warn('Notebook has no resources or shaders');
            return;
        }

        for (const resource of this.notebook.resources) {
            await this.resourceManager.loadResource(resource, this.device);
        }

        for (const shader of this.notebook.shaders) {
            await this.resourceManager.loadShader(shader, this.device);
        }
    }

    _createRenderers() {
        const content =
            typeof this.notebook.content === 'string'
                ? JSON.parse(this.notebook.content)
                : this.notebook.content;

        if (!content?.cells) {
            console.warn('Invalid notebook content format');
            return;
        }

        const renderCells = content.cells.filter((cell) => cell.cell_type === 'render');

        if (renderCells.length === 0) {
            console.warn('No render cells found in notebook');
            return;
        }

        for (const cell of renderCells) {
            const renderConfig =
                typeof cell.content === 'string' ? JSON.parse(cell.content) : cell.content;

            const renderContainer = document.createElement('div');
            renderContainer.className = 'renderer-container';
            renderContainer.dataset.cellId = cell.id;
            renderContainer.style.width = `${renderConfig.width}px`;
            renderContainer.style.height = `${renderConfig.height}px`;
            this.container.appendChild(renderContainer);

            const renderer = new ShaderRenderer(
                renderContainer,
                this.device,
                renderConfig,
                this.resourceManager,
            );

            renderer.initialize();

            this.renderers.set(cell.id, renderer);
        }
    }

    _setupResizeObserver() {
        this.resizeObserver = new ResizeObserver((entries) => {
            for (const entry of entries) {
                const containerWidth = entry.contentRect.width;
                for (const [, renderer] of this.renderers) {
                    renderer.handleContainerResize(containerWidth);
                }
            }
        });

        this.resizeObserver.observe(this.container);
    }

    _startRenderLoop() {
        if (this._animationFrame) {
            return;
        }

        const render = () => {
            if (!this.isRunning) {
                return;
            }

            for (const [, renderer] of this.renderers) {
                if (renderer.isVisible()) {
                    renderer.render();
                }
            }

            this._animationFrame = requestAnimationFrame(render);
        };

        this._animationFrame = requestAnimationFrame(render);
    }

    update(data) {
        if (data.notebook) {
            this.notebook = data.notebook;
            this.reset().then(() => this.initialize());
        }

        if (data.cellId && data.config) {
            const renderer = this.renderers.get(data.cellId);
            if (renderer) {
                renderer.updateConfig(data.config);
            }
        }
    }

    async reset() {
        this.pause();

        for (const [, renderer] of this.renderers) {
            renderer.destroy();
        }

        this.renderers.clear();

        while (this.container.firstChild) {
            this.container.removeChild(this.container.firstChild);
        }

        this.status = 'reset';
    }

    destroy() {
        this.pause();

        if (this.resizeObserver) {
            this.resizeObserver.disconnect();
            this.resizeObserver = null;
        }

        for (const [, renderer] of this.renderers) {
            renderer.destroy();
        }

        this.renderers.clear();

        this.resourceManager.releaseAll();

        if (this.device) {
            this.device = null;
        }

        this.status = 'destroyed';
    }

    resize(width, height) {
        for (const [, renderer] of this.renderers) {
            renderer.resize(width, height);
        }
    }

    pause() {
        this.isRunning = false;
        if (this._animationFrame) {
            cancelAnimationFrame(this._animationFrame);
            this._animationFrame = null;
        }
        this.status = 'paused';
    }

    resume() {
        if (this.status !== 'destroyed' && !this.isRunning) {
            this.isRunning = true;
            this.status = 'running';
            this._startRenderLoop();
        }
    }

    getStatus() {
        return {
            status: this.status,
            rendererCount: this.renderers.size,
            resourceCount: this.resourceManager.getResourceCount(),
            gpuInfo: this.adapter
                ? {
                      name: this.adapter.name,
                      isIntegratedGPU: !!this.adapter.isIntegratedGPU,
                      limits: this.device ? this.device.limits : null,
                  }
                : null,
        };
    }
}

export function createNotebookRenderer(containerId, notebook, options = {}) {
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
