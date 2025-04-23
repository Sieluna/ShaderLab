import { UniformManager } from './uniform-manager.js';
import { GeometryManager } from './geometry-manager.js';
import { TextureManager } from './texture-manager.js';
import { RenderPipelineManager } from './render-pipeline-manager.js';
import { BindGroupManager } from './bind-group-manager.js';
import { RenderPassExecutor } from './render-pass-executor.js';
import { PerformanceMonitor } from './performance-monitor.js';

export class ShaderRenderer {
    // DOM elements
    /** @type {HTMLElement} DOM container element */
    container;
    /** @type {HTMLCanvasElement|null} Canvas element */
    canvas = null;
    /** @type {HTMLElement|null} DOM element for displaying statistics */
    statsDisplay = null;

    // WebGPU context
    /** @type {GPUDevice} WebGPU device */
    device;
    /** @type {GPUCanvasContext|null} WebGPU context */
    context = null;
    /** @type {string} Canvas format */
    canvasFormat = 'rgba8unorm';

    // Resources and configuration
    /** @type {ResourceManager} Resource manager */
    resourceManager;
    /** @type {Object} Render configuration */
    config;

    // Modular components
    /** @type {UniformManager} Uniform manager */
    uniformManager;
    /** @type {GeometryManager} Geometry manager */
    geometryManager;
    /** @type {TextureManager} Texture manager */
    textureManager;
    /** @type {RenderPipelineManager} Render pipeline manager */
    pipelineManager;
    /** @type {BindGroupManager} Bind group manager */
    bindGroupManager;
    /** @type {RenderPassExecutor} Render pass executor */
    passExecutor;
    /** @type {PerformanceMonitor} Performance monitor */
    performanceMonitor;

    // Custom Uniform value storage
    /** @type {Map<string, any>} Custom uniform values */
    customUniforms = new Map();

    // Performance tracking
    /** @type {boolean} Initialization status */
    isInitialized = false;
    /** @type {number} Frame counter */
    frameCount = 0;
    /** @type {number} Timestamp when rendering started */
    startTime = performance.now();
    /** @type {number} Timestamp of last frame */
    lastFrameTime = 0;
    /** @type {Array<number>} Recent frame times array for FPS calculation */
    frameTimes = [];
    /** @type {Object} Performance statistics */
    stats = {
        fps: 0,
        frameTime: 0,
        drawCalls: 0,
        triangleCount: 0,
    };
    /** @type {number} Performance optimization: cached current performance factor */
    _currentPerformanceFactor = 1.0;
    /** @type {number|null} Animation frame request ID */
    _animationFrame = null;

    // Visibility tracking
    /** @type {IntersectionObserver|null} IntersectionObserver for visibility detection */
    intersectionObserver = null;
    /** @type {boolean} Renderer visibility status */
    _isVisible = true;
    /** @type {boolean} Whether rendering is paused */
    _isPaused = false;

    /**
     * Create a shader renderer instance
     * @param {HTMLElement} container - DOM container element for the renderer
     * @param {GPUDevice} device - WebGPU device instance
     * @param {Object} config - Render configuration
     * @param {number} config.width - Canvas width (pixels)
     * @param {number} config.height - Canvas height (pixels)
     * @param {Array<number>} config.shader_ids - Shader ID array
     * @param {Object} config.pipeline - Pipeline configuration
     * @param {Object} config.performance - Performance settings
     * @param {ResourceManager} resourceManager - Resource manager instance
     */
    constructor(container, device, config, resourceManager) {
        this.container = container;
        this.device = device;
        this.config = config;
        this.resourceManager = resourceManager;

        // Create all managers
        this.uniformManager = new UniformManager(device, resourceManager);
        this.geometryManager = new GeometryManager(device, resourceManager);
        this.textureManager = new TextureManager(device);
        this.pipelineManager = new RenderPipelineManager(device);
        this.bindGroupManager = new BindGroupManager(device, resourceManager, this.uniformManager);
        this.passExecutor = new RenderPassExecutor(device);
        this.performanceMonitor = new PerformanceMonitor(config.performance);
    }

