export class ShaderRenderer {
    /**
     * Create shader renderer
     * @param {HTMLElement} container - Container element
     * @param {GPUDevice} device - WebGPU device
     * @param {Object} config - Render configuration
     * @param {ResourceManager} resourceManager - Resource manager
     */
    constructor(container, device, config, resourceManager) {
        this.container = container;
        this.device = device;
        this.config = config;
        this.resourceManager = resourceManager;

        this.canvas = null;
        this.context = null;
        this.renderPipeline = null;
        this.bindGroups = [];
        this.vertexBuffers = new Map();
        this.indexBuffer = null;
        this.indexCount = 0;
        this.uniformBuffers = new Map();
        this.isInitialized = false;
        this.frameCount = 0;
        this.startTime = performance.now();
        this.lastFrameTime = 0;
        this.frameTimes = []; // For calculating average FPS

        // Performance monitoring
        this.stats = {
            fps: 0,
            frameTime: 0,
            drawCalls: 0,
            triangleCount: 0,
        };

        // IntersectionObserver for visibility detection
        this.intersectionObserver = null;
        this._isVisible = true;
    }

    /**
     * Initialize renderer
     */
    initialize() {
        try {
            // Create canvas
            this._createCanvas();

            // Initialize WebGPU context
            this._initContext();

            // Create default geometry (if needed)
            this._createDefaultGeometry();

            // Create pipeline
            this._createRenderPipeline();

            // Create bind groups
            this._createBindGroups();

            // Setup visibility detection
            this._setupVisibilityDetection();

            this.isInitialized = true;
            return true;
        } catch (error) {
            console.error('Renderer initialization failed:', error);
            this._showError(error.message);
            return false;
        }
    }

    /**
     * Create canvas element
     * @private
     */
    _createCanvas() {
        this.canvas = document.createElement('canvas');
        this.canvas.width = this.config.width || 640;
        this.canvas.height = this.config.height || 480;
        this.canvas.className = 'webgpu-canvas';
        this.canvas.style.width = '100%';
        this.canvas.style.height = '100%';

        // Clear container and add canvas
        while (this.container.firstChild) {
            this.container.removeChild(this.container.firstChild);
        }
        this.container.appendChild(this.canvas);

        // Create stats display (in debug mode)
        if (this.config.performance?.debug) {
            this._createStatsDisplay();
        }
    }

    /**
     * Create performance stats display
     * @private
     */
    _createStatsDisplay() {
        this.statsDisplay = document.createElement('div');
        this.statsDisplay.className = 'renderer-stats';
        this.statsDisplay.style.position = 'absolute';
        this.statsDisplay.style.top = '0';
        this.statsDisplay.style.left = '0';
        this.statsDisplay.style.background = 'rgba(0,0,0,0.5)';
        this.statsDisplay.style.color = 'white';
        this.statsDisplay.style.padding = '5px';
        this.statsDisplay.style.fontSize = '12px';
        this.statsDisplay.style.fontFamily = 'monospace';
        this.statsDisplay.style.zIndex = '100';
        this.container.appendChild(this.statsDisplay);
    }

    /**
     * Initialize WebGPU context
     * @private
     */
    _initContext() {
        // Get WebGPU context
        this.context = this.canvas.getContext('webgpu');

        if (!this.context) {
            throw new Error('WebGPU context not available');
        }

        // Configure context
        const canvasFormat = navigator.gpu.getPreferredCanvasFormat();

        this.context.configure({
            device: this.device,
            format: canvasFormat,
            alphaMode: 'premultiplied',
            usage: GPUTextureUsage.RENDER_ATTACHMENT,
        });

        // Store canvas format
        this.canvasFormat = canvasFormat;
    }

