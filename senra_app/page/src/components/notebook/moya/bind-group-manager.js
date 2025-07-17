/**
 * Binding Group Manager
 * Responsible for creating and managing WebGPU binding groups
 */
export class BindGroupManager {
    /**
     * Create binding group manager
     * @param {GPUDevice} device - WebGPU device
     * @param {ResourceManager} resourceManager - Resource manager
     * @param {UniformManager} uniformManager - Uniform manager
     */
    constructor(device, resourceManager, uniformManager) {
        this.device = device;
        this.resourceManager = resourceManager;
        this.uniformManager = uniformManager;
    }

    /**
     * Create binding groups for all render passes
     * @param {Map<string, Object>} renderPipelines - Render pipeline mapping
     * @param {Object} config - Render configuration
     * @param {Map<string, GPUBuffer>} uniformBuffers - Uniform buffer mapping
     * @param {Map<string, Object>} passTextures - Pass texture mapping
     */
    createBindGroups(renderPipelines, config, uniformBuffers, passTextures) {
        for (const [passId, pipelineInfo] of renderPipelines.entries()) {
            const passConfig = pipelineInfo.config;
            const bindGroups = [];

            // Get shader IDs for this pass
            const { vertexShaderId, fragmentShaderId } = this.#getPassShaderIds(passConfig, config);

            // First try to create time/resolution binding group
            const timeResolutionBindGroup = this.#createTimeResolutionBindGroup(
                passId,
                pipelineInfo,
                fragmentShaderId,
                vertexShaderId,
                config,
                uniformBuffers,
            );

            if (timeResolutionBindGroup) {
                bindGroups.push(timeResolutionBindGroup);
            }

            // Create texture binding group (if input textures exist)
            if (passConfig.input_textures?.length > 0) {
                const textureBindGroup = this.#createTextureBindGroup(
                    passId,
                    passConfig,
                    pipelineInfo,
                    passTextures,
                );

                if (textureBindGroup) {
                    bindGroups.push(textureBindGroup);
                }
            }

            // Create resource binding groups (if resource bindings exist)
            if (config.pipeline.resource_bindings?.length > 0) {
                const resourceBindGroups = this.#createResourceBindGroups(
                    passId,
                    pipelineInfo,
                    config,
                );

