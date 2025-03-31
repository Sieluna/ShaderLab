export class ResourceManager {
    shaders = new Map(); // Shader resources
    textures = new Map(); // Texture resources
    buffers = new Map(); // Buffer resources
    samplers = new Map(); // Sampler resources
    shaderModules = new Map(); // Shader modules

    /**
     * Load resource
     * @param {Object} resource - Resource object
     * @param {GPUDevice} device - WebGPU device
     * @returns {Promise<void>}
     */
    async loadResource(resource, device) {
        try {
            // Handle based on resource type
            switch (resource.resource_type.toLowerCase()) {
                case 'texture':
                case 'image':
                    await this._loadTexture(resource, device);
                    break;
                case 'buffer':
                case 'data':
                    this._loadBuffer(resource, device);
                    break;
                default:
                    console.warn(`Unknown resource type: ${resource.resource_type}`);
            }
        } catch (error) {
            console.error(`Failed to load resource ${resource.name}:`, error);
            throw error;
        }
    }

    /**
     * Load shader
     * @param {Object} shader - Shader object
     * @param {GPUDevice} device - WebGPU device
     * @returns {Promise<void>}
     */
    async loadShader(shader, device) {
        try {
            // Create shader module
            const shaderModule = device.createShaderModule({
                code: shader.code,
                label: `Shader-${shader.name}`,
            });

            // Store shader module
            this.shaderModules.set(shader.id, {
                module: shaderModule,
                type: shader.shader_type,
                code: shader.code,
            });

            // Store shader reference in shaders Map
            this.shaders.set(shader.id, {
                id: shader.id,
                name: shader.name,
                type: shader.shader_type,
            });
        } catch (error) {
            console.error(`Failed to load shader ${shader.name}:`, error);
            throw error;
        }
    }

    /**
     * Load texture resource
     * @param {Object} resource - Resource object
     * @param {GPUDevice} device - WebGPU device
     * @returns {Promise<void>}
     * @private
     */
    async _loadTexture(resource, device) {
        // Create image from resource data
        const blob = new Blob([new Uint8Array(resource.data)]);
        const imageUrl = URL.createObjectURL(blob);
        const image = new Image();

        // Wait for image load
        await new Promise((resolve, reject) => {
            image.onload = resolve;
            image.onerror = reject;
            image.src = imageUrl;
        });

        // Release URL
        URL.revokeObjectURL(imageUrl);

        // Create ImageBitmap
        const imageBitmap = await createImageBitmap(image);

        // Create texture
        const texture = device.createTexture({
            label: `Texture-${resource.name}`,
            size: [imageBitmap.width, imageBitmap.height, 1],
            format: 'rgba8unorm',
            usage:
                GPUTextureUsage.TEXTURE_BINDING |
                GPUTextureUsage.COPY_DST |
                GPUTextureUsage.RENDER_ATTACHMENT,
        });

        // Copy ImageBitmap to texture
        device.queue.copyExternalImageToTexture({ source: imageBitmap }, { texture }, [
            imageBitmap.width,
            imageBitmap.height,
        ]);

        // Store texture
        this.textures.set(resource.id, {
            texture,
            width: imageBitmap.width,
            height: imageBitmap.height,
            format: 'rgba8unorm',
        });

        // Create default sampler for this texture
        const sampler = device.createSampler({
            magFilter: 'linear',
            minFilter: 'linear',
            mipmapFilter: 'linear',
            addressModeU: 'repeat',
            addressModeV: 'repeat',
            addressModeW: 'repeat',
        });

        this.samplers.set(`${resource.id}_default`, sampler);
    }

    /**
     * Load buffer resource
     * @param {Object} resource - Resource object
     * @param {GPUDevice} device - WebGPU device
     * @private
     */
    _loadBuffer(resource, device) {
        // Create resource buffer
        const buffer = device.createBuffer({
            label: `Buffer-${resource.name}`,
            size: resource.data.length,
            usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_DST | GPUBufferUsage.COPY_SRC,
            mappedAtCreation: true,
        });

        // Write data to buffer
        new Uint8Array(buffer.getMappedRange()).set(new Uint8Array(resource.data));

        // Commit buffer
        buffer.unmap();

        // Store buffer
        this.buffers.set(resource.id, {
            buffer,
            size: resource.data.length,
            type: resource.metadata?.type ?? 'raw',
        });
    }

    /**
     * Create vertex buffer
     * @param {String} id - Buffer ID
     * @param {Float32Array|Uint16Array} data - Vertex data
     * @param {GPUDevice} device - WebGPU device
     * @param {GPUBufferUsage} usage - Buffer usage
     * @returns {GPUBuffer} Created buffer
     */
    createVertexBuffer(id, data, device, usage = GPUBufferUsage.VERTEX) {
        // Create vertex buffer
        const buffer = device.createBuffer({
            label: `VertexBuffer-${id}`,
            size: data.byteLength,
            usage: usage | GPUBufferUsage.COPY_DST,
            mappedAtCreation: true,
        });

        // Write data to buffer
        const writeArray =
            data instanceof Float32Array
                ? new Float32Array(buffer.getMappedRange())
                : new Uint16Array(buffer.getMappedRange());

        writeArray.set(data);

        // Commit buffer
        buffer.unmap();

        // Store buffer
        this.buffers.set(id, {
            buffer,
            size: data.byteLength,
            type: data instanceof Float32Array ? 'float32' : 'uint16',
        });

        return buffer;
    }

    /**
     * Create uniform buffer
     * @param {String} id - Buffer ID
     * @param {Float32Array|Uint8Array} data - Buffer data
     * @param {GPUDevice} device - WebGPU device
     * @returns {GPUBuffer} Created buffer
     */
    createUniformBuffer(id, data, device) {
        // Calculate aligned size (must be multiple of 256)
        const alignedSize = Math.ceil(data.byteLength / 256) * 256;

        // Create uniform buffer
        const buffer = device.createBuffer({
            label: `UniformBuffer-${id}`,
            size: alignedSize,
            usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
            mappedAtCreation: true,
        });

        // Write data to buffer
        const writeArray =
            data instanceof Float32Array
                ? new Float32Array(buffer.getMappedRange())
                : new Uint8Array(buffer.getMappedRange());

        writeArray.set(data);

        // Commit buffer
        buffer.unmap();

        // Store buffer
        this.buffers.set(id, {
            buffer,
            size: alignedSize,
            type: 'uniform',
        });

        return buffer;
    }

    /**
     * Update buffer data
     * @param {String} id - Buffer ID
     * @param {TypedArray} data - New data
     * @param {GPUDevice} device - WebGPU device
     * @param {Number} offset - Update offset
     */
    updateBuffer(id, data, device, offset = 0) {
        const bufferInfo = this.buffers.get(id);
        if (!bufferInfo) {
            console.warn(`Buffer ${id} not found`);
            return;
        }

        // Write data to buffer
        device.queue.writeBuffer(
            bufferInfo.buffer,
            offset,
            data.buffer,
            data.byteOffset,
            data.byteLength,
        );
    }

    /**
     * Get shader module
     * @param {Number} id - Shader ID
     * @returns {Object|null} Shader module info
     */
    getShaderModule(id) {
        return this.shaderModules.get(id) ?? null;
    }

    /**
     * Get texture
     * @param {Number} id - Texture ID
     * @returns {Object|null} Texture info
     */
    getTexture(id) {
        return this.textures.get(id) ?? null;
    }

    /**
     * Get buffer
     * @param {String} id - Buffer ID
     * @returns {Object|null} Buffer info
     */
    getBuffer(id) {
        return this.buffers.get(id) ?? null;
    }

    /**
     * Get sampler
     * @param {String} id - Sampler ID
     * @returns {GPUSampler|null} Sampler
     */
    getSampler(id) {
        return this.samplers.get(id) ?? null;
    }

    /**
     * Get resource statistics
     * @returns {Object} Resource statistics
     */
    getResourceCount() {
        return {
            shaders: this.shaders.size,
            textures: this.textures.size,
            buffers: this.buffers.size,
            samplers: this.samplers.size,
        };
    }

    /**
     * Release specific resource
     * @param {String} type - Resource type
     * @param {Number|String} id - Resource ID
     */
    release(type, id) {
        switch (type) {
            case 'shader': {
                const shaderModule = this.shaderModules.get(id);
                if (shaderModule) {
                    this.shaderModules.delete(id);
                    this.shaders.delete(id);
                }
                break;
            }
            case 'texture': {
                const texture = this.textures.get(id);
                if (texture) {
                    texture.texture.destroy();
                    this.textures.delete(id);
                    // Remove related sampler
                    this.samplers.delete(`${id}_default`);
                }
                break;
            }
            case 'buffer': {
                const buffer = this.buffers.get(id);
                if (buffer) {
                    buffer.buffer.destroy();
                    this.buffers.delete(id);
                }
                break;
            }
            case 'sampler':
                this.samplers.delete(id);
                break;
            default:
                console.warn(`Unknown resource type: ${type}`);
        }
    }

    /**
     * Release all resources
     */
    releaseAll() {
        // Release textures
        for (const [, texture] of this.textures) {
            texture.texture.destroy();
        }
        this.textures.clear();

        // Release buffers
        for (const [, buffer] of this.buffers) {
            buffer.buffer.destroy();
        }
        this.buffers.clear();

        // Clear shaders
        this.shaderModules.clear();
        this.shaders.clear();

        // Clear samplers
        this.samplers.clear();
    }
}
