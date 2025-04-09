function parseShaderUniforms(shaderCode) {
    // More precise regex that can match various possible uniform declaration formats
    const uniformPattern =
        /@group\(\s*(\d+)\s*\)\s*@binding\(\s*(\d+)\s*\)\s*var\s*<\s*uniform\s*>\s*(\w+)\s*:\s*(\w+)/g;
    const uniforms = [];

    // Capture all matches
    let match;
    while ((match = uniformPattern.exec(shaderCode)) !== null) {
        const group = parseInt(match[1]);
        const binding = parseInt(match[2]);
        const name = match[3];
        const type = match[4];

        console.log(
            `Found uniform in shader: group=${group}, binding=${binding}, name=${name}, type=${type}`,
        );

        uniforms.push({
            group,
            binding,
            name,
            type,
        });
    }

    return uniforms;
}

export class UniformManager {
    #parsedShaders = new Map(); // Store parsed shaders
    #uniformBuffers = new Map(); // Store all uniform buffers
    #bindGroupLayouts = new Map(); // Store binding group layouts
    #bindGroups = new Map(); // Store binding groups
    #defaultValues = new Map(); // Store default values

    constructor(device, resourceManager = null) {
        this.device = device;
        this.resourceManager = resourceManager; // Optional resource manager
    }

    // Analyze shader and record its uniform requirements
    analyzeShader(shaderId, shaderCode) {
        const uniforms = parseShaderUniforms(shaderCode);
        this.#parsedShaders.set(shaderId, uniforms);
        return uniforms;
    }

    // Set default value for a uniform
    setDefaultValue(shaderId, uniformName, defaultValue) {
        const key = `${shaderId}_${uniformName}`;
        this.#defaultValues.set(key, defaultValue);
    }

    // Create binding group layout for shader
    createBindGroupLayoutForShader(shaderId) {
        const uniforms = this.#parsedShaders.get(shaderId);
        if (!uniforms || uniforms.length === 0) return null;

        // Group by group
        const groupMap = new Map();
        for (const uniform of uniforms) {
            if (!groupMap.has(uniform.group)) {
                groupMap.set(uniform.group, []);
            }
            groupMap.get(uniform.group).push(uniform);
        }

        // Create layout for each group
        for (const [groupId, groupUniforms] of groupMap.entries()) {
            // Sort by binding
            groupUniforms.sort((a, b) => a.binding - b.binding);

            const entries = groupUniforms.map((u) => ({
                binding: u.binding,
                visibility: GPUShaderStage.VERTEX | GPUShaderStage.FRAGMENT,
                buffer: { type: 'uniform' },
            }));

            const layout = this.device.createBindGroupLayout({
                label: `UniformLayout_${shaderId}_Group${groupId}`,
                entries,
            });

            this.#bindGroupLayouts.set(`${shaderId}_group${groupId}`, layout);
        }

        return Array.from(groupMap.keys());
    }

