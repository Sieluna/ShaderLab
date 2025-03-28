import { ShaderRenderer } from './shader-renderer.js';
import { ResourceManager } from './resource-manager.js';

export class NotebookRendererManager {
    constructor(container, notebook, options = {}) {
        this.container = container;
        this.notebook = notebook;
        this.options = options;
        this.renderers = new Map();
        this.resourceManager = new ResourceManager();
        this.isRunning = false;
        this.adapter = null;
        this.device = null;
        this.status = 'created';
        this.resizeObserver = null;
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
            powerPreference: this.options.powerPreference || 'high-performance',
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

        if (!content || !content.cells) {
            console.warn('Invalid notebook content format');
            return;
        }

        const renderCells = content.cells.filter((cell) => cell.cell_type === 'render');

        if (renderCells.length === 0) {
            console.warn('No render cells found in notebook');
            return;
        }

        for (let i = 0; i < renderCells.length; i++) {
            const cell = renderCells[i];
            const renderConfig = JSON.parse(cell.content);

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
