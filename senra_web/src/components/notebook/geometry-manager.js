/**
 * Geometry Manager
 * Responsible for creating and managing render geometry and uniform buffers
 */
export class GeometryManager {
    /**
     * Create a geometry manager
     * @param {GPUDevice} device - WebGPU device
     * @param {ResourceManager} resourceManager - Resource manager
     */
    constructor(device, resourceManager) {
        this.device = device;
        this.resourceManager = resourceManager;

        this.vertexBuffers = new Map();
        this.indexBuffer = null;
        this.indexCount = 0;
        this.uniformBuffers = new Map();
    }

    /**
     * Create default geometry and basic uniform buffers
     * @param {number} canvasWidth - Canvas width
     * @param {number} canvasHeight - Canvas height
     * @param {Object} config - Render configuration
     * @returns {Object} Object containing vertex buffers, index buffer, and uniform buffers
     */
    createDefaultGeometry(canvasWidth, canvasHeight, config) {
        // Create basic quad (square) with position and texture coordinates
        const vertices = new Float32Array([
            // Position (x, y, z)    // UV coordinates (u, v)
            -1.0, -1.0, 0.0,        0.0, 1.0,
             1.0, -1.0, 0.0,        1.0, 1.0,
             1.0,  1.0, 0.0,        1.0, 0.0,
            -1.0,  1.0, 0.0,        0.0, 0.0,
        ]);

        // Quad indices (two triangles)
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

        // Write indices to buffer
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
            canvasWidth,
            canvasHeight,
            canvasWidth / canvasHeight,
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

        // If camera exists in config, initialize camera uniform buffer
        if (config && config.camera) {
            const { position, target, up, fov, near, far } = config.camera;
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

        // Create custom uniform buffers
        this._createCustomUniformBuffers(config);

        return {
            vertexBuffers: this.vertexBuffers,
            indexBuffer: this.indexBuffer,
            indexCount: this.indexCount,
            uniformBuffers: this.uniformBuffers,
        };
    }

    /**
     * Create buffers for custom uniform values defined in render config
     * @param {Object} config - Render configuration
     * @private
     */
    _createCustomUniformBuffers(config) {
        if (!config || !config.uniforms || !Array.isArray(config.uniforms)) {
            return;
        }

        for (const uniform of config.uniforms) {
            if (uniform.name && uniform.default !== undefined) {
                // Create typed array suitable for the value type
                const typedValue = this._createTypedArrayForValue(uniform.default);
                if (typedValue) {
                    // Create uniform buffer
                    const buffer = this.resourceManager.createUniformBuffer(
                        `custom_uniform_${uniform.name}`,
                        typedValue,
                        this.device,
                    );
                    this.uniformBuffers.set(uniform.name, buffer);
                }
            }
        }
    }

    /**
     * Update time uniform buffer
     * @param {number} time - Total time (seconds)
     * @param {number} delta - Frame interval (seconds)
     * @param {number} frameCount - Frame count
     */
    updateTimeUniform(time, delta, frameCount) {
        const timeData = new Float32Array([
            time, // Total time (seconds)
            delta, // Frame interval (seconds)
            frameCount, // Frame count
            0, // Reserved
        ]);

        this.resourceManager.updateBuffer('time_uniform', timeData, this.device);
    }

    /**
     * Update resolution uniform buffer
     * @param {number} width - Width
     * @param {number} height - Height
     */
    updateResolutionUniform(width, height) {
        const resolutionData = new Float32Array([
            width,
            height,
            width / height, // Aspect ratio
            0, // Reserved
        ]);

        this.resourceManager.updateBuffer('resolution_uniform', resolutionData, this.device);
    }

    /**
     * Update custom uniform buffer
     * @param {string} name - Uniform value name
     * @param {any} value - New value
     */
    updateCustomUniform(name, value) {
        const typedValue = this._createTypedArrayForValue(value);
        if (!typedValue) {
            console.warn(`Cannot create typed array for ${name}`);
            return;
        }

        this.resourceManager.updateBuffer(`custom_uniform_${name}`, typedValue, this.device);
    }

    /**
     * Create typed array for specified value type
     * @param {any} value - Value
     * @returns {TypedArray|null} - Typed array suitable for the value
     * @private
     */
    _createTypedArrayForValue(value) {
        if (value === null || value === undefined) {
            return null;
        }

        // Handle different value types
        if (Array.isArray(value)) {
            // Vector type
            const array = new Float32Array(Math.max(4, Math.ceil(value.length / 4) * 4));
            for (let i = 0; i < value.length; i++) {
                array[i] = value[i];
            }
            return array;
        } else if (typeof value === 'number') {
            // Single number
            const array = new Float32Array(4);
            array[0] = value;
            return array;
        } else if (typeof value === 'boolean') {
            // Boolean value
            const array = new Uint32Array(4);
            array[0] = value ? 1 : 0;
            return array;
        }

        return null;
    }

    /**
     * Create custom vertex buffer
     * @param {string} id - Buffer ID
     * @param {Float32Array|Uint16Array} data - Vertex data
     * @param {GPUBufferUsage} usage - Buffer usage
     * @returns {GPUBuffer} Created buffer
     */
    createVertexBuffer(id, data, usage = GPUBufferUsage.VERTEX) {
        const buffer = this.resourceManager.createVertexBuffer(id, data, this.device, usage);
        this.vertexBuffers.set(id, buffer);
        return buffer;
    }

    /**
     * Create custom index buffer
     * @param {string} id - Buffer ID
     * @param {Uint16Array|Uint32Array} data - Index data
     * @returns {GPUBuffer} Created buffer
     */
    createIndexBuffer(id, data) {
        // Determine index format
        const is32Bit = data instanceof Uint32Array;

        // Create index buffer
        const buffer = this.device.createBuffer({
            label: `IndexBuffer-${id}`,
            size: data.byteLength,
            usage: GPUBufferUsage.INDEX | GPUBufferUsage.COPY_DST,
            mappedAtCreation: true,
        });

        // Write data
        if (is32Bit) {
            new Uint32Array(buffer.getMappedRange()).set(data);
        } else {
            new Uint16Array(buffer.getMappedRange()).set(data);
        }

        buffer.unmap();

        // Replace default index buffer
        if (this.indexBuffer) {
            this.indexBuffer.destroy();
        }

        this.indexBuffer = buffer;
        this.indexCount = data.length;

        return buffer;
    }

    /**
     * Create custom uniform buffer
     * @param {string} id - Buffer ID
     * @param {TypedArray} data - Buffer data
     * @returns {GPUBuffer} Created buffer
     */
    createUniformBuffer(id, data) {
        const buffer = this.resourceManager.createUniformBuffer(id, data, this.device);
        this.uniformBuffers.set(id, buffer);
        return buffer;
    }

    /**
     * Get vertex buffer
     * @param {string} id - Buffer ID
     * @returns {GPUBuffer|null} Buffer or null
     */
    getVertexBuffer(id) {
        return this.vertexBuffers.get(id) || null;
    }

    /**
     * Get uniform buffer
     * @param {string} id - Buffer ID
     * @returns {GPUBuffer|null} Buffer or null
     */
    getUniformBuffer(id) {
        return this.uniformBuffers.get(id) || null;
    }

    /**
     * Get current index buffer and index count
     * @returns {Object} Object containing index buffer and index count
     */
    getIndexBufferInfo() {
        return {
            buffer: this.indexBuffer,
            count: this.indexCount,
        };
    }

    /**
     * Release all resources
     */
    destroy() {
        // Index buffer is directly managed, needs to be destroyed
        if (this.indexBuffer) {
            this.indexBuffer.destroy();
            this.indexBuffer = null;
        }

        // Other buffers are managed by ResourceManager, clear references
        this.vertexBuffers.clear();
        this.uniformBuffers.clear();
        this.indexCount = 0;
    }
}