    // Ensure all uniform buffers exist
    ensureUniformBuffers(shaderId) {
        const uniforms = this.#parsedShaders.get(shaderId);
        if (!uniforms || uniforms.length === 0) return;

        for (const uniform of uniforms) {
            const bufferKey = `${shaderId}_${uniform.name}`;

            if (!this.#uniformBuffers.has(bufferKey)) {
                // Check if there's a specified default value
                const defaultValue =
                    this.#defaultValues.get(bufferKey) ||
                    this.#createDefaultValueForType(uniform.type);

                // Create typed array
                const typedArray = this.#createTypedArrayForValue(defaultValue, uniform.type);

                // Create buffer
                let buffer;
                if (this.resourceManager) {
                    // Use resource manager to create
                    buffer = this.resourceManager.createUniformBuffer(
                        `uniform_${bufferKey}`,
                        typedArray,
                        this.device,
                    );
                } else {
                    // Create directly
                    buffer = this.device.createBuffer({
                        label: `UniformBuffer_${bufferKey}`,
                        size: typedArray.byteLength,
                        usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
                    });
                    this.device.queue.writeBuffer(buffer, 0, typedArray);
                }

                this.#uniformBuffers.set(bufferKey, {
                    buffer,
                    type: uniform.type,
                    value: typedArray,
                });
            }
        }
    }

    // Create all needed binding groups
    createBindGroups(shaderId) {
        const uniforms = this.#parsedShaders.get(shaderId);
        if (!uniforms || uniforms.length === 0) return;

        // Group by group
        const groupMap = new Map();
        for (const uniform of uniforms) {
            if (!groupMap.has(uniform.group)) {
                groupMap.set(uniform.group, []);
            }
            groupMap.get(uniform.group).push(uniform);
        }

        // Create binding group for each group
        for (const [groupId, groupUniforms] of groupMap.entries()) {
            // Sort by binding
            groupUniforms.sort((a, b) => a.binding - b.binding);

            const entries = [];
            let hasAllBuffers = true;

            for (const u of groupUniforms) {
                const bufferInfo = this.#uniformBuffers.get(`${shaderId}_${u.name}`);
                if (!bufferInfo) {
                    hasAllBuffers = false;
                    console.warn(`Missing buffer for uniform ${u.name} in shader ${shaderId}`);
                    break;
                }

                entries.push({
                    binding: u.binding,
                    resource: { buffer: bufferInfo.buffer },
                });
            }

            if (!hasAllBuffers) continue;

            try {
                const layout = this.#bindGroupLayouts.get(`${shaderId}_group${groupId}`);
                if (!layout) {
                    console.warn(`Missing layout for group ${groupId} in shader ${shaderId}`);
                    continue;
                }

                const bindGroup = this.device.createBindGroup({
                    label: `UniformBindGroup_${shaderId}_Group${groupId}`,
                    layout: layout,
                    entries,
                });

                this.#bindGroups.set(`${shaderId}_group${groupId}`, bindGroup);
            } catch (error) {
                console.error(`Error creating bind group for ${shaderId}_group${groupId}:`, error);
            }
        }
    }

    // Get specific binding group
    getBindGroup(shaderId, groupId) {
        return this.#bindGroups.get(`${shaderId}_group${groupId}`);
    }

    // Get binding group layout
    getBindGroupLayout(shaderId, groupId) {
        return this.#bindGroupLayouts.get(`${shaderId}_group${groupId}`);
    }

    // Update uniform value
    updateUniform(shaderId, uniformName, value) {
        const bufferKey = `${shaderId}_${uniformName}`;
        const uniformInfo = this.#uniformBuffers.get(bufferKey);

        if (!uniformInfo) {
            console.warn(`Uniform ${uniformName} not found for shader ${shaderId}`);
            return false;
        }

        try {
            // Create typed array for the given value
            const typedArray = this.#createTypedArrayForValue(value, uniformInfo.type);

            // Update buffer
            if (this.resourceManager) {
                this.resourceManager.updateBuffer(`uniform_${bufferKey}`, typedArray, this.device);
            } else {
                this.device.queue.writeBuffer(uniformInfo.buffer, 0, typedArray);
            }

            // Update stored value
            uniformInfo.value = typedArray;
            return true;
        } catch (error) {
            console.error(`Error updating uniform ${uniformName}:`, error);
            return false;
        }
    }

    // Create default value based on type
    #createDefaultValueForType(type) {
        switch (type) {
            case 'f32':
            case 'float':
                return 0.0;
            case 'i32':
            case 'int':
                return 0;
            case 'u32':
            case 'uint':
                return 0;
            case 'bool':
                return false;
            case 'vec2f':
                return [0, 0];
            case 'vec3f':
                return [0, 0, 0];
            case 'vec4f':
                return [0, 0, 0, 1];
            case 'mat2x2f':
                return [1, 0, 0, 1]; // Identity matrix
            case 'mat3x3f':
                return [1, 0, 0, 0, 1, 0, 0, 0, 1]; // Identity matrix
            case 'mat4x4f':
                return [
                    1, 0, 0, 0,
                    0, 1, 0, 0,
                    0, 0, 1, 0,
                    0, 0, 0, 1
                ]; // Identity matrix
            default:
                console.warn(`Unknown uniform type: ${type}, using float array`);
                return new Float32Array(4);
        }
    }

    // Create typed array for given value
    #createTypedArrayForValue(value, type) {
        // If already a TypedArray, return it directly
        if (ArrayBuffer.isView(value)) {
            return value;
        }

        switch (type) {
            case 'f32':
            case 'float':
                return new Float32Array([value]);
            case 'i32':
            case 'int':
                return new Int32Array([value]);
            case 'u32':
            case 'uint':
                return new Uint32Array([value]);
            case 'bool':
                return new Uint32Array([value ? 1 : 0]);
            case 'vec2f':
                return new Float32Array(Array.isArray(value) ? value : [value, 0]);
            case 'vec3f':
                return new Float32Array(Array.isArray(value) ? value : [value, 0, 0]);
            case 'vec4f':
                return new Float32Array(Array.isArray(value) ? value : [value, 0, 0, 0]);
            case 'mat2x2f':
            case 'mat3x3f':
            case 'mat4x4f':
                if (Array.isArray(value)) {
                    return new Float32Array(value);
                } else {
                    return new Float32Array(this.#createDefaultValueForType(type));
                }
            default:
                console.warn(`Unknown type: ${type}, using float array`);
                if (Array.isArray(value)) {
                    // Ensure array length is a multiple of 4 (WebGPU alignment requirement)
                    const paddedLength = Math.ceil(value.length / 4) * 4;
                    const paddedArray = new Float32Array(paddedLength);
                    paddedArray.set(value);
                    return paddedArray;
                } else {
                    return new Float32Array([value, 0, 0, 0]);
                }
        }
    }

    // Remove all resources related to shader
    removeShader(shaderId) {
        // Remove parsed records
        this.#parsedShaders.delete(shaderId);

        // Delete uniform buffers
        for (const [key, info] of this.#uniformBuffers.entries()) {
            if (key.startsWith(`${shaderId}_`)) {
                this.#uniformBuffers.delete(key);
                if (!this.resourceManager) {
                    info.buffer.destroy();
                }
            }
        }

        // Delete default values
        for (const key of this.#defaultValues.keys()) {
            if (key.startsWith(`${shaderId}_`)) {
                this.#defaultValues.delete(key);
            }
        }

        // Delete binding groups and layouts
        for (const key of this.#bindGroups.keys()) {
            if (key.startsWith(`${shaderId}_`)) {
                this.#bindGroups.delete(key);
            }
        }

        for (const key of this.#bindGroupLayouts.keys()) {
            if (key.startsWith(`${shaderId}_`)) {
                this.#bindGroupLayouts.delete(key);
            }
        }
    }

    // Destructor - clean up all resources
    destroy() {
        // Only need to release directly created buffers (not managed by resource manager)
        if (!this.resourceManager) {
            for (const info of this.#uniformBuffers.values()) {
                info.buffer.destroy();
            }
        }

        this.#uniformBuffers.clear();
        this.#bindGroups.clear();
        this.#bindGroupLayouts.clear();
        this.#parsedShaders.clear();
        this.#defaultValues.clear();
    }

    // Getter for parsedShaders - used in other components
    get parsedShaders() {
        return this.#parsedShaders;
    }

    // Get all shader IDs
    getShaderIds() {
        return Array.from(this.#parsedShaders.keys());
    }
}
