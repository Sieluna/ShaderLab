export class RenderPipelineManager {
    /**
     * Create render pipeline manager
     * @param {GPUDevice} device - WebGPU device
     */
    constructor(device) {
        this.device = device;
        this.renderPipelines = new Map();
        this.bindGroupLayouts = new Map();
    }

    /**
     * Create pipelines for all render passes
     * @param {Object} config - Render configuration
     * @param {ResourceManager} resourceManager - Resource manager
     * @param {UniformManager} uniformManager - Uniform manager
     * @param {Map<string, Object>} passTextures - Pass texture mapping
     * @param {string} canvasFormat - Canvas format
     * @returns {Map<string, Object>} Render pipeline mapping
     */
    createRenderPipelines(config, resourceManager, uniformManager, passTextures, canvasFormat) {
        // Clear previous pipelines
        this.renderPipelines.clear();
        this.bindGroupLayouts.clear();

        // If no render passes are defined, create default main pass
        if (!config.pipeline.render_passes || config.pipeline.render_passes.length === 0) {
            config.pipeline.render_passes = [
                {
                    id: 'main',
                    pass_type: 'main',
                    clear_color: [0.0, 0.0, 0.0, 1.0],
                    clear_depth: 1.0,
                    clear_stencil: 0,
                },
            ];
        }

        // Create pipeline for each render pass
        for (const passConfig of config.pipeline.render_passes) {
            this.createPassPipeline(
                passConfig,
                config,
                resourceManager,
                uniformManager,
                passTextures,
                canvasFormat,
            );
        }

        return this.renderPipelines;
    }

    /**
     * Create pipeline for specific render pass
     * @param {Object} passConfig - Pass configuration
     * @param {Object} config - Render configuration
     * @param {ResourceManager} resourceManager - Resource manager
     * @param {UniformManager} uniformManager - Uniform manager
     * @param {Map<string, Object>} passTextures - Pass texture mapping
     * @param {string} canvasFormat - Canvas format
     */
    createPassPipeline(
        passConfig,
        config,
        resourceManager,
        uniformManager,
        passTextures,
        canvasFormat,
    ) {
        // Get all shader bindings from config
        const shaderBindings = config.pipeline.shader_bindings;
        let vertexShader = null;
        let fragmentShader = null;
        let vertexShaderId = null;
        let fragmentShaderId = null;

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
                vertexShaderId = config.shader_ids[vertexBinding.shader_index];
                const shader = resourceManager.getShaderModule(vertexShaderId);
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
                fragmentShaderId = config.shader_ids[fragmentBinding.shader_index];
                const shader = resourceManager.getShaderModule(fragmentShaderId);
                if (shader) {
                    fragmentShader = {
                        module: shader.module,
                        entryPoint: fragmentBinding.entry_point,
                    };
                }
            }
        } else {
            // Use default method to find shaders (backwards compatible)
            for (const binding of shaderBindings) {
                const shaderId = config.shader_ids[binding.shader_index];
                const shader = resourceManager.getShaderModule(shaderId);

                if (!shader) {
                    throw new Error(`Shader with ID ${shaderId} not found`);
                }

                if (binding.shader_stage === 'vertex') {
                    vertexShaderId = shaderId;
                    vertexShader = {
                        module: shader.module,
                        entryPoint: binding.entry_point,
                    };
                } else if (binding.shader_stage === 'fragment') {
                    fragmentShaderId = shaderId;
                    fragmentShader = {
                        module: shader.module,
                        entryPoint: binding.entry_point,
                    };
                }
            }
        }

        // Ensure both vertex and fragment shaders exist for non-compute passes
        if (passConfig.pass_type !== 'compute' && (!vertexShader || !fragmentShader)) {
            throw new Error(
                `Both vertex and fragment shaders are required for pass ${passConfig.id}`,
            );
        }

        // Create vertex buffer layouts
        const bufferLayouts = this.#createVertexBufferLayouts();

        // Create binding group layouts
        const bindGroupLayouts = this.#createBindGroupLayouts(
            passConfig,
            config,
            vertexShaderId,
            fragmentShaderId,
            uniformManager,
        );

        // Create pipeline layout
        const pipelineLayout = this.device.createPipelineLayout({
            label: `Render Pipeline Layout for ${passConfig.id}`,
            bindGroupLayouts,
        });

        // Create output textures for intermediate passes
        if (
            (passConfig.pass_type === 'intermediate' || passConfig.pass_type === 'postprocess') &&
            passConfig.output_textures?.length > 0
        ) {
            this.#createPassTextures(passConfig, passTextures, config.width, config.height);
        }

        // Determine color target formats
        const colorTargets = this.#createColorTargets(passConfig, canvasFormat);

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

        // Add depth-stencil state if depth testing is enabled
        if (passConfig.depth_enabled) {
            pipelineDescriptor.depthStencil = {
                format: 'depth24plus',
                depthWriteEnabled: true,
                depthCompare: 'less',
            };
        }

        // Create render pipeline
        const pipeline = this.device.createRenderPipeline(pipelineDescriptor);

        // Store pipeline and related information
        this.renderPipelines.set(passConfig.id, {
            pipeline,
            config: passConfig,
            layout: pipelineLayout,
            bindGroupLayouts,
            colorTargets,
            bindGroups: [], // This will be filled by BindGroupManager
        });
    }

    /**
     * Create vertex buffer layouts
     * @private
     * @returns {Array<Object>} Vertex buffer layout array
     */
    #createVertexBufferLayouts() {
        const bufferLayouts = [];

        // Add position and UV layout
        bufferLayouts.push({
            arrayStride: 5 * 4, // 5 floats (3 position + 2 UV) * 4 bytes
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

        return bufferLayouts;
    }

    /**
     * Create binding group layouts
     * @param {Object} passConfig - Pass configuration
     * @param {Object} config - Render configuration
     * @param {number} vertexShaderId - Vertex shader ID
     * @param {number} fragmentShaderId - Fragment shader ID
     * @param {UniformManager} uniformManager - Uniform manager
     * @returns {Array<GPUBindGroupLayout>} Binding group layout array
     * @private
     */
    #createBindGroupLayouts(passConfig, config, vertexShaderId, fragmentShaderId, uniformManager) {
        const bindGroupLayouts = [];
        const passId = passConfig.id;

        // Try to get binding group layout from UniformManager
        let foundLayout = false;

        if (vertexShaderId && fragmentShaderId) {
            const passVertexShaderKey = `pass_${passId}_shader_${vertexShaderId}`;
            const passFragmentShaderKey = `pass_${passId}_shader_${fragmentShaderId}`;

            // First check if fragment shader has group 0 binding group layout
            const fragmentLayout = uniformManager.getBindGroupLayout(passFragmentShaderKey, 0);
            if (fragmentLayout) {
                bindGroupLayouts.push(fragmentLayout);
                foundLayout = true;
                // Also record this layout in manager for later use
                this.bindGroupLayouts.set(`${passId}_timeResolution`, fragmentLayout);
            } else {
                // Try to use vertex shader's group 0 binding group layout
                const vertexLayout = uniformManager.getBindGroupLayout(passVertexShaderKey, 0);
                if (vertexLayout) {
                    bindGroupLayouts.push(vertexLayout);
                    foundLayout = true;
                    // Also record this layout in manager
                    this.bindGroupLayouts.set(`${passId}_timeResolution`, vertexLayout);
                }
            }
        }

        // If not found, create default time and resolution binding group layout
        if (!foundLayout) {
            const timeResolutionLayout = this.#createTimeResolutionBindGroupLayout(passId, config);
            bindGroupLayouts.push(timeResolutionLayout);
        }

        // Add input texture binding group layout
        if (passConfig.input_textures?.length > 0) {
            const textureLayout = this.#createTextureBindGroupLayout(passConfig);
            if (textureLayout) {
                bindGroupLayouts.push(textureLayout);
            }
        }

        // Add resource binding group layouts
        if (config.pipeline.resource_bindings?.length > 0) {
            const resourceLayouts = this.#createResourceBindGroupLayouts(passId, config);
            bindGroupLayouts.push(...resourceLayouts);
        }

        return bindGroupLayouts;
    }

    /**
     * Create time and resolution uniform buffer binding group layout
     * @param {string} passId - Pass ID
     * @param {Object} config - Render configuration
     * @returns {GPUBindGroupLayout} Binding group layout
     * @private
     */
    #createTimeResolutionBindGroupLayout(passId, config) {
        // Create basic bindings array
        const entries = [
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
        ];

        // Add all known custom uniform values
        if (config.uniforms && Array.isArray(config.uniforms)) {
            for (let i = 0; i < config.uniforms.length; i++) {
                const uniform = config.uniforms[i];
                if (uniform.name) {
                    entries.push({
                        binding: i + 2, // Start from binding 2
                        visibility: GPUShaderStage.FRAGMENT,
                        buffer: { type: 'uniform' },
                    });
                }
            }
        }

        const layout = this.device.createBindGroupLayout({
            label: `Time and Resolution Bind Group Layout for ${passId}`,
            entries: entries,
        });

        this.bindGroupLayouts.set(`${passId}_timeResolution`, layout);
        return layout;
    }

    /**
     * Create input texture binding group layout
     * @param {Object} passConfig - Pass configuration
     * @returns {GPUBindGroupLayout|null} Created binding group layout or null (if no textures)
     * @private
     */
    #createTextureBindGroupLayout(passConfig) {
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

            // Sampler binding (usually next binding)
            textureLayoutEntries.push({
                binding: inputTexture.binding + 1,
                visibility: GPUShaderStage.FRAGMENT,
                sampler: {},
            });
        }

        // Create texture binding group layout
        const layout = this.device.createBindGroupLayout({
            label: `Texture Bind Group Layout for ${passConfig.id}`,
            entries: textureLayoutEntries,
        });

        this.bindGroupLayouts.set(`${passConfig.id}_textures`, layout);
        return layout;
    }

    /**
     * Create resource binding group layouts
     * @param {string} passId - Pass ID
     * @param {Object} config - Render configuration
     * @returns {Array<GPUBindGroupLayout>} Created binding group layout array
     * @private
     */
    #createResourceBindGroupLayouts(passId, config) {
        const layouts = [];
        const bindingsByGroup = {};

        // Group bindings by group index
        for (const binding of config.pipeline.resource_bindings) {
            if (!bindingsByGroup[binding.group]) {
                bindingsByGroup[binding.group] = [];
            }
            bindingsByGroup[binding.group].push(binding);
        }

        // Create layout for each binding group
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
     * Create render pass color targets
     * @param {Object} passConfig - Pass configuration
     * @param {string} canvasFormat - Canvas format
     * @returns {Array<Object>} Color target array
     * @private
     */
    #createColorTargets(passConfig, canvasFormat) {
        const colorTargets = [];

        if (passConfig.pass_type === 'main') {
            // Main pass renders to screen
            colorTargets.push({
                format: canvasFormat,
                blend: this.#createBlendState(passConfig),
            });
        } else if (passConfig.output_textures && passConfig.output_textures.length > 0) {
            // Other passes render to textures
            for (const outputTexture of passConfig.output_textures) {
                colorTargets.push({
                    format: outputTexture.format || 'rgba8unorm',
                    blend: this.#createBlendState(outputTexture.blend),
                });
            }
        } else {
            // If no output specified, use default format
            colorTargets.push({
                format: 'rgba8unorm',
                blend: this.#createBlendState(),
            });
        }

        return colorTargets;
    }

    /**
     * Create blend state
     * @param {Object} blendConfig - Blend configuration or null for defaults
     * @returns {Object} Blend state configuration
     * @private
     */
    #createBlendState(blendConfig = null) {
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
     * Create/update pass textures
     * @param {Object} passConfig - Pass configuration
     * @param {Map<string, Object>} passTextures - Pass texture mapping
     * @param {number} canvasWidth - Canvas width
     * @param {number} canvasHeight - Canvas height
     * @private
     */
    #createPassTextures(passConfig, passTextures, canvasWidth, canvasHeight) {
        if (!passConfig.output_textures || passConfig.output_textures.length === 0) {
            return;
        }

        for (const textureConfig of passConfig.output_textures) {
            const width = Math.max(1, Math.floor(canvasWidth * (textureConfig.width_scale ?? 1.0)));
            const height = Math.max(
                1,
                Math.floor(canvasHeight * (textureConfig.height_scale ?? 1.0)),
            );
            const format = textureConfig.format || 'rgba8unorm';

            // Try to find existing texture
            const textureKey = `${passConfig.id}_${textureConfig.id}`;
            const existingTexture = passTextures.get(textureKey);

            // If texture already exists and size is the same, skip
            if (
                existingTexture &&
                existingTexture.width === width &&
                existingTexture.height === height
            ) {
                continue;
            }

            // If exists but size is different, destroy old texture
            if (existingTexture) {
                existingTexture.texture.destroy();
            }

            const texture = this.device.createTexture({
                label: `Pass Texture ${textureKey}`,
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

            passTextures.set(textureKey, {
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
     * Rebuild pipelines and textures after resize
     * @param {number} width - New width
     * @param {number} height - New height
     * @param {Map<string, Object>} passTextures - Pass texture mapping
     */
    resizePipelines(width, height, passTextures) {
        // Update all pass textures
        for (const [passId, pipelineInfo] of this.renderPipelines.entries()) {
            const passConfig = pipelineInfo.config;
            if (passConfig.pass_type === 'intermediate' || passConfig.pass_type === 'postprocess') {
                this.#createPassTextures(passConfig, passTextures, width, height);
            }
        }
    }

    /**
     * Get binding group layout for specified pass
     * @param {string} passId - Pass ID
     * @param {string} layoutType - Layout type (e.g. 'timeResolution', 'textures', 'resources')
     * @param {number} [groupIndex] - Resource group index (for resources type)
     * @returns {GPUBindGroupLayout|null} Binding group layout or null
     */
    getBindGroupLayout(passId, layoutType, groupIndex) {
        if (layoutType === 'resources' && groupIndex !== undefined) {
            return this.bindGroupLayouts.get(`${passId}_${layoutType}_${groupIndex}`);
        }
        return this.bindGroupLayouts.get(`${passId}_${layoutType}`);
    }
}
