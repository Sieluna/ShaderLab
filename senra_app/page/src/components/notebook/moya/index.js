import { ResourceManager } from './resource-manager.js';
import { ShaderRenderer } from './renderer.js';

export class MoyaEngine {
    options;
    adapter = null;
    device = null;
    resourceManager;
    renderers = new Map();
    status = 'created';
    _animationFrame = null;
    isRunning = false;

    /**
     * Create engine instance
     * @param {Object} options - Engine options
     */
    constructor(options = {}) {
        this.options = options;
        this.resourceManager = new ResourceManager();
    }

    /**
     * Initialize engine
     * @returns {Promise<boolean>} Whether initialization was successful
     */
    async initialize() {
        try {
            this.status = 'initializing';

            // Initialize WebGPU
            await this._initWebGPU();

            // Start render loop
            this._startRenderLoop();

            this.status = 'ready';
            this.isRunning = true;
            return true;
        } catch (error) {
            this.status = 'error';
            console.error('Engine initialization failed:', error);
            throw error;
        }
    }

    /**
     * Initialize WebGPU
     * @private
     */
    async _initWebGPU() {
        // Request adapter
        this.adapter = await navigator.gpu.requestAdapter({
            powerPreference: this.options.powerPreference ?? 'high-performance',
        });

        if (!this.adapter) {
            throw new Error('WebGPU adapter not available');
        }

        // Request device
        this.device = await this.adapter.requestDevice();

        // Listen for device loss
        this.device.lost.then((info) => {
            console.error('WebGPU device lost:', info);
            this.status = 'device-lost';

            // Try to reinitialize
            if (this.isRunning) {
                this.initialize().catch(console.error);
            }
        });
    }

    /**
     * Start render loop
     * @private
     */
    _startRenderLoop() {
        if (this._animationFrame) {
            return;
        }

        const render = () => {
            if (!this.isRunning) {
                return;
            }

            // Render all active renderers
            for (const [containerId, renderer] of this.renderers.entries()) {
                if (renderer.isVisible && renderer.isVisible() && !renderer._isPaused) {
                    // Check if renderer is paused
                    renderer.render();
                }
            }

            this._animationFrame = requestAnimationFrame(render);
        };

        this._animationFrame = requestAnimationFrame(render);
    }

    /**
     * Process engine commands
     * @param {Array} commands - Command array
     */
    processCommands(commands) {
        if (!Array.isArray(commands) || commands.length === 0) {
            return;
        }

        for (const command of commands) {
            const { type, data } = command;
            this._executeCommand(type, data);
        }
    }

    /**
     * Execute single command
     * @param {string} type - Command type
     * @param {Object} data - Command data
     * @private
     */
    _executeCommand(type, data) {
        switch (type) {
            case 'init':
                this._handleInitCommand(data);
                break;

            case 'update':
                this._handleUpdateCommand(data);
                break;

            case 'resize':
                this._handleResizeCommand(data);
                break;

            case 'uniform':
                this._handleUniformCommand(data);
                break;

            case 'reset':
                this._handleResetCommand(data);
                break;

            case 'destroy':
                this._handleDestroyCommand(data);
                break;

            case 'pause':
                this._handlePauseCommand(data);
                break;

            case 'resume':
                this._handleResumeCommand(data);
                break;

            default:
                console.warn(`Unknown command type: ${type}`);
                break;
        }
    }

    /**
     * Handle initialization command
     * @param {Object} data - Command data
     * @private
     */
    async _handleInitCommand(data) {
        const { containerId, notebook, config, options } = data;

        // Ensure container exists
        const container = document.getElementById(containerId);
        if (!container) {
            console.error(`Container with id '${containerId}' not found`);
            return;
        }

        // Load resources
        if (notebook) {
            await this._loadNotebookResources(notebook);
        }

        // Create renderer
        const renderer = new ShaderRenderer(container, this.device, config, this.resourceManager);

        // Initialize renderer
        if (renderer.initialize()) {
            this.renderers.set(containerId, renderer);
        } else {
            console.error(`Failed to initialize renderer for container '${containerId}'`);
        }
    }