    /**
     * Create default geometry (quad)
     * @private
     */
    _createDefaultGeometry() {
        // Basic quad vertex positions
        const vertices = new Float32Array([
            // Position (x, y, z)   // UV coordinates (u, v)
            -1.0, -1.0, 0.0, 0.0, 1.0, 
            1.0, -1.0, 0.0, 1.0, 1.0, 
            1.0, 1.0, 0.0, 1.0, 0.0, 
            -1.0, 1.0, 0.0, 0.0, 0.0,
        ]);

        // Indices
        const indices = new Uint16Array([
            0, 1, 2, // First triangle
            0, 2, 3, // Second triangle
        ]);

        // Create vertex buffer
        this.vertexBuffers.set(
            'position',
            this.resourceManager.createVertexBuffer('quad_vertex', vertices, this.device),
        );

        // Create index buffer
        this.indexBuffer = this.device.createBuffer({
            label: 'Quad Index Buffer',
            size: indices.byteLength,
            usage: GPUBufferUsage.INDEX | GPUBufferUsage.COPY_DST,
            mappedAtCreation: true,
        });

        new Uint16Array(this.indexBuffer.getMappedRange()).set(indices);
        this.indexBuffer.unmap();

        this.indexCount = indices.length;

        // Create time and resolution uniform buffers
        const timeUniform = new Float32Array([0, 0, 0, 0]); // time, delta, frame, reserved
        this.uniformBuffers.set(
            'time',
            this.resourceManager.createUniformBuffer('time_uniform', timeUniform, this.device),
        );

        const resolutionUniform = new Float32Array([
            this.canvas.width,
            this.canvas.height,
            this.canvas.width / this.canvas.height,
            0,
        ]);
        this.uniformBuffers.set(
            'resolution',
            this.resourceManager.createUniformBuffer(
                'resolution_uniform',
                resolutionUniform,
                this.device,
            ),
        );

        // Create camera uniform buffer
        if (this.config.camera) {
            const { position, target, up, fov, near, far } = this.config.camera;
            const cameraUniform = new Float32Array([
                ...position,
                0, // position (vec4)
                ...target,
                0, // target (vec4)
                ...up,
                0, // up (vec4)
                fov,
                near,
                far,
                0, // fov, near, far, padding
            ]);
            this.uniformBuffers.set(
                'camera',
                this.resourceManager.createUniformBuffer(
                    'camera_uniform',
                    cameraUniform,
                    this.device,
                ),
            );
        }
    }