                bindGroups.push(...resourceBindGroups);
            }

            // Store binding groups
            pipelineInfo.bindGroups = bindGroups;
        }
    }

    /**
     * Get shader IDs for pass
     * @param {Object} passConfig - Pass configuration
     * @param {Object} config - Render configuration
     * @returns {Object} Vertex and fragment shader IDs
     * @private
     */
    #getPassShaderIds(passConfig, config) {
        let vertexShaderId = null;
        let fragmentShaderId = null;

        if (passConfig.shader_bindings && Array.isArray(passConfig.shader_bindings)) {
            for (const bindingIndex of passConfig.shader_bindings) {
                const binding = config.pipeline.shader_bindings[bindingIndex];
                if (!binding) continue;

                const shaderId = config.shader_ids[binding.shader_index];
                if (!shaderId) continue;

                if (binding.shader_stage === 'vertex') {
                    vertexShaderId = shaderId;
                } else if (binding.shader_stage === 'fragment') {
                    fragmentShaderId = shaderId;
                }
            }
        }

        return { vertexShaderId, fragmentShaderId };
    }

    /**
     * Create time and resolution binding group
     * @param {string} passId - Pass ID
     * @param {Object} pipelineInfo - Pipeline information
     * @param {number} fragmentShaderId - Fragment shader ID
     * @param {number} vertexShaderId - Vertex shader ID
     * @param {Object} config - Render configuration
     * @param {Map<string, GPUBuffer>} uniformBuffers - Uniform buffer mapping
     * @returns {GPUBindGroup|null} Created binding group or null
     * @private
     */
    #createTimeResolutionBindGroup(
        passId,
        pipelineInfo,
        fragmentShaderId,
        vertexShaderId,
        config,
        uniformBuffers,
    ) {
        // First try to use UniformManager's binding group
        if (fragmentShaderId) {
            const passFragmentShaderKey = `pass_${passId}_shader_${fragmentShaderId}`;
            const bindGroup = this.uniformManager.getBindGroup(passFragmentShaderKey, 0);

            if (bindGroup) {
                return bindGroup;
            }
        }

        if (vertexShaderId) {
            const passVertexShaderKey = `pass_${passId}_shader_${vertexShaderId}`;
            const bindGroup = this.uniformManager.getBindGroup(passVertexShaderKey, 0);

            if (bindGroup) {
                return bindGroup;
            }
        }

        // If unable to get binding group through UniformManager, use traditional method
        const layout = pipelineInfo.bindGroupLayouts[0]; // First binding group layout should be time/resolution layout
        if (!layout) {
            console.error(`Missing time/resolution bind group layout for pass ${passId}`);
            return null;
        }

        try {
            // Estimate needed entry count (at least time and resolution)
            let entryCount = 2;

            // Create binding entry array, first add time and resolution
            const timeResolutionEntries = [
                {
                    binding: 0,
                    resource: { buffer: uniformBuffers.get('time') },
                },
                {
                    binding: 1,
                    resource: { buffer: uniformBuffers.get('resolution') },
                },
            ];

            // If config.uniforms exists, add more bindings
            if (config.uniforms && Array.isArray(config.uniforms)) {
                for (let i = 0; i < config.uniforms.length; i++) {
                    const uniform = config.uniforms[i];
                    if (uniform.name) {
                        // First check if there is a corresponding buffer
                        let buffer = uniformBuffers.get(uniform.name);

                        if (buffer) {
                            timeResolutionEntries.push({
                                binding: i + 2, // binding starts from 2
                                resource: { buffer: buffer },
                            });

                            // Update count
                            entryCount = Math.max(entryCount, i + 3);
                        }
                    }
                }
            }

            // Check if all necessary binding points are covered
            if (timeResolutionEntries.length < entryCount) {
                console.error(
                    `Missing bindings for pass ${passId}. Have ${timeResolutionEntries.length}, need ${entryCount}`,
                );
                return null;
            }

            // Create binding group
            return this.device.createBindGroup({
                label: `Time and Resolution Bind Group for ${passId}`,
                layout: layout,
                entries: timeResolutionEntries,
            });
        } catch (error) {
            console.error(`Error creating binding group for pass ${passId}:`, error);
            return null;
        }
    }

    /**
     * Create texture binding group
     * @param {string} passId - Pass ID
     * @param {Object} passConfig - Pass configuration
     * @param {Object} pipelineInfo - Pipeline information
     * @param {Map<string, Object>} passTextures - Pass texture mapping
     * @returns {GPUBindGroup|null} Created binding group or null
     * @private
     */
    #createTextureBindGroup(passId, passConfig, pipelineInfo, passTextures) {
        const textureEntries = [];
        let hasValidTextures = false;

        for (const inputTexture of passConfig.input_textures) {
            // Get input texture
            let textureInfo = null;

            // Check if texture is from another pass
            if (inputTexture.texture_id.includes('_')) {
                // Directly look up specified texture ID (passID_textureID)
                textureInfo = passTextures.get(inputTexture.texture_id);
            } else if (inputTexture.texture_id === 'previous') {
                // Special case: output from previous pass
                // Get all pass IDs sorted by order
                const passIds = Array.from(pipelineInfo.renderPipelines.keys());
                const currentIndex = passIds.indexOf(passId);

                if (currentIndex > 0) {
                    const previousPassId = passIds[currentIndex - 1];
                    // Look for first output texture of previous pass
                    const previousPassInfo = pipelineInfo.renderPipelines.get(previousPassId);

                    if (
                        previousPassInfo &&
                        previousPassInfo.config.output_textures &&
                        previousPassInfo.config.output_textures.length > 0
                    ) {
                        const outputId = previousPassInfo.config.output_textures[0].id;
                        const textureKey = `${previousPassId}_${outputId}`;
                        textureInfo = passTextures.get(textureKey);

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
                            sampler: this.resourceManager.getSampler(`${resourceId}_default`),
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
                console.error(`Error creating texture binding for ${inputTexture.texture_id}:`, e);
            }
        }

        // Only create texture binding group when there are valid texture bindings
        if (hasValidTextures && textureEntries.length > 0) {
            try {
                // Find texture binding group layout (should be second binding group layout)
                const textureLayout = pipelineInfo.bindGroupLayouts[1];
                if (!textureLayout) {
                    console.warn(`Texture bind group layout not found for pass ${passId}`);
                    return null;
                }

                return this.device.createBindGroup({
                    label: `Texture Bind Group for ${passId}`,
                    layout: textureLayout,
                    entries: textureEntries,
                });
            } catch (e) {
                console.error(`Error creating texture bind group for pass ${passId}:`, e);
                return null;
            }
        }

        return null;
    }

    /**
     * Create resource binding groups
     * @param {string} passId - Pass ID
     * @param {Object} pipelineInfo - Pipeline information
     * @param {Object} config - Render configuration
     * @returns {Array<GPUBindGroup>} Created binding group array
     * @private
     */
    #createResourceBindGroups(passId, pipelineInfo, config) {
        const resourceBindGroups = [];

        if (!config.pipeline.resource_bindings || config.pipeline.resource_bindings.length === 0) {
            return resourceBindGroups;
        }

        // Group by binding group
        const bindingsByGroup = {};

        for (const binding of config.pipeline.resource_bindings) {
            if (!bindingsByGroup[binding.group]) {
                bindingsByGroup[binding.group] = [];
            }
            bindingsByGroup[binding.group].push(binding);
        }

        // Create binding group for each group
        for (const [groupIndex, bindings] of Object.entries(bindingsByGroup)) {
            const entries = [];
            let hasValidResources = true;

            for (const binding of bindings) {
                const resourceId = config.resource_ids[binding.resource_index];
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
                                resource = texture.texture.createView();
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

            // Only create resource binding group when all resources are valid
            if (hasValidResources && entries.length > 0) {
                try {
                    // Find resource binding group layout
                    const layoutKey = `${passId}_resources_${groupIndex}`;
                    const layout = pipelineInfo.bindGroupLayouts.find((layout) =>
                        layout.label.includes(`Resource Bind Group Layout ${groupIndex}`),
                    );

                    if (!layout) {
                        console.warn(`Bind group layout ${layoutKey} not found`);
                        continue;
                    }

                    const resourceBindGroup = this.device.createBindGroup({
                        label: `Resource Bind Group ${groupIndex} for ${passId}`,
                        layout: layout,
                        entries,
                    });

                    resourceBindGroups.push(resourceBindGroup);
                } catch (e) {
                    console.error(
                        `Error creating resource bind group ${groupIndex} for pass ${passId}:`,
                        e,
                    );
                }
            }
        }

        return resourceBindGroups;
    }

    /**
     * Update uniform value buffer
     * @param {string} name - Uniform value name
     * @param {any} value - New value
     * @param {Map<string, GPUBuffer>} uniformBuffers - Uniform buffer mapping
     * @param {Function} createTypedArrayForValue - Function to create typed array
     */
    updateUniformBuffer(name, value, uniformBuffers, createTypedArrayForValue) {
        const buffer = uniformBuffers.get(name);
        if (!buffer) {
            console.warn(`Uniform buffer for ${name} not found`);
            return;
        }

        // Create typed array
        const typedValue = createTypedArrayForValue(value);
        if (!typedValue) {
            console.warn(`Failed to create typed array for ${name}`);
            return;
        }

        // Use ResourceManager to update buffer
        this.resourceManager.updateBuffer(`custom_uniform_${name}`, typedValue, this.device);

        // Also try to update through UniformManager
        for (const shaderId of this.uniformManager.getShaderIds()) {
            this.uniformManager.updateUniform(shaderId, name, value);
        }
    }
}
