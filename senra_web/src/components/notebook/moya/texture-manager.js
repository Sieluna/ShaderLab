export class TextureManager {
    /**
     * Create a texture manager
     * @param {GPUDevice} device - WebGPU device
     */
    constructor(device) {
        this.device = device;
        this.passTextures = new Map();
        this.depthTexture = null;
    }

    /**
     * Create textures for render pass
     * @param {Object} passConfig - Pass configuration
     * @param {number} canvasWidth - Canvas width
     * @param {number} canvasHeight - Canvas height
     * @returns {Map<string, Object>} Pass texture mapping
     */
    createPassTextures(passConfig, canvasWidth, canvasHeight) {
        if (!passConfig.output_textures || passConfig.output_textures.length === 0) {
            return this.passTextures;
        }

        for (const textureConfig of passConfig.output_textures) {
            const width = Math.max(1, Math.floor(canvasWidth * (textureConfig.width_scale || 1.0)));
            const height = Math.max(
                1,
                Math.floor(canvasHeight * (textureConfig.height_scale || 1.0)),
            );
            const format = textureConfig.format || 'rgba8unorm';

            // Try to find existing texture
            const textureKey = `${passConfig.id}_${textureConfig.id}`;
            const existingTexture = this.passTextures.get(textureKey);

            // Skip if texture exists and size matches
            if (
                existingTexture &&
                existingTexture.width === width &&
                existingTexture.height === height
            ) {
                continue;
            }

            // If exists but size different, destroy old texture
            if (existingTexture) {
                existingTexture.texture.destroy();
            }

            // Create new texture
            const texture = this.device.createTexture({
                label: `Pass Texture ${textureKey}`,
                size: [width, height, 1],
                format: format,
                usage:
                    GPUTextureUsage.TEXTURE_BINDING |
                    GPUTextureUsage.RENDER_ATTACHMENT |
                    GPUTextureUsage.COPY_SRC,
            });

            // Create sampler
            const sampler = this.device.createSampler({
                magFilter: textureConfig.sampler_config?.mag_filter || 'linear',
                minFilter: textureConfig.sampler_config?.min_filter || 'linear',
                mipmapFilter: 'linear',
                addressModeU: textureConfig.sampler_config?.address_mode_u || 'clamp-to-edge',
                addressModeV: textureConfig.sampler_config?.address_mode_v || 'clamp-to-edge',
                addressModeW: 'clamp-to-edge',
            });

            // Store texture information
            this.passTextures.set(textureKey, {
                texture,
                sampler,
                width,
                height,
                format,
                config: textureConfig,
            });
        }

        return this.passTextures;
    }

    /**
     * Create textures for all render passes
     * @param {Array<Object>} passConfigs - Array of pass configurations
     * @param {number} canvasWidth - Canvas width
     * @param {number} canvasHeight - Canvas height
     */
    createAllPassTextures(passConfigs, canvasWidth, canvasHeight) {
        for (const passConfig of passConfigs) {
            // Only create textures for intermediate or postprocess passes
            if (passConfig.pass_type === 'intermediate' || passConfig.pass_type === 'postprocess') {
                this.createPassTextures(passConfig, canvasWidth, canvasHeight);
            }
        }
    }

    /**
     * Ensure depth texture exists and has correct size
     * @param {number} width - Width
     * @param {number} height - Height
     * @returns {GPUTexture} Depth texture
     */
    ensureDepthTexture(width, height) {
        if (
            !this.depthTexture ||
            this.depthTexture.width !== width ||
            this.depthTexture.height !== height
        ) {
            // Destroy old texture (if exists)
            if (this.depthTexture) {
                this.depthTexture.destroy();
            }

            // Create new depth texture
            this.depthTexture = this.device.createTexture({
                label: 'Depth Texture',
                size: [width, height],
                format: 'depth24plus',
                usage: GPUTextureUsage.RENDER_ATTACHMENT,
            });
        }

        return this.depthTexture;
    }

    /**
     * Resize textures
     * @param {number} width - New width
     * @param {number} height - New height
     */
    resizeTextures(width, height) {
        // Iterate through all textures and resize
        for (const [textureKey, textureInfo] of this.passTextures.entries()) {
            // Parse pass ID and texture ID
            const [passId, textureId] = textureKey.split('_');

            // Calculate new size based on configuration
            const widthScale = textureInfo.config.width_scale || 1.0;
            const heightScale = textureInfo.config.height_scale || 1.0;

            const newWidth = Math.max(1, Math.floor(width * widthScale));
            const newHeight = Math.max(1, Math.floor(height * heightScale));

            // Skip if size unchanged
            if (textureInfo.width === newWidth && textureInfo.height === newHeight) {
                continue;
            }

            // Destroy old texture
            textureInfo.texture.destroy();

            // Create new texture
            const newTexture = this.device.createTexture({
                label: `Pass Texture ${textureKey}`,
                size: [newWidth, newHeight, 1],
                format: textureInfo.format,
                usage:
                    GPUTextureUsage.TEXTURE_BINDING |
                    GPUTextureUsage.RENDER_ATTACHMENT |
                    GPUTextureUsage.COPY_SRC,
            });

            // Update texture information
            textureInfo.texture = newTexture;
            textureInfo.width = newWidth;
            textureInfo.height = newHeight;
        }

        // Update depth texture
        this.ensureDepthTexture(width, height);
    }

    /**
     * Get texture for specified pass and ID
     * @param {string} passId - Pass ID
     * @param {string} textureId - Texture ID
     * @returns {Object|null} Texture information or null
     */
    getPassTexture(passId, textureId) {
        return this.passTextures.get(`${passId}_${textureId}`) || null;
    }

    /**
     * Get depth texture
     * @returns {GPUTexture|null} Depth texture or null
     */
    getDepthTexture() {
        return this.depthTexture;
    }

    /**
     * Check if specified pass texture exists
     * @param {string} passId - Pass ID
     * @param {string} textureId - Texture ID
     * @returns {boolean} Whether it exists
     */
    hasPassTexture(passId, textureId) {
        return this.passTextures.has(`${passId}_${textureId}`);
    }

    /**
     * Get mapping of all pass textures
     * @returns {Map<string, Object>} Pass texture mapping
     */
    getAllPassTextures() {
        return this.passTextures;
    }

    /**
     * Release all resources
     */
    destroy() {
        // Release all pass textures
        for (const [, textureInfo] of this.passTextures.entries()) {
            textureInfo.texture.destroy();
        }
        this.passTextures.clear();

        // Release depth texture
        if (this.depthTexture) {
            this.depthTexture.destroy();
            this.depthTexture = null;
        }
    }
}