    /**
     * Create render pipeline
     * @private
     */
    _createRenderPipeline() {
        // Get shaders from config
        const shaderBindings = this.config.pipeline.shader_bindings;
        let vertexShader = null;
        let fragmentShader = null;

        // Find vertex and fragment shaders
        for (const binding of shaderBindings) {
            const shaderId = this.config.shader_ids[binding.shader_index];
            const shader = this.resourceManager.getShaderModule(shaderId);

            if (!shader) {
                throw new Error(`Shader with ID ${shaderId} not found`);
            }

            if (binding.shader_stage === 'vertex') {
                vertexShader = {
                    module: shader.module,
                    entryPoint: binding.entry_point,
                };
            } else if (binding.shader_stage === 'fragment') {
                fragmentShader = {
                    module: shader.module,
                    entryPoint: binding.entry_point,
                };
            }
        }

        if (!vertexShader || !fragmentShader) {
            throw new Error('Both vertex and fragment shaders are required');
        }

        // Create vertex buffer layouts
        const bufferLayouts = [];

        // Add position and UV layout
        bufferLayouts.push({
            arrayStride: 5 * 4, // 5 floats (3 position + 2 uv) * 4 bytes
            attributes: [
                // Position
                {
                    shaderLocation: 0,
                    offset: 0,
                    format: 'float32x3',
                },
                // UV coordinates
                {
                    shaderLocation: 1,
                    offset: 3 * 4, // 3 floats * 4 bytes
                    format: 'float32x2',
                },
            ],
        });

        // Create bind group layouts
        const bindGroupLayouts = [];

        // Time and resolution bind group
        bindGroupLayouts.push(
            this.device.createBindGroupLayout({
                label: 'Time and Resolution Bind Group Layout',
                entries: [
                    // Time uniform buffer
                    {
                        binding: 0,
                        visibility: GPUShaderStage.VERTEX | GPUShaderStage.FRAGMENT,
                        buffer: { type: 'uniform' },
                    },
                    // Resolution uniform buffer
                    {
                        binding: 1,
                        visibility: GPUShaderStage.VERTEX | GPUShaderStage.FRAGMENT,
                        buffer: { type: 'uniform' },
                    },
                ],
            }),
        );

        // Add user-defined resource bindings
        if (
            this.config.pipeline.resource_bindings &&
            this.config.pipeline.resource_bindings.length > 0
        ) {
            // Group bindings by group
            const bindingsByGroup = {};

            for (const binding of this.config.pipeline.resource_bindings) {
                if (!bindingsByGroup[binding.group]) {
                    bindingsByGroup[binding.group] = [];
                }
                bindingsByGroup[binding.group].push(binding);
            }

            // Create layout for each binding group
            for (const [group, bindings] of Object.entries(bindingsByGroup)) {
                const entries = bindings.map((binding) => {
                    const resourceId = this.config.resource_ids[binding.resource_index];
                    let entry = {
                        binding: binding.binding,
                        visibility: GPUShaderStage.VERTEX | GPUShaderStage.FRAGMENT,
                    };

                    switch (binding.binding_type) {
                        case 'uniform':
                            entry.buffer = { type: 'uniform' };
                            break;
                        case 'storage':
                            entry.buffer = { type: 'read-only-storage' };
                            break;
                        case 'texture':
                            entry.texture = {};
                            break;
                        case 'sampler':
                            entry.sampler = {};
                            break;
                        default:
                            throw new Error(`Unsupported binding type: ${binding.binding_type}`);
                    }

                    return entry;
                });

                bindGroupLayouts.push(
                    this.device.createBindGroupLayout({
                        label: `Resource Bind Group Layout ${group}`,
                        entries,
                    }),
                );
            }
        }

        // Create pipeline layout
        const pipelineLayout = this.device.createPipelineLayout({
            label: 'Render Pipeline Layout',
            bindGroupLayouts,
        });

        // Create render pipeline
        this.renderPipeline = this.device.createRenderPipeline({
            label: 'Render Pipeline',
            layout: pipelineLayout,
            vertex: {
                module: vertexShader.module,
                entryPoint: vertexShader.entryPoint,
                buffers: bufferLayouts,
            },
            fragment: {
                module: fragmentShader.module,
                entryPoint: fragmentShader.entryPoint,
                targets: [
                    {
                        format: this.canvasFormat,
                        blend: {
                            color: {
                                srcFactor: 'src-alpha',
                                dstFactor: 'one-minus-src-alpha',
                                operation: 'add',
                            },
                            alpha: {
                                srcFactor: 'one',
                                dstFactor: 'one-minus-src-alpha',
                                operation: 'add',
                            },
                        },
                    },
                ],
            },
            primitive: {
                topology: 'triangle-list',
                cullMode: 'none',
                frontFace: 'ccw',
            },
            depthStencil: this.config.pipeline.render_pass?.depth_enabled
                ? {
                      format: 'depth24plus',
                      depthWriteEnabled: true,
                      depthCompare: 'less',
                  }
                : undefined,
        });
    }

