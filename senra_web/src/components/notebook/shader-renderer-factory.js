import { ShaderRenderer } from './shader-renderer.js';
import { PerformanceMonitor } from './performance-monitor.js';
import { GeometryManager } from './geometry-manager.js';
import { TextureManager } from './texture-manager.js';
import { RenderPipelineManager } from './render-pipeline-manager.js';
import { BindGroupManager } from './bind-group-manager.js';
import { RenderPassExecutor } from './render-pass-executor.js';
import { UniformManager } from './uniform-manager.js';

export class ShaderRendererFactory {
    /**
     * Create a standard configured ShaderRenderer instance
     * @param {HTMLElement} container - DOM container element for the renderer
     * @param {GPUDevice} device - WebGPU device instance
     * @param {Object} config - Rendering configuration
     * @param {ResourceManager} resourceManager - Resource manager instance
     * @returns {ShaderRenderer} Created ShaderRenderer instance
     */
    static createRenderer(container, device, config, resourceManager) {
        // Create shader renderer
        const renderer = new ShaderRenderer(container, device, config, resourceManager);

        // Initialize renderer
        renderer.initialize();

        // Return initialized renderer
        return renderer;
    }

    /**
     * Create a ShaderRenderer instance with custom components
     * @param {HTMLElement} container - DOM container element for the renderer
     * @param {GPUDevice} device - WebGPU device instance
     * @param {Object} config - Rendering configuration
     * @param {ResourceManager} resourceManager - Resource manager instance
     * @param {Object} customComponents - Custom component configuration
     * @returns {ShaderRenderer} Created ShaderRenderer instance
     */
    static createCustomRenderer(container, device, config, resourceManager, customComponents = {}) {
        // Create UniformManager (if not provided)
        const uniformManager =
            customComponents.uniformManager || new UniformManager(device, resourceManager);

        // Create GeometryManager (if not provided)
        const geometryManager =
            customComponents.geometryManager || new GeometryManager(device, resourceManager);

        // Create TextureManager (if not provided)
        const textureManager = customComponents.textureManager || new TextureManager(device);

        // Create RenderPipelineManager (if not provided)
        const pipelineManager =
            customComponents.pipelineManager || new RenderPipelineManager(device);

        // Create BindGroupManager (if not provided)
        const bindGroupManager =
            customComponents.bindGroupManager ||
            new BindGroupManager(device, resourceManager, uniformManager);

        // Create RenderPassExecutor (if not provided)
        const passExecutor = customComponents.passExecutor || new RenderPassExecutor(device);

        // Create PerformanceMonitor (if not provided)
        const performanceMonitor =
            customComponents.performanceMonitor || new PerformanceMonitor(config.performance);

        // Create custom instance
        const renderer = new ShaderRenderer(container, device, config, resourceManager);

        // Replace default components
        renderer.uniformManager = uniformManager;
        renderer.geometryManager = geometryManager;
        renderer.textureManager = textureManager;
        renderer.pipelineManager = pipelineManager;
        renderer.bindGroupManager = bindGroupManager;
        renderer.passExecutor = passExecutor;

        // Store performance monitor
        renderer.performanceMonitor = performanceMonitor;

        // Initialize renderer
        renderer.initialize();

        // Return initialized renderer
        return renderer;
    }

    /**
     * Extract components from renderer (for reusing existing components)
     * @param {ShaderRenderer} renderer - Renderer instance
     * @returns {Object} Component set
     */
    static extractComponents(renderer) {
        return {
            uniformManager: renderer.uniformManager,
            geometryManager: renderer.geometryManager,
            textureManager: renderer.textureManager,
            pipelineManager: renderer.pipelineManager,
            bindGroupManager: renderer.bindGroupManager,
            passExecutor: renderer.passExecutor,
            performanceMonitor: renderer.performanceMonitor,
        };
    }
}