    /**
     * Initialize renderer
     * @returns {boolean} Success status
     * @public
     */
    initialize() {
        try {
            // Create canvas
            this._createCanvas();

            // Initialize WebGPU context
            this._initContext();

            // Create statistics display if performance debugging is enabled
            if (this.config.performance?.debug) {
                this.statsDisplay = this.performanceMonitor.createStatsDisplay(this.container);
            }

            // Create default geometry
            const geometryResult = this.geometryManager.createDefaultGeometry(
                this.canvas.width,
                this.canvas.height,
                this.config,
            );

            // Analyze shader code and set default uniforms
            this._analyzeShaders();

            // Create render pipelines
            const renderPipelines = this.pipelineManager.createRenderPipelines(
                this.config,
                this.resourceManager,
                this.uniformManager,
                this.textureManager.getAllPassTextures(),
                this.canvasFormat,
            );

            // Create pass textures
            if (this.config.pipeline && this.config.pipeline.render_passes) {
                this.textureManager.createAllPassTextures(
                    this.config.pipeline.render_passes,
                    this.canvas.width,
                    this.canvas.height,
                );
            }

            // Create bind groups
            this.bindGroupManager.createBindGroups(
                renderPipelines,
                this.config,
                geometryResult.uniformBuffers,
                this.textureManager.getAllPassTextures(),
            );

            // Set up visibility detection
            this._setupVisibilityDetection();

            // Initialize custom uniform values
            this._initCustomUniforms();

            // Start performance monitor
            this.performanceMonitor.startSession();

            this.isInitialized = true;
            return true;
        } catch (error) {
            console.error('Renderer initialization failed:', error);
            this._showError(error.message);
            return false;
        }
    }

    /**
     * Initialize custom uniform values
     * @private
     */
    _initCustomUniforms() {
        // Initialize uniforms if defined in config
        if (this.config.uniforms && Array.isArray(this.config.uniforms)) {
            for (const uniform of this.config.uniforms) {
                if (uniform.name && uniform.default !== undefined) {
                    // Store default value
                    this.customUniforms.set(uniform.name, uniform.default);
                }
            }
        }
    }

    /**
     * Create canvas element and add to container
     * @private
     */
    _createCanvas() {
        this.canvas = document.createElement('canvas');
        this.canvas.width = this.config.width ?? 640;
        this.canvas.height = this.config.height ?? 480;
        this.canvas.className = 'webgpu-canvas';
        this.canvas.style.width = '100%';
        this.canvas.style.height = '100%';

        // Clear container and add canvas
        while (this.container.firstChild) {
            this.container.removeChild(this.container.firstChild);
        }
        this.container.appendChild(this.canvas);
    }

    /**
     * Initialize WebGPU context for canvas
     * @throws {Error} If WebGPU context is not available
     * @private
     */
    _initContext() {
        // Get WebGPU context from canvas
        this.context = this.canvas.getContext('webgpu');

        if (!this.context) {
            throw new Error('WebGPU context not available');
        }

        // Get preferred canvas format from current adapter
        const canvasFormat = navigator.gpu.getPreferredCanvasFormat();

        // Configure canvas context
        this.context.configure({
            device: this.device,
            format: canvasFormat,
            alphaMode: 'premultiplied',
            usage: GPUTextureUsage.RENDER_ATTACHMENT,
        });

        // Store canvas format for later use
        this.canvasFormat = canvasFormat;
    }