    /**
     * Create bind groups
     * @private
     */
    _createBindGroups() {
        // Create time and resolution bind group
        this.bindGroups[0] = this.device.createBindGroup({
            label: 'Time and Resolution Bind Group',
            layout: this.renderPipeline.getBindGroupLayout(0),
            entries: [
                {
                    binding: 0,
                    resource: { buffer: this.uniformBuffers.get('time') },
                },
                {
                    binding: 1,
                    resource: { buffer: this.uniformBuffers.get('resolution') },
                },
            ],
        });

        // Create user-defined resource bind groups
        if (
            this.config.pipeline.resource_bindings &&
            this.config.pipeline.resource_bindings.length > 0
        ) {
            // Group bindings by group
            const bindingsByGroup = {};

            for (const binding of this.config.pipeline.resource_bindings) {
                if (!bindingsByGroup[binding.group]) {
                    bindingsByGroup[binding.group] = [];
                }
                bindingsByGroup[binding.group].push(binding);
            }

            // Create bind groups for each binding group
            for (const [groupIndex, bindings] of Object.entries(bindingsByGroup)) {
                const entries = bindings.map((binding) => {
                    const resourceId = this.config.resource_ids[binding.resource_index];
                    let resource = null;

                    switch (binding.binding_type) {
                        case 'uniform':
                        case 'storage':
                            const buffer = this.resourceManager.getBuffer(resourceId);
                            resource = { buffer: buffer?.buffer };
                            break;
                        case 'texture':
                            const texture = this.resourceManager.getTexture(resourceId);
                            resource = { textureView: texture?.texture.createView() };
                            break;
                        case 'sampler':
                            const sampler =
                                this.resourceManager.getSampler(resourceId) ||
                                this.resourceManager.getSampler(`${resourceId}_default`);
                            resource = sampler;
                            break;
                    }

                    if (!resource) {
                        throw new Error(
                            `Resource ${resourceId} not found for binding ${binding.binding}`,
                        );
                    }

                    return {
                        binding: binding.binding,
                        resource,
                    };
                });

                // Create bind group
                this.bindGroups[parseInt(groupIndex)] = this.device.createBindGroup({
                    label: `Resource Bind Group ${groupIndex}`,
                    layout: this.renderPipeline.getBindGroupLayout(parseInt(groupIndex)),
                    entries,
                });
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
     * Update rendering configuration
     * @param {Object} config - updated configuration
     */
    updateConfig(config) {
        const needsRebuild =
            config.shader_ids !== this.config.shader_ids ||
            config.pipeline?.shader_bindings !== this.config.pipeline?.shader_bindings;

        // Update configuration
        this.config = { ...this.config, ...config };

        // Update resolution
        if (config.width && config.height) {
            this.resize(config.width, config.height);
        }

        // Whether to rebuild the pipeline
        if (needsRebuild) {
            // Rebuild pipeline
            this._createRenderPipeline();
            // Rebuild bind groups
            this._createBindGroups();
        }
    }

    /**
     * Render a frame
     */
    render() {
        if (!this.isInitialized || !this._isVisible) {
            return;
        }

        const now = performance.now();
        const time = (now - this.startTime) / 1000;
        const delta = (now - this.lastFrameTime) / 1000;
        this.lastFrameTime = now;

        // Update time uniform buffer
        const timeData = new Float32Array([
            time, // Total time (seconds)
            delta, // Frame interval (seconds)
            this.frameCount, // Frame count
            0, // Reserved
        ]);

        this.resourceManager.updateBuffer('time_uniform', timeData, this.device);

        // Get current frame texture view
        const textureView = this.context.getCurrentTexture().createView();

        // Create render pass descriptor
        const renderPassDescriptor = {
            colorAttachments: [
                {
                    view: textureView,
                    clearValue: this.config.pipeline.render_pass?.clear_color || {
                        r: 0,
                        g: 0,
                        b: 0,
                        a: 1,
                    },
                    loadOp: 'clear',
                    storeOp: 'store',
                },
            ],
        };

        // Add depth attachment (if needed)
        if (this.config.pipeline.render_pass?.depth_enabled) {
            if (
                !this.depthTexture ||
                this.depthTexture.width !== this.canvas.width ||
                this.depthTexture.height !== this.canvas.height
            ) {
                // Create depth texture
                if (this.depthTexture) {
                    this.depthTexture.destroy();
                }

                this.depthTexture = this.device.createTexture({
                    label: 'Depth Texture',
                    size: [this.canvas.width, this.canvas.height],
                    format: 'depth24plus',
                    usage: GPUTextureUsage.RENDER_ATTACHMENT,
                });
            }

            renderPassDescriptor.depthStencilAttachment = {
                view: this.depthTexture.createView(),
                depthClearValue: this.config.pipeline.render_pass.clear_depth || 1.0,
                depthLoadOp: 'clear',
                depthStoreOp: 'store',
            };
        }

        // Create command encoder
        const encoder = this.device.createCommandEncoder();

        // Begin render pass
        const pass = encoder.beginRenderPass(renderPassDescriptor);

        // Set pipeline
        pass.setPipeline(this.renderPipeline);

        // Set vertex buffer
        pass.setVertexBuffer(0, this.vertexBuffers.get('position'));

        // Set index buffer
        pass.setIndexBuffer(this.indexBuffer, 'uint16');

        // Set bind groups
        for (let i = 0; i < this.bindGroups.length; i++) {
            if (this.bindGroups[i]) {
                pass.setBindGroup(i, this.bindGroups[i]);
            }
        }

        // Draw command
        pass.drawIndexed(this.indexCount);

        // End render pass
        pass.end();

        // Submit command buffer
        this.device.queue.submit([encoder.finish()]);

        // Update statistics
        this._updateStats(delta);

        // Update frame count
        this.frameCount++;
    }

    /**
     * Update performance statistics
     * @param {number} delta - Frame time (in seconds)
     * @private
     */
    _updateStats(delta) {
        // Update frame time
        this.stats.frameTime = delta * 1000; // Convert to milliseconds

        // Maintain recent 30 frame times
        this.frameTimes.push(delta);
        if (this.frameTimes.length > 30) {
            this.frameTimes.shift();
        }

        // Calculate average FPS
        const averageDelta = this.frameTimes.reduce((a, b) => a + b, 0) / this.frameTimes.length;
        this.stats.fps = Math.round(1 / averageDelta);

        // Update draw calls and triangle count
        this.stats.drawCalls = 1;
        this.stats.triangleCount = this.indexCount / 3;

        // If debug mode is enabled, update display
        if (this.statsDisplay) {
            this.statsDisplay.textContent =
                `FPS: ${this.stats.fps} | ` +
                `Frame time: ${this.stats.frameTime.toFixed(2)}ms | ` +
                `Draw calls: ${this.stats.drawCalls} | ` +
                `Triangles: ${this.stats.triangleCount}`;
        }
    }

    /**
     * Resize the renderer
     * @param {number} width - update width
     * @param {number} height - update height
     */
    resize(width, height) {
        // Update Canvas size
        this.canvas.width = width;
        this.canvas.height = height;

        // Update resolution uniform buffer
        const resolutionData = new Float32Array([width, height, width / height, 0]);

        this.resourceManager.updateBuffer('resolution_uniform', resolutionData, this.device);

        // Clear depth texture, it will be recreated in the next frame
        if (this.depthTexture) {
            this.depthTexture.destroy();
            this.depthTexture = null;
        }
    }

    /**
     * Handle container resize
     * @param {number} containerWidth - Container width
     */
    handleContainerResize(containerWidth) {
        // Keep aspect ratio
        const aspectRatio = this.config.width / this.config.height;
        const newWidth = containerWidth;
        const newHeight = containerWidth / aspectRatio;

        // If adaptive resolution is enabled
        if (this.config.performance?.adaptive_resolution) {
            // Adjust actual rendering resolution based on device performance
            const performanceFactor = this._getPerformanceFactor();
            const renderWidth = Math.floor(newWidth * performanceFactor);
            const renderHeight = Math.floor(newHeight * performanceFactor);

            // Adjust Canvas rendering size
            this.canvas.width = renderWidth;
            this.canvas.height = renderHeight;

            // Keep display size through CSS
            this.canvas.style.width = `${newWidth}px`;
            this.canvas.style.height = `${newHeight}px`;
        } else {
            // Directly adjust Canvas size
            this.resize(newWidth, newHeight);
        }
    }

    /**
     * Get performance factor
     * @returns {number} Performance factor (0.5 to 1.0)
     * @private
     */
    _getPerformanceFactor() {
        // Determine performance based on FPS
        if (!this.stats.fps) return 1.0;

        const targetFps = this.config.performance?.max_fps || 60;

        if (this.stats.fps >= targetFps * 0.9) {
            // Good performance
            return Math.min(this._currentPerformanceFactor + 0.05, 1.0);
        } else if (this.stats.fps < targetFps * 0.7) {
            // Poor performance
            return Math.max(this._currentPerformanceFactor - 0.1, 0.5);
        }

        // Keep current factor
        return this._currentPerformanceFactor || 1.0;
    }

    /**
     * Check if the renderer is visible
     * @returns {boolean} Is visible
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
        errorElement.textContent = `Renderer error: ${message}`;

        this.container.innerHTML = '';
        this.container.appendChild(errorElement);
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

        // Release depth texture
        if (this.depthTexture) {
            this.depthTexture.destroy();
            this.depthTexture = null;
        }

        // Mark as uninitialized
        this.isInitialized = false;
    }
}
