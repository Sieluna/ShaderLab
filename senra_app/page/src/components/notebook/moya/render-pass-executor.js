/**
 * Render Pass Executor
 * Responsible for configuring and executing individual render passes
 */
export class RenderPassExecutor {
    /**
     * Create a render pass executor
     * @param {GPUDevice} device - WebGPU device
     */
    constructor(device) {
        this.device = device;
    }

    /**
     * Execute a render pass
     * @param {GPUCommandEncoder} encoder - Command encoder
     * @param {string} passId - Pass ID
     * @param {Object} pipelineInfo - Pipeline information
     * @param {Object} resources - Render resources
     * @param {Map<string, Object>} passTextures - Pass texture mapping
     * @param {GPUTexture} depthTexture - Depth texture
     * @param {GPUCanvasContext} canvasContext - Canvas context
     * @param {Map<string, GPUBuffer>} vertexBuffers - Vertex buffers
     * @param {GPUBuffer} indexBuffer - Index buffer
     * @param {number} indexCount - Index count
     * @returns {boolean} Execution success status
     */
    executeRenderPass(
        encoder,
        passId,
        pipelineInfo,
        { passTextures, depthTexture, canvasContext, vertexBuffers, indexBuffer, indexCount },
    ) {
        const { config: passConfig } = pipelineInfo;

        // Create render pass descriptor
        const renderPassDescriptor = {
            colorAttachments: [],
        };

        // Configure color attachments
        if (passConfig.pass_type === 'main') {
            try {
                // Get current frame texture view
                const textureView = canvasContext.getCurrentTexture().createView();

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
                return false;
            }
        } else if (passConfig.output_textures?.length > 0) {
            // Other passes render to textures
            for (const outputTexture of passConfig.output_textures) {
                const textureKey = `${passId}_${outputTexture.id}`;
                const textureInfo = passTextures.get(textureKey);

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
            return false;
        }

        // Skip this pass if no valid color attachments
        if (renderPassDescriptor.colorAttachments.length === 0) {
            console.warn(`No valid color attachments for pass ${passId}`);
            return false;
        }

        // Add depth attachment (if needed)
        if (passConfig.depth_enabled && depthTexture) {
            renderPassDescriptor.depthStencilAttachment = {
                view: depthTexture.createView(),
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
            pass.setVertexBuffer(0, vertexBuffers.get('position'));

            // Set index buffer
            pass.setIndexBuffer(indexBuffer, 'uint16');

            // Set bind groups
            for (let i = 0; i < pipelineInfo.bindGroups.length; i++) {
                pass.setBindGroup(i, pipelineInfo.bindGroups[i]);
            }

            // Draw according to geometry configuration
            if (passConfig.geometry) {
                switch (passConfig.geometry.type) {
                    case 'indexed':
                        pass.drawIndexed(
                            passConfig.geometry.index_count ?? indexCount,
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
                        // Default draw entire quad
                        pass.drawIndexed(indexCount);
                        break;
                }
            } else {
                // Default draw entire quad
                pass.drawIndexed(indexCount);
            }

            // End render pass
            pass.end();
            return true;
        } catch (e) {
            console.error(`Error executing render pass ${passId}:`, e);
            return false;
        }
    }

    /**
     * Ensure depth texture exists and meets size requirements
     * @param {number} width - Texture width
     * @param {number} height - Texture height
     * @param {GPUTexture|null} currentDepthTexture - Current depth texture
     * @returns {GPUTexture} Depth texture
     */
    ensureDepthTexture(width, height, currentDepthTexture) {
        // If depth texture doesn't exist or size doesn't match, need to recreate
        if (
            !currentDepthTexture ||
            currentDepthTexture.width !== width ||
            currentDepthTexture.height !== height
        ) {
            // Destroy old texture (if exists)
            currentDepthTexture?.destroy();

            // Create new depth texture
            return this.device.createTexture({
                label: 'Depth Texture',
                size: [width, height],
                format: 'depth24plus',
                usage: GPUTextureUsage.RENDER_ATTACHMENT,
            });
        }

        return currentDepthTexture;
    }
}