    /**
     * Analyze shader code and set default uniform values
     * @private
     */
    _analyzeShaders() {
        // Get all shaders
        if (!this.config.shader_ids || !Array.isArray(this.config.shader_ids)) {
            console.warn('No shader IDs specified in configuration');
            return;
        }

        // Get shader module for each shader ID and analyze
        for (const shaderId of this.config.shader_ids) {
            const shader = this.resourceManager.getShaderModule(shaderId);
            if (!shader) {
                console.warn(`Shader with ID ${shaderId} not found`);
                continue;
            }

            // Analyze shader code
            this.uniformManager.analyzeShader(`shader_${shaderId}`, shader.code);

            // Apply default values if custom uniform settings exist
            if (this.config.uniforms && Array.isArray(this.config.uniforms)) {
                for (const uniform of this.config.uniforms) {
                    if (uniform.name && uniform.default !== undefined) {
                        this.uniformManager.setDefaultValue(
                            `shader_${shaderId}`,
                            uniform.name,
                            uniform.default,
                        );
                    }
                }
            }
        }

        // Special handling for render pass bound shaders
        if (this.config.pipeline && this.config.pipeline.render_passes) {
            for (const passConfig of this.config.pipeline.render_passes) {
                const passId = passConfig.id;

                // If shader_bindings exist, create bind group layout for each bound shader
                if (passConfig.shader_bindings && Array.isArray(passConfig.shader_bindings)) {
                    for (const bindingIndex of passConfig.shader_bindings) {
                        const binding = this.config.pipeline.shader_bindings[bindingIndex];
                        if (!binding) continue;

                        const shaderId = this.config.shader_ids[binding.shader_index];
                        const shader = this.resourceManager.getShaderModule(shaderId);

                        if (shader) {
                            const passShaderKey = `pass_${passId}_shader_${shaderId}`;
                            this.uniformManager.analyzeShader(passShaderKey, shader.code);

                            // Apply default uniform values
                            if (this.config.uniforms && Array.isArray(this.config.uniforms)) {
                                for (const uniform of this.config.uniforms) {
                                    if (uniform.name && uniform.default !== undefined) {
                                        this.uniformManager.setDefaultValue(
                                            passShaderKey,
                                            uniform.name,
                                            uniform.default,
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /**
     * Set up visibility detection
     * @private
     */
    _setupVisibilityDetection() {
        this.intersectionObserver = new IntersectionObserver(
            (entries) => {
                for (const entry of entries) {
                    this._isVisible = entry.isIntersecting;
                }
            },
            { threshold: 0.1 },
        );

        this.intersectionObserver.observe(this.container);
    }

    /**
     * Render a frame
     */
    render() {
        if (!this.isInitialized || !this._isVisible || this._isPaused) {
            return;
        }

        // Start frame timing
        const frameStartTime = this.performanceMonitor.beginFrame();

        // Calculate time differences
        const now = performance.now();
        const time = (now - this.performanceMonitor.startTime) / 1000;
        const delta = (now - this.performanceMonitor.lastFrameTime) / 1000;
        this.performanceMonitor.lastFrameTime = now;

        // Update time uniform buffer
        this.geometryManager.updateTimeUniform(time, delta, this.performanceMonitor.frameCount);

        // Create command encoder
        const encoder = this.device.createCommandEncoder();

        // Get depth texture (if needed)
        const depthTexture = this.textureManager.ensureDepthTexture(
            this.canvas.width,
            this.canvas.height,
        );

        // Execute all render passes
        for (const [passId, pipelineInfo] of this.pipelineManager.renderPipelines.entries()) {
            // Use render pass executor to execute render pass
            this.passExecutor.executeRenderPass(encoder, passId, pipelineInfo, {
                passTextures: this.textureManager.getAllPassTextures(),
                depthTexture,
                canvasContext: this.context,
                vertexBuffers: this.geometryManager.vertexBuffers,
                indexBuffer: this.geometryManager.indexBuffer,
                indexCount: this.geometryManager.indexCount,
            });
        }

        // Submit command buffer
        this.device.queue.submit([encoder.finish()]);

        // End frame timing and update statistics
        this.performanceMonitor.endFrame(
            frameStartTime,
            this.pipelineManager.renderPipelines.size,
            (this.geometryManager.indexCount / 3) * this.pipelineManager.renderPipelines.size,
        );
    }

    /**
     * Resize renderer
     * @param {number} width - Width
     * @param {number} height - Height
     */
    resize(width, height) {
        console.log(`ShaderRenderer.resize called: ${width}x${height}`);

        // Save old dimensions for comparison
        const oldWidth = this.canvas.width;
        const oldHeight = this.canvas.height;

        // Only perform update if dimensions actually changed
        if (width === oldWidth && height === oldHeight) {
            console.log('Dimensions unchanged, skipping resize operation');
            return;
        }

        console.log(`Resizing canvas: from ${oldWidth}x${oldHeight} to ${width}x${height}`);

        // Update canvas size
        this.canvas.width = width;
        this.canvas.height = height;

        // Reconfigure context to ensure WebGPU context works after resize
        try {
            this.context.configure({
                device: this.device,
                format: this.canvasFormat,
                alphaMode: 'premultiplied',
                usage: GPUTextureUsage.RENDER_ATTACHMENT,
            });
        } catch (e) {
            console.error('Error configuring WebGPU context:', e);
        }

        // Update resolution uniform buffer
        this.geometryManager.updateResolutionUniform(width, height);

        // Resize all textures
        this.textureManager.resizeTextures(width, height);

        // Recreate bind groups (because texture views have changed)
        this.bindGroupManager.createBindGroups(
            this.pipelineManager.renderPipelines,
            this.config,
            this.geometryManager.uniformBuffers,
            this.textureManager.getAllPassTextures(),
        );

        console.log(`Resolution update complete: ${width}x${height}`);
    }

    /**
     * Handle container resize
     * @param {number} containerWidth - Container width
     */
    handleContainerResize(containerWidth) {
        // Maintain aspect ratio
        const aspectRatio = this.config.width / this.config.height;
        const newWidth = containerWidth;
        const newHeight = containerWidth / aspectRatio;

        // If adaptive resolution is enabled
        if (this.config.performance?.adaptive_resolution) {
            // Adjust actual render resolution based on device performance
            const performanceFactor = this.performanceMonitor.getPerformanceFactor();
            const renderWidth = Math.floor(newWidth * performanceFactor);
            const renderHeight = Math.floor(newHeight * performanceFactor);

            // Adjust canvas render size
            this.canvas.width = renderWidth;
            this.canvas.height = renderHeight;

            // Maintain display size through CSS
            this.canvas.style.width = `${newWidth}px`;
            this.canvas.style.height = `${newHeight}px`;

            // Update resolution uniform buffer and textures
            this.geometryManager.updateResolutionUniform(renderWidth, renderHeight);
            this.textureManager.resizeTextures(renderWidth, renderHeight);

            // Update bind groups
            this.bindGroupManager.createBindGroups(
                this.pipelineManager.renderPipelines,
                this.config,
                this.geometryManager.uniformBuffers,
                this.textureManager.getAllPassTextures(),
            );
        } else {
            // Directly resize canvas
            this.resize(newWidth, newHeight);
        }
    }

    /**
     * Update render configuration
     * @param {Object} config - Updated configuration
     */
    updateConfig(config) {
        const needsRebuild =
            config.shader_ids !== this.config.shader_ids ||
            config.pipeline?.shader_bindings !== this.config.pipeline?.shader_bindings;

        // Record size before update
        const oldWidth = this.config.width;
        const oldHeight = this.config.height;

        // Update configuration
        this.config = { ...this.config, ...config };

        // If performance settings updated, sync to performance monitor
        if (config.performance) {
            this.performanceMonitor.config = {
                ...this.performanceMonitor.config,
                ...config.performance,
            };
        }

        // Update resolution (check if size actually changed)
        if (
            config.width &&
            config.height &&
            (config.width !== oldWidth || config.height !== oldHeight)
        ) {
            console.log(
                `ShaderRenderer: Updating size: from ${oldWidth}x${oldHeight} to ${config.width}x${config.height}`,
            );
            this.resize(config.width, config.height);
        }

        // If custom uniform settings exist, initialize them
        if (config.uniforms && Array.isArray(config.uniforms)) {
            for (const uniform of config.uniforms) {
                if (
                    uniform.name &&
                    uniform.default !== undefined &&
                    !this.customUniforms.has(uniform.name)
                ) {
                    // Update current storage
                    this.customUniforms.set(uniform.name, uniform.default);

                    // Update uniform buffer
                    this.geometryManager.updateCustomUniform(uniform.name, uniform.default);
                }
            }
        }

        // If render pipeline needs rebuilding
        if (needsRebuild) {
            // Reanalyze shaders
            this._analyzeShaders();

            // Recreate pipelines
            const renderPipelines = this.pipelineManager.createRenderPipelines(
                this.config,
                this.resourceManager,
                this.uniformManager,
                this.textureManager.getAllPassTextures(),
                this.canvasFormat,
            );

            // Create pass textures
            if (this.config.pipeline && this.config.pipeline.render_passes) {
                this.textureManager.createAllPassTextures(
                    this.config.pipeline.render_passes,
                    this.canvas.width,
                    this.canvas.height,
                );
            }

            // Recreate bind groups
            this.bindGroupManager.createBindGroups(
                renderPipelines,
                this.config,
                this.geometryManager.uniformBuffers,
                this.textureManager.getAllPassTextures(),
            );
        }
    }

    /**
     * Update custom uniform value
     * @param {string} name - Uniform name
     * @param {any} value - New value
     */
    updateUniform(name, value) {
        if (!this.isInitialized) {
            console.warn('Cannot update uniform: renderer not initialized');
            return;
        }

        // Store new value
        this.customUniforms.set(name, value);

        try {
            // Update uniform buffer through GeometryManager
            this.geometryManager.updateCustomUniform(name, value);

            // Update through BindGroupManager
            this.bindGroupManager.updateUniformBuffer(
                name,
                value,
                this.geometryManager.uniformBuffers,
                (value) => this.geometryManager._createTypedArrayForValue(value),
            );
        } catch (error) {
            // Handle error silently - uniform update failed but we can continue
            console.debug(`Failed to update uniform ${name}:`, error);
        }
    }

    /**
     * Check if renderer is visible
     * @returns {boolean} Whether it is visible
     */
    isVisible() {
        return this._isVisible;
    }

    /**
     * Show error message
     * @param {string} message - Error message
     * @private
     */
    _showError(message) {
        const errorElement = document.createElement('div');
        errorElement.className = 'renderer-error';
        errorElement.style.color = 'red';
        errorElement.style.padding = '10px';
        errorElement.textContent = `Renderer Error: ${message}`;

        this.container.innerHTML = '';
        this.container.appendChild(errorElement);
    }

    /**
     * Pause rendering
     * @public
     */
    pause() {
        this._isPaused = true;

        // Cancel any pending animation frame
        if (this._animationFrame) {
            cancelAnimationFrame(this._animationFrame);
            this._animationFrame = null;
        }
    }

    /**
     * Resume rendering
     * @public
     */
    resume() {
        this._isPaused = false;

        // Only resume if we were previously rendering
        if (this.isInitialized && !this._animationFrame) {
            this._animationFrame = requestAnimationFrame(() => this.render());
        }
    }

    /**
     * Reset renderer state
     * @public
     */
    reset() {
        // Reset performance monitor
        this.performanceMonitor.startSession();

        // Reset custom uniform values to default state
        if (this.config.uniforms && Array.isArray(this.config.uniforms)) {
            for (const uniform of this.config.uniforms) {
                if (uniform.name && uniform.default !== undefined) {
                    this.updateUniform(uniform.name, uniform.default);
                }
            }
        }
    }

    /**
     * Destroy renderer and release resources
     */
    destroy() {
        // Stop visibility detection
        if (this.intersectionObserver) {
            this.intersectionObserver.disconnect();
            this.intersectionObserver = null;
        }

        // Destroy all components
        this.textureManager.destroy();
        this.geometryManager.destroy();

        // Clear UniformManager
        if (this.uniformManager) {
            this.uniformManager.destroy();
        }

        // Mark as uninitialized
        this.isInitialized = false;
    }

    /**
     * Get performance statistics
     * @returns {Object} Performance statistics
     */
    getPerformanceStats() {
        return this.performanceMonitor.getStats();
    }
}
