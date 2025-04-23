export class ShaderRenderer {
    // DOM elements
    /** @type {HTMLElement} DOM container element */
    container;
    /** @type {HTMLCanvasElement|null} Canvas element */
    canvas = null;
    /** @type {HTMLElement|null} DOM element for stats display */
    statsDisplay = null;

    // WebGPU context
    /** @type {GPUDevice} WebGPU device */
    device;
    /** @type {GPUCanvasContext|null} WebGPU context */
    context = null;
    /** @type {string} Canvas format */
    canvasFormat = 'rgba8unorm';

    // Resources
    /** @type {ResourceManager} Resource manager */
    resourceManager;
    /** @type {Object} Rendering configuration */
    config;

    // WebGPU resources
    /** @type {Map<string, Object>} Map of render pipelines for multi-pass rendering */
    renderPipelines = new Map();
    /** @type {Map<string, GPUBindGroupLayout>} Map of bind group layouts */
    bindGroupLayouts = new Map();
    /** @type {Array<GPUBindGroup>} Bind groups for rendering */
    bindGroups = [];
    /** @type {Map<string, GPUBuffer>} Map of vertex buffers */
    vertexBuffers = new Map();
    /** @type {GPUBuffer|null} Index buffer */
    indexBuffer = null;
    /** @type {number} Number of indices in the index buffer */
    indexCount = 0;
    /** @type {Map<string, GPUBuffer>} Map of uniform buffers */
    uniformBuffers = new Map();
    /** @type {Map<string, Object>} Map of pass textures for inter-pass communication */
    passTextures = new Map();
    /** @type {GPUTexture|null} Depth texture for depth testing */
    depthTexture = null;

    // Performance tracking
    /** @type {boolean} Initialization status */
    isInitialized = false;
    /** @type {number} Frame counter */
    frameCount = 0;
    /** @type {number} Timestamp when rendering started */
    startTime = performance.now();
    /** @type {number} Timestamp of the last frame */
    lastFrameTime = 0;
    /** @type {Array<number>} Array of recent frame times for FPS calculation */
    frameTimes = [];
    /** @type {Object} Performance statistics */
    stats = {
        fps: 0,
        frameTime: 0,
        drawCalls: 0,
        triangleCount: 0,
    };
    /** @type {number} Performance optimization: Cache for current performance factor */
    _currentPerformanceFactor = 1.0;
    /** @type {number|null} Animation frame request ID */
    _animationFrame = null;

    // Visibility tracking
    /** @type {IntersectionObserver|null} IntersectionObserver for visibility detection */
    intersectionObserver = null;
    /** @type {boolean} Visibility status of the renderer */
    _isVisible = true;

    /**
     * Creates a shader renderer instance
     * @param {HTMLElement} container - DOM container element where the renderer will be placed
     * @param {GPUDevice} device - WebGPU device instance
     * @param {Object} config - Rendering configuration
     * @param {number} config.width - Canvas width in pixels
     * @param {number} config.height - Canvas height in pixels
     * @param {Array<number>} config.shader_ids - Array of shader IDs
     * @param {Object} config.pipeline - Pipeline configuration
     * @param {Object} config.performance - Performance settings
     * @param {ResourceManager} resourceManager - Resource manager instance
     */
    constructor(container, device, config, resourceManager) {
        this.container = container;
        this.device = device;
        this.config = config;
        this.resourceManager = resourceManager;
    }

    /**
     * Initializes the renderer
     * @returns {boolean} Success status
     * @public
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
     * Creates a canvas element and adds it to the container
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

        // Create stats display (in debug mode)
        if (this.config.performance?.debug) {
            this._createStatsDisplay();
        }
    }

    /**
     * Creates a performance stats display
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
     * Initializes the WebGPU context for the canvas
     * @throws {Error} If WebGPU context is not available
     * @private
     */
    _initContext() {
        // Get WebGPU context from canvas
        this.context = this.canvas.getContext('webgpu');

        if (!this.context) {
            throw new Error('WebGPU context not available');
        }

        // Get preferred canvas format for the current adapter
        const canvasFormat = navigator.gpu.getPreferredCanvasFormat();

        // Configure the canvas context
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
     * Creates default quad geometry for rendering
     * Creates default buffers:
     * - Position/UV vertex buffer
     * - Index buffer
     * - Time uniform buffer
     * - Resolution uniform buffer
     * - Camera uniform buffer (if camera is defined in config)
     * @private
     */
    _createDefaultGeometry() {
        // Basic quad with position and texture coordinates
        const vertices = new Float32Array([
            // Position (x, y, z)   // UV coordinates (u, v)
            -1.0, -1.0, 0.0,        0.0, 1.0,
             1.0, -1.0, 0.0,        1.0, 1.0,
             1.0,  1.0, 0.0,        1.0, 0.0,
            -1.0,  1.0, 0.0,        0.0, 0.0,
        ]);

        // Indices for the quad (two triangles)
        const indices = new Uint16Array([
            0, 1, 2, // First triangle
            0, 2, 3, // Second triangle
        ]);

        // Create vertex buffer using resource manager
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

        // Write indices to the buffer
        new Uint16Array(this.indexBuffer.getMappedRange()).set(indices);
        this.indexBuffer.unmap();
        this.indexCount = indices.length;

        // Initialize time uniform buffer (time, delta, frame, reserved)
        const timeUniform = new Float32Array([0, 0, 0, 0]);
        this.uniformBuffers.set(
            'time',
            this.resourceManager.createUniformBuffer('time_uniform', timeUniform, this.device),
        );

        // Initialize resolution uniform buffer (width, height, aspect, reserved)
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

        // Initialize camera uniform buffer if camera config is provided
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
     * Creates render pipelines for all render passes
     * @private
     */
    _createRenderPipeline() {
        // If no render passes are defined, create a default main pass
        if (
            !this.config.pipeline.render_passes ||
            this.config.pipeline.render_passes.length === 0
        ) {
            this.config.pipeline.render_passes = [
                {
                    id: 'main',
                    pass_type: 'main',
                    clear_color: [0.0, 0.0, 0.0, 1.0],
                    clear_depth: 1.0,
                    clear_stencil: 0,
                },
            ];
        }

        // Create pipelines for each render pass
        for (const passConfig of this.config.pipeline.render_passes) {
            this._createPassPipeline(passConfig);
        }
    }

    /**
     * Creates a pipeline for a specific render pass
     * @param {Object} passConfig - Render pass configuration
     * @param {string} passConfig.id - Unique ID for the pass
     * @param {string} passConfig.pass_type - Type of pass (main, intermediate, postprocess, compute)
     * @param {Array<number>} [passConfig.shader_bindings] - Indices of shader bindings to use for this pass
     * @param {Array<Object>} [passConfig.input_textures] - Input textures for this pass
     * @param {Array<Object>} [passConfig.output_textures] - Output textures for this pass
     * @param {boolean} [passConfig.depth_enabled] - Whether depth testing is enabled
     * @param {Array<number>} passConfig.clear_color - RGBA clear color
     * @throws {Error} If required shaders are not found
     * @private
     */
    _createPassPipeline(passConfig) {
        // Get all shader bindings from the config
        const shaderBindings = this.config.pipeline.shader_bindings;
        let vertexShader = null;
        let fragmentShader = null;

        // Check if pass has specific shader bindings
        const usePassSpecificShaders =
            Array.isArray(passConfig.shader_bindings) && passConfig.shader_bindings.length >= 2;

        if (usePassSpecificShaders) {
            // Use pass-specific shader bindings
            const vertexBindingIndex = passConfig.shader_bindings[0];
            const fragmentBindingIndex = passConfig.shader_bindings[1];

            // Get vertex shader
            const vertexBinding = shaderBindings[vertexBindingIndex];
            if (vertexBinding && vertexBinding.shader_stage === 'vertex') {
                const shaderId = this.config.shader_ids[vertexBinding.shader_index];
                const shader = this.resourceManager.getShaderModule(shaderId);
                if (shader) {
                    vertexShader = {
                        module: shader.module,
                        entryPoint: vertexBinding.entry_point,
                    };
                }
            }

            // Get fragment shader
            const fragmentBinding = shaderBindings[fragmentBindingIndex];
            if (fragmentBinding && fragmentBinding.shader_stage === 'fragment') {
                const shaderId = this.config.shader_ids[fragmentBinding.shader_index];
                const shader = this.resourceManager.getShaderModule(shaderId);
                if (shader) {
                    fragmentShader = {
                        module: shader.module,
                        entryPoint: fragmentBinding.entry_point,
                    };
                }
            }
        } else {
            // Use the default method to find shaders (backward compatibility)
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
        }

        // Ensure we have both vertex and fragment shaders for non-compute passes
        if (passConfig.pass_type !== 'compute' && (!vertexShader || !fragmentShader)) {
            throw new Error(
                `Both vertex and fragment shaders are required for pass ${passConfig.id}`,
            );
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

        // Time and resolution bind group layout
        const timeResolutionLayout = this._createTimeResolutionBindGroupLayout(passConfig.id);
        bindGroupLayouts.push(timeResolutionLayout);

        // Add input texture bind group layout if pass has input textures
        if (passConfig.input_textures && passConfig.input_textures.length > 0) {
            const textureLayout = this._createTextureBindGroupLayout(passConfig);
            if (textureLayout) {
                bindGroupLayouts.push(textureLayout);
            }
        }

        // Add resource bind group layouts
        if (
            this.config.pipeline.resource_bindings &&
            this.config.pipeline.resource_bindings.length > 0
        ) {
            const resourceLayouts = this._createResourceBindGroupLayouts(passConfig.id);
            bindGroupLayouts.push(...resourceLayouts);
        }

        // Create pipeline layout
        const pipelineLayout = this.device.createPipelineLayout({
            label: `Render Pipeline Layout for ${passConfig.id}`,
            bindGroupLayouts,
        });

        // Create output textures for intermediate passes
        if (passConfig.pass_type === 'intermediate' || passConfig.pass_type === 'postprocess') {
            this._createPassTextures(passConfig);
        }

        // Determine color target formats
        const colorTargets = this._createColorTargets(passConfig);

        // Create pipeline descriptor
        const pipelineDescriptor = {
            label: `Render Pipeline for ${passConfig.id}`,
            layout: pipelineLayout,
            vertex: {
                module: vertexShader.module,
                entryPoint: vertexShader.entryPoint,
                buffers: bufferLayouts,
            },
            fragment: {
                module: fragmentShader.module,
                entryPoint: fragmentShader.entryPoint,
                targets: colorTargets,
            },
            primitive: {
                topology: 'triangle-list',
                cullMode: 'none',
                frontFace: 'ccw',
            },
        };

        // Add depth stencil state if depth is enabled
        if (passConfig.depth_enabled) {
            pipelineDescriptor.depthStencil = {
                format: 'depth24plus',
                depthWriteEnabled: true,
                depthCompare: 'less',
            };
        }

        // Create the render pipeline
        const pipeline = this.device.createRenderPipeline(pipelineDescriptor);

        // Store pipeline and related information
        this.renderPipelines.set(passConfig.id, {
            pipeline,
            config: passConfig,
            layout: pipelineLayout,
            bindGroupLayouts,
            colorTargets,
        });
    }

    /**
     * Creates a bind group layout for time and resolution uniforms
     * @param {string} passId - Pass identifier
     * @returns {GPUBindGroupLayout} The created bind group layout
     * @private
     */
    _createTimeResolutionBindGroupLayout(passId) {
        const layout = this.device.createBindGroupLayout({
            label: `Time and Resolution Bind Group Layout for ${passId}`,
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
        });

        this.bindGroupLayouts.set(`${passId}_timeResolution`, layout);
        return layout;
    }

    /**
     * Creates a bind group layout for input textures
     * @param {Object} passConfig - Pass configuration
     * @returns {GPUBindGroupLayout|null} The created bind group layout or null if no textures
     * @private
     */
    _createTextureBindGroupLayout(passConfig) {
        if (!passConfig.input_textures || passConfig.input_textures.length === 0) {
            return null;
        }

        const textureLayoutEntries = [];

        for (const inputTexture of passConfig.input_textures) {
            // Texture binding
            textureLayoutEntries.push({
                binding: inputTexture.binding,
                visibility: GPUShaderStage.FRAGMENT,
                texture: {},
            });

            // Sampler binding (usually the next binding)
            textureLayoutEntries.push({
                binding: inputTexture.binding + 1,
                visibility: GPUShaderStage.FRAGMENT,
                sampler: {},
            });
        }

        // Create texture bind group layout
        const layout = this.device.createBindGroupLayout({
            label: `Texture Bind Group Layout for ${passConfig.id}`,
            entries: textureLayoutEntries,
        });

        this.bindGroupLayouts.set(`${passConfig.id}_textures`, layout);
        return layout;
    }

    /**
     * Creates bind group layouts for resource bindings
     * @param {string} passId - Pass identifier
     * @returns {Array<GPUBindGroupLayout>} Array of created bind group layouts
     * @private
     */
    _createResourceBindGroupLayouts(passId) {
        const layouts = [];
        const bindingsByGroup = {};

        // Group bindings by group index
        for (const binding of this.config.pipeline.resource_bindings) {
            if (!bindingsByGroup[binding.group]) {
                bindingsByGroup[binding.group] = [];
            }
            bindingsByGroup[binding.group].push(binding);
        }

        // Create a layout for each binding group
        for (const [group, bindings] of Object.entries(bindingsByGroup)) {
            const entries = bindings.map((binding) => {
                const entry = {
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

            const layout = this.device.createBindGroupLayout({
                label: `Resource Bind Group Layout ${group} for ${passId}`,
                entries,
            });

            layouts.push(layout);
            this.bindGroupLayouts.set(`${passId}_resources_${group}`, layout);
        }

        return layouts;
    }

    /**
     * Creates color targets for a render pass
     * @param {Object} passConfig - Pass configuration
     * @returns {Array<Object>} Array of color targets
     * @private
     */
    _createColorTargets(passConfig) {
        const colorTargets = [];

        if (passConfig.pass_type === 'main') {
            // Main pass renders to screen
            colorTargets.push({
                format: this.canvasFormat,
                blend: this._createBlendState(passConfig),
            });
        } else if (passConfig.output_textures && passConfig.output_textures.length > 0) {
            // Other passes render to textures
            for (const outputTexture of passConfig.output_textures) {
                colorTargets.push({
                    format: outputTexture.format || 'rgba8unorm',
                    blend: this._createBlendState(outputTexture.blend),
                });
            }
        } else {
            // Fallback to default format if no outputs specified
            colorTargets.push({
                format: 'rgba8unorm',
                blend: this._createBlendState(),
            });
        }

        return colorTargets;
    }

    /**
     * Create pass textures for intermediate render passes
     * @param {Object} passConfig - Render pass configuration
     * @private
     */
    _createPassTextures(passConfig) {
        if (!passConfig.output_textures || passConfig.output_textures.length === 0) {
            return;
        }

        for (const textureConfig of passConfig.output_textures) {
            const width = Math.max(
                1,
                Math.floor(this.canvas.width * (textureConfig.width_scale || 1.0)),
            );
            const height = Math.max(
                1,
                Math.floor(this.canvas.height * (textureConfig.height_scale || 1.0)),
            );
            const format = textureConfig.format || 'rgba8unorm';

            const texture = this.device.createTexture({
                label: `Pass Texture ${passConfig.id}_${textureConfig.id}`,
                size: [width, height, 1],
                format: format,
                usage:
                    GPUTextureUsage.TEXTURE_BINDING |
                    GPUTextureUsage.RENDER_ATTACHMENT |
                    GPUTextureUsage.COPY_SRC,
            });

            const sampler = this.device.createSampler({
                magFilter: textureConfig.sampler_config?.mag_filter || 'linear',
                minFilter: textureConfig.sampler_config?.min_filter || 'linear',
                mipmapFilter: 'linear',
                addressModeU: textureConfig.sampler_config?.address_mode_u || 'clamp-to-edge',
                addressModeV: textureConfig.sampler_config?.address_mode_v || 'clamp-to-edge',
                addressModeW: 'clamp-to-edge',
            });

            this.passTextures.set(`${passConfig.id}_${textureConfig.id}`, {
                texture,
                sampler,
                width,
                height,
                format,
                config: textureConfig,
            });
        }
    }

    /**
     * Create blend state from configuration
     * @param {Object} blendConfig - Blend configuration or null for default
     * @returns {Object} Blend state configuration
     * @private
     */
    _createBlendState(blendConfig = null) {
        if (!blendConfig) {
            return {
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
            };
        }

        return {
            color: {
                srcFactor: blendConfig.src_factor || 'src-alpha',
                dstFactor: blendConfig.dst_factor || 'one-minus-src-alpha',
                operation: blendConfig.operation || 'add',
            },
            alpha: {
                srcFactor: blendConfig.src_factor || 'one',
                dstFactor: blendConfig.dst_factor || 'one-minus-src-alpha',
                operation: blendConfig.operation || 'add',
            },
        };
    }

    /**
     * Create bind groups
     * @private
     */
    _createBindGroups() {
        for (const [passId, pipelineInfo] of this.renderPipelines.entries()) {
            const passConfig = pipelineInfo.config;
            const bindGroups = [];

            const timeResolutionBindGroup = this.device.createBindGroup({
                label: `Time and Resolution Bind Group for ${passId}`,
                layout: this.bindGroupLayouts.get(`${passId}_timeResolution`),
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

            bindGroups.push(timeResolutionBindGroup);

            // Create bind group for pass input textures
            if (passConfig.input_textures && passConfig.input_textures.length > 0) {
                const textureEntries = [];
                let hasValidTextures = false;

                for (const inputTexture of passConfig.input_textures) {
                    // Get input texture
                    let textureInfo = null;

                    // Check if it's a texture from another pass
                    if (inputTexture.texture_id.includes('_')) {
                        // Directly find the specified texture ID (passID_textureID)
                        textureInfo = this.passTextures.get(inputTexture.texture_id);
                    } else if (inputTexture.texture_id === 'previous') {
                        // Special case: previous pass output
                        // Get all pass IDs, sorted in order
                        const passIds = Array.from(this.renderPipelines.keys());
                        const currentIndex = passIds.indexOf(passId);

                        if (currentIndex > 0) {
                            const previousPassId = passIds[currentIndex - 1];
                            // Find the first output texture of the previous pass
                            const previousPassInfo = this.renderPipelines.get(previousPassId);

                            if (
                                previousPassInfo &&
                                previousPassInfo.config.output_textures &&
                                previousPassInfo.config.output_textures.length > 0
                            ) {
                                const outputId = previousPassInfo.config.output_textures[0].id;
                                const textureKey = `${previousPassId}_${outputId}`;
                                textureInfo = this.passTextures.get(textureKey);

                                if (!textureInfo) {
                                    console.warn(
                                        `Previous pass texture ${textureKey} not found for pass ${passId}`,
                                    );
                                }
                            }
                        }
                    } else {
                        // Check if it's a resource texture
                        const resourceId = parseInt(inputTexture.texture_id);
                        if (!isNaN(resourceId)) {
                            const resourceTexture = this.resourceManager.getTexture(resourceId);
                            if (resourceTexture) {
                                textureInfo = {
                                    texture: resourceTexture.texture,
                                    sampler: this.resourceManager.getSampler(
                                        `${resourceId}_default`,
                                    ),
                                };
                            }
                        }
                    }

                    if (!textureInfo || !textureInfo.texture) {
                        console.warn(
                            `Input texture ${inputTexture.texture_id} not found or invalid for pass ${passId}`,
                        );
                        continue;
                    }

                    try {
                        // Add texture binding
                        textureEntries.push({
                            binding: inputTexture.binding,
                            resource: textureInfo.texture.createView(),
                        });

                        // Add sampler binding
                        textureEntries.push({
                            binding: inputTexture.binding + 1,
                            resource: textureInfo.sampler,
                        });

                        hasValidTextures = true;
                    } catch (e) {
                        console.error(
                            `Error creating texture binding for ${inputTexture.texture_id}:`,
                            e,
                        );
                    }
                }

                // Only create texture bind group if there are valid texture bindings
                if (hasValidTextures && textureEntries.length > 0) {
                    try {
                        const textureBindGroup = this.device.createBindGroup({
                            label: `Texture Bind Group for ${passId}`,
                            layout: this.bindGroupLayouts.get(`${passId}_textures`),
                            entries: textureEntries,
                        });

                        bindGroups.push(textureBindGroup);
                    } catch (e) {
                        console.error(`Error creating texture bind group for pass ${passId}:`, e);
                    }
                }
            }

            // Create user-defined resource bind groups
            if (
                this.config.pipeline.resource_bindings &&
                this.config.pipeline.resource_bindings.length > 0
            ) {
                // Group by bind group
                const bindingsByGroup = {};

                for (const binding of this.config.pipeline.resource_bindings) {
                    if (!bindingsByGroup[binding.group]) {
                        bindingsByGroup[binding.group] = [];
                    }
                    bindingsByGroup[binding.group].push(binding);
                }

                // Create bind groups for each bind group
                for (const [groupIndex, bindings] of Object.entries(bindingsByGroup)) {
                    const entries = [];
                    let hasValidResources = true;

                    for (const binding of bindings) {
                        const resourceId = this.config.resource_ids[binding.resource_index];
                        let resource = null;

                        try {
                            switch (binding.binding_type) {
                                case 'uniform':
                                case 'storage':
                                    const buffer = this.resourceManager.getBuffer(resourceId);
                                    if (buffer && buffer.buffer) {
                                        resource = { buffer: buffer.buffer };
                                    }
                                    break;
                                case 'texture':
                                    const texture = this.resourceManager.getTexture(resourceId);
                                    if (texture && texture.texture) {
                                        resource = { textureView: texture.texture.createView() };
                                    }
                                    break;
                                case 'sampler':
                                    const sampler =
                                        this.resourceManager.getSampler(resourceId) ||
                                        this.resourceManager.getSampler(`${resourceId}_default`);
                                    resource = sampler;
                                    break;
                            }

                            if (!resource) {
                                console.warn(
                                    `Resource ${resourceId} not found for binding ${binding.binding} in group ${groupIndex}`,
                                );
                                hasValidResources = false;
                                continue;
                            }

                            entries.push({
                                binding: binding.binding,
                                resource,
                            });
                        } catch (e) {
                            console.error(`Error creating resource binding for ${resourceId}:`, e);
                            hasValidResources = false;
                        }
                    }

                    // Only create resource bind group if all resources are valid
                    if (hasValidResources && entries.length > 0) {
                        try {
                            const layoutKey = `${passId}_resources_${groupIndex}`;
                            const layout = this.bindGroupLayouts.get(layoutKey);

                            if (!layout) {
                                console.warn(`Bind group layout ${layoutKey} not found`);
                                continue;
                            }

                            const resourceBindGroup = this.device.createBindGroup({
                                label: `Resource Bind Group ${groupIndex} for ${passId}`,
                                layout: layout,
                                entries,
                            });

                            bindGroups.push(resourceBindGroup);
                        } catch (e) {
                            console.error(
                                `Error creating resource bind group ${groupIndex} for pass ${passId}:`,
                                e,
                            );
                        }
                    }
                }
            }

            // Store bind groups
            pipelineInfo.bindGroups = bindGroups;
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

        this.config = { ...this.config, ...config };

        if (config.width && config.height) {
            this.resize(config.width, config.height);
        }

        if (needsRebuild) {
            this._createRenderPipeline();
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

        // Create command encoder
        const encoder = this.device.createCommandEncoder();

        // Execute all render passes
        for (const [passId, pipelineInfo] of this.renderPipelines.entries()) {
            this._executeRenderPass(encoder, passId, pipelineInfo);
        }

        // Submit command buffer
        this.device.queue.submit([encoder.finish()]);

        // Update statistics
        this._updateStats(delta, this.renderPipelines.size);

        // Update frame count
        this.frameCount++;
    }

    /**
     * Execute a render pass
     * @param {GPUCommandEncoder} encoder - Command encoder
     * @param {string} passId - Pass ID
     * @param {Object} pipelineInfo - Pipeline info
     * @private
     */
    _executeRenderPass(encoder, passId, pipelineInfo) {
        const { config: passConfig } = pipelineInfo;

        // Create render pass descriptor
        const renderPassDescriptor = {
            colorAttachments: [],
        };

        // Configure color attachments
        if (passConfig.pass_type === 'main') {
            // Get current frame texture view
            try {
                const textureView = this.context.getCurrentTexture().createView();

                // Main pass renders to screen
                renderPassDescriptor.colorAttachments.push({
                    view: textureView,
                    clearValue: {
                        r: passConfig.clear_color[0],
                        g: passConfig.clear_color[1],
                        b: passConfig.clear_color[2],
                        a: passConfig.clear_color[3],
                    },
                    loadOp: 'clear',
                    storeOp: 'store',
                });
            } catch (e) {
                console.error('Error getting current texture view:', e);
                return;
            }
        } else if (passConfig.output_textures?.length > 0) {
            // Other passes render to texture
            for (const outputTexture of passConfig.output_textures) {
                const textureKey = `${passId}_${outputTexture.id}`;
                const textureInfo = this.passTextures.get(textureKey);

                if (!textureInfo) {
                    console.warn(`Output texture ${textureKey} not found for pass ${passId}`);
                    continue;
                }

                try {
                    renderPassDescriptor.colorAttachments.push({
                        view: textureInfo.texture.createView(),
                        clearValue: {
                            r: passConfig.clear_color[0],
                            g: passConfig.clear_color[1],
                            b: passConfig.clear_color[2],
                            a: passConfig.clear_color[3],
                        },
                        loadOp: 'clear',
                        storeOp: 'store',
                    });
                } catch (e) {
                    console.error(`Error creating view for texture ${textureKey}:`, e);
                    continue;
                }
            }
        } else {
            console.warn(`No output configured for pass ${passId}`);
            return;
        }

        // If there are no valid color attachments, skip this pass
        if (renderPassDescriptor.colorAttachments.length === 0) {
            console.warn(`No valid color attachments for pass ${passId}`);
            return;
        }

        // Add depth attachment (if needed)
        if (passConfig.depth_enabled) {
            if (
                !this.depthTexture ||
                this.depthTexture.width !== this.canvas.width ||
                this.depthTexture.height !== this.canvas.height
            ) {
                this.depthTexture?.destroy();

                this.depthTexture = this.device.createTexture({
                    label: 'Depth Texture',
                    size: [this.canvas.width, this.canvas.height],
                    format: 'depth24plus',
                    usage: GPUTextureUsage.RENDER_ATTACHMENT,
                });
            }

            renderPassDescriptor.depthStencilAttachment = {
                view: this.depthTexture.createView(),
                depthClearValue: passConfig.clear_depth,
                depthLoadOp: 'clear',
                depthStoreOp: 'store',
            };
        }

        try {
            const pass = encoder.beginRenderPass(renderPassDescriptor);

            // Set pipeline
            pass.setPipeline(pipelineInfo.pipeline);

            // Set vertex buffer
            pass.setVertexBuffer(0, this.vertexBuffers.get('position'));

            // Set index buffer
            pass.setIndexBuffer(this.indexBuffer, 'uint16');

            // Set bind groups
            for (let i = 0; i < pipelineInfo.bindGroups.length; i++) {
                pass.setBindGroup(i, pipelineInfo.bindGroups[i]);
            }

            // Draw based on geometry configuration
            if (passConfig.geometry) {
                switch (passConfig.geometry.type) {
                    case 'indexed':
                        pass.drawIndexed(
                            passConfig.geometry.index_count ?? this.indexCount,
                            passConfig.geometry.instance_count ?? 1,
                            0,
                            0,
                            0,
                        );
                        break;
                    case 'nonindexed':
                        pass.draw(
                            passConfig.geometry.vertex_count,
                            passConfig.geometry.instance_count ?? 1,
                            0,
                            0,
                        );
                        break;
                    default:
                        // Default draw whole quad
                        pass.drawIndexed(this.indexCount);
                        break;
                }
            } else {
                // Default draw whole quad
                pass.drawIndexed(this.indexCount);
            }

            // End render pass
            pass.end();
        } catch (e) {
            console.error(`Error executing render pass ${passId}:`, e);
        }
    }

    /**
     * Update performance statistics
     * @param {number} delta - Frame time (in seconds)
     * @param {number} passCount - Number of render passes
     * @private
     */
    _updateStats(delta, passCount = 1) {
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
        this.stats.drawCalls = passCount;
        this.stats.triangleCount = (this.indexCount / 3) * passCount;

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
     * @param {number} width - Width
     * @param {number} height - Height
     */
    resize(width, height) {
        this.canvas.width = width;
        this.canvas.height = height;

        // Update resolution uniform buffer
        const resolutionData = new Float32Array([width, height, width / height, 0]);
        this.resourceManager.updateBuffer('resolution_uniform', resolutionData, this.device);

        // Update all pass textures
        for (const [textureId, textureInfo] of this.passTextures.entries()) {
            // Calculate new size
            const newWidth = Math.floor(width * textureInfo.config.width_scale);
            const newHeight = Math.floor(height * textureInfo.config.height_scale);

            // Destroy old texture
            textureInfo.texture.destroy();

            // Create new texture
            const newTexture = this.device.createTexture({
                label: `Pass Texture ${textureId}`,
                size: [newWidth, newHeight, 1],
                format: textureInfo.format,
                usage:
                    GPUTextureUsage.TEXTURE_BINDING |
                    GPUTextureUsage.RENDER_ATTACHMENT |
                    GPUTextureUsage.COPY_SRC,
            });

            // Update texture info
            textureInfo.texture = newTexture;
            textureInfo.width = newWidth;
            textureInfo.height = newHeight;
        }

        // Recreate bind groups (since texture views have changed)
        this._createBindGroups();

        // Clear depth texture, it will be recreated on the next frame
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
        // Clear all pass textures
        for (const [, textureInfo] of this.passTextures.entries()) {
            textureInfo.texture.destroy();
        }
        this.passTextures.clear();

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