    /**
     * Load notebook resources
     * @param {Object} notebook - Notebook data
     * @private
     */
    async _loadNotebookResources(notebook) {
        if (!notebook.resources && !notebook.shaders) {
            return;
        }

        // Load resources
        if (notebook.resources) {
            for (const resource of notebook.resources) {
                await this.resourceManager.loadResource(resource, this.device);
            }
        }

        // Load shaders
        if (notebook.shaders) {
            for (const shader of notebook.shaders) {
                await this.resourceManager.loadShader(shader, this.device);
            }
        }
    }

    /**
     * Handle update command
     * @param {Object} data - Command data
     * @private
     */
    async _handleUpdateCommand(data) {
        // If notebook data is included, load resources first
        if (data.notebook) {
            await this._loadNotebookResources(data.notebook);
        }

        // If container ID is specified, update specific renderer
        if (data.containerId && this.renderers.has(data.containerId)) {
            const renderer = this.renderers.get(data.containerId);

            if (data.config) {
                renderer.updateConfig(data.config);
            }
        }
        // Otherwise it might be a global update, need to reload all renderers
        else if (data.notebook) {
            // This case is usually handled by the upper Manager, recreating all renderers
        }
    }

    /**
     * Handle resize command
     * @param {Object} data - Command data
     * @private
     */
    _handleResizeCommand(data) {
        const { containerId, width, height } = data;

        if (containerId && this.renderers.has(containerId)) {
            const renderer = this.renderers.get(containerId);
            renderer.resize(width, height);
        }
    }

    /**
     * Handle Uniform update command
     * @param {Object} data - Command data
     * @private
     */
    _handleUniformCommand(data) {
        const { containerId, name, value } = data;

        if (containerId && this.renderers.has(containerId)) {
            const renderer = this.renderers.get(containerId);
            renderer.updateUniform(name, value);
        }
    }

    /**
     * Handle reset command
     * @param {Object} data - Command data
     * @private
     */
    _handleResetCommand(data) {
        const { containerId } = data;

        if (containerId && this.renderers.has(containerId)) {
            const renderer = this.renderers.get(containerId);
            renderer.reset();
        }
    }

    /**
     * Handle destroy command
     * @param {Object} data - Command data
     * @private
     */
    _handleDestroyCommand(data) {
        const { containerId } = data;

        if (containerId && this.renderers.has(containerId)) {
            const renderer = this.renderers.get(containerId);
            renderer.destroy();
            this.renderers.delete(containerId);
        }
    }

    /**
     * Handle pause command
     * @param {Object} data - Command data
     * @private
     */
    _handlePauseCommand(data) {
        const { containerId } = data;

        if (containerId && this.renderers.has(containerId)) {
            const renderer = this.renderers.get(containerId);
            renderer.pause();
        }
    }

    /**
     * Handle resume command
     * @param {Object} data - Command data
     * @private
     */
    _handleResumeCommand(data) {
        const { containerId } = data;

        if (containerId && this.renderers.has(containerId)) {
            const renderer = this.renderers.get(containerId);
            renderer.resume();
        }
    }

    /**
     * Pause engine
     */
    pause() {
        this.isRunning = false;

        if (this._animationFrame) {
            cancelAnimationFrame(this._animationFrame);
            this._animationFrame = null;
        }

        // Pause all renderers
        for (const [, renderer] of this.renderers.entries()) {
            renderer.pause();
        }

        this.status = 'paused';
    }

    /**
     * Resume engine
     */
    resume() {
        if (this.status !== 'destroyed') {
            // Resume all renderers
            for (const [, renderer] of this.renderers.entries()) {
                renderer.resume();
            }

            this.isRunning = true;
            this.status = 'running';

            // Restart render loop
            this._startRenderLoop();
        }
    }

    /**
     * Destroy engine
     */
    destroy() {
        // Stop render loop
        this.pause();

        // Destroy all renderers
        for (const [, renderer] of this.renderers.entries()) {
            renderer.destroy();
        }

        this.renderers.clear();

        // Release all resources
        this.resourceManager.releaseAll();

        this.device = null;
        this.adapter = null;
        this.status = 'destroyed';
    }

    /**
     * Get engine status
     * @returns {Object} Engine status
     */
    getStatus() {
        return {
            status: this.status,
            rendererCount: this.renderers.size,
            resourceCount: this.resourceManager.getResourceCount(),
            gpu: this.adapter
                ? {
                      name: this.adapter.name,
                      isIntegratedGPU: !!this.adapter.isIntegratedGPU,
                      limits: this.device ? this.device.limits : null,
                  }
                : null,
        };
    }
}
