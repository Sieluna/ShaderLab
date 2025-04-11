import { MoyaEngine } from './moya/index.js';

/** @typedef {import('./index.js').Notebook} Notebook */

/**
 * @typedef {Object} RendererOptions
 * @property {string} [powerPreference] - Power preference for WebGPU
 */

/**
 * @typedef {Object} RendererStatus
 * @property {string} status - Renderer status
 * @property {number} rendererCount - Number of active renderers
 * @property {Object|null} engineStatus - Engine status
 */

/**
 * @typedef {Object} RendererAPI
 * @property {function(Object): void} update - Update renderer
 * @property {function(): void} reset - Reset renderer
 * @property {function(): void} destroy - Destroy renderer
 * @property {function(number, number): void} resize - Resize renderer
 * @property {function(): void} pause - Pause renderer
 * @property {function(): void} resume - Resume renderer
 * @property {function(): RendererStatus} getStatus - Get renderer status
 * @property {function(string, any): void} updateUniform - Update uniform value
 */

// Store active renderers
const activeRenderers = new Map();

/**
 * Create a notebook renderer
 * @param {string} containerId - Container ID
 * @param {Notebook} notebook - Notebook data
 * @param {RendererOptions} options - Render options
 * @returns {RendererAPI|null} Renderer API
 */
export function createNotebookRenderer(containerId, notebook, options = {}) {
    // Get container element
    const container = document.getElementById(containerId);
    if (!container) {
        console.error(`Container with id '${containerId}' not found`);
        return null;
    }

    // Check WebGPU support
    if (!navigator.gpu) {
        console.error('WebGPU is not supported in this browser');
        container.innerHTML =
            '<div class="error-message">Your browser does not support WebGPU. Please use the latest version of Chrome or Edge.</div>';
        return null;
    }

    // Internal state
    let engine = null;
    let notebookData = notebook;
    let renderCells = new Map(); // Store render cell information
    let containersMap = new Map(); // Container ID to cell ID mapping
    let resizeObserver = null;
    let commandBuffer = []; // Command buffer
    let isRunning = false;
    let status = 'created';

    // API object with direct method implementations
    const api = {
        update: (data) => {
            // Update entire notebook
            if (data.notebook) {
                notebookData = data.notebook;
                commandBuffer.push({
                    type: 'update',
                    data: { notebook: notebookData },
                });

                // Parse notebook content
                const content =
                    typeof notebookData.content === 'string'
                        ? JSON.parse(notebookData.content)
                        : notebookData.content;

                if (!content?.cells) {
                    console.warn('Invalid notebook content format');
                    return;
                }

                // Extract render cells
                const cells = content.cells.filter((cell) => cell.cell_type === 'render');
                renderCells.clear();

                // Create render cells
                for (const cell of cells) {
                    const config =
                        typeof cell.content === 'string' ? JSON.parse(cell.content) : cell.content;

                    const renderContainer = document.createElement('div');
                    renderContainer.className = 'renderer-container';
                    renderContainer.id = `render-${cell.id}`;
                    renderContainer.dataset.cellId = cell.id;
                    renderContainer.style.width = `${config.width || 640}px`;
                    renderContainer.style.height = `${config.height || 480}px`;
                    container.appendChild(renderContainer);

                    renderCells.set(cell.id, {
                        id: cell.id,
                        config: config,
                        container: renderContainer,
                    });
                    containersMap.set(renderContainer.id, cell.id);

                    commandBuffer.push({
                        type: 'init',
                        data: {
                            containerId: renderContainer.id,
                            notebook: notebookData,
                            config: config,
                            options: options,
                        },
                    });
                }
            }

            // Update specific cell
            if (data.cellId && data.config) {
                const cellInfo = renderCells.get(data.cellId);
                if (cellInfo?.container) {
                    cellInfo.config = { ...cellInfo.config, ...data.config };
                    commandBuffer.push({
                        type: 'update',
                        data: {
                            containerId: cellInfo.container.id,
                            config: data.config,
                        },
                    });
                }
            }

            // Update uniform values
            if (data.uniform) {
                renderCells.forEach((cellInfo) => {
                    if (cellInfo.container) {
                        commandBuffer.push({
                            type: 'uniform',
                            data: {
                                containerId: cellInfo.container.id,
                                name: data.uniform.name,
                                value: data.uniform.value,
                            },
                        });
                    }
                });
            }

            // Flush commands to engine
            if (commandBuffer.length > 0 && engine) {
                engine.processCommands(commandBuffer);
                commandBuffer = [];
            }
        },

        reset: () => {
            // Pause rendering
            api.pause();

            // Generate reset commands
            renderCells.forEach((cellInfo) => {
                if (cellInfo.container) {
                    commandBuffer.push({
                        type: 'reset',
                        data: { containerId: cellInfo.container.id },
                    });
                }
            });

            // Flush commands
            if (engine && commandBuffer.length > 0) {
                engine.processCommands(commandBuffer);
                commandBuffer = [];
            }

            // Clear DOM container
            while (container.firstChild) {
                container.removeChild(container.firstChild);
            }

            // Clear local state
            renderCells.clear();
            containersMap.clear();
            status = 'reset';
        },

        destroy: () => {
            api.pause();

            if (resizeObserver) {
                resizeObserver.disconnect();
                resizeObserver = null;
            }

            renderCells.forEach((cellInfo) => {
                if (cellInfo.container) {
                    commandBuffer.push({
                        type: 'destroy',
                        data: { containerId: cellInfo.container.id },
                    });
                }
            });

            if (engine && commandBuffer.length > 0) {
                engine.processCommands(commandBuffer);
                commandBuffer = [];
            }

            if (engine) {
                engine.destroy();
                engine = null;
            }

            while (container.firstChild) {
                container.removeChild(container.firstChild);
            }

            renderCells.clear();
            containersMap.clear();
            activeRenderers.delete(containerId);
            status = 'destroyed';
        },

        resize: (width, height) => {
            renderCells.forEach((cellInfo) => {
                if (cellInfo.container) {
                    commandBuffer.push({
                        type: 'resize',
                        data: {
                            containerId: cellInfo.container.id,
                            width: width,
                            height: height,
                        },
                    });

                    cellInfo.container.style.width = `${width}px`;
                    cellInfo.container.style.height = `${height}px`;
                }
            });

            if (engine && commandBuffer.length > 0) {
                engine.processCommands(commandBuffer);
                commandBuffer = [];
            }
        },

        pause: () => {
            // Send pause commands to all cell renderers
            renderCells.forEach((cellInfo) => {
                if (cellInfo.container) {
                    commandBuffer.push({
                        type: 'pause',
                        data: { containerId: cellInfo.container.id },
                    });
                }
            });

            if (engine && commandBuffer.length > 0) {
                engine.processCommands(commandBuffer);
                commandBuffer = [];
            }

            // Also directly pause the engine
            if (engine && typeof engine.pause === 'function') {
                engine.pause();
            }

            isRunning = false;
            status = 'paused';
        },

        resume: () => {
            if (status !== 'destroyed') {
                // Send resume commands to all cell renderers
                renderCells.forEach((cellInfo) => {
                    if (cellInfo.container) {
                        commandBuffer.push({
                            type: 'resume',
                            data: { containerId: cellInfo.container.id },
                        });
                    }
                });

                if (engine && commandBuffer.length > 0) {
                    engine.processCommands(commandBuffer);
                    commandBuffer = [];
                }

                // Also directly resume the engine
                if (engine && typeof engine.resume === 'function') {
                    engine.resume();
                }

                isRunning = true;
                status = 'running';
            }
        },

        getStatus: () => {
            return {
                status: status,
                rendererCount: renderCells.size,
                engineStatus: engine ? engine.getStatus() : null,
            };
        },

        updateUniform: (name, value) => {
            if (!engine) return;

            // Create a simplified update command that directly targets the main uniform buffer
            // without trying to update every shader
            commandBuffer.push({
                type: 'uniform',
                data: {
                    // Use first available render cell container as target
                    containerId: Array.from(renderCells.values())[0]?.container?.id,
                    name: name,
                    value: value,
                },
            });

            if (commandBuffer.length > 0) {
                engine.processCommands(commandBuffer);
                commandBuffer = [];
            }
        },
    };

    // Initialize the renderer
    (async () => {
        try {
            status = 'initializing';
            engine = new MoyaEngine(options);
            await engine.initialize();

            // Parse initial notebook content
            const content =
                typeof notebookData.content === 'string'
                    ? JSON.parse(notebookData.content)
                    : notebookData.content;

            if (!content?.cells) {
                console.warn('Invalid notebook content format');
                return;
            }

            // Create initial render cells
            const cells = content.cells.filter((cell) => cell.cell_type === 'render');
            for (const cell of cells) {
                const config =
                    typeof cell.content === 'string' ? JSON.parse(cell.content) : cell.content;

                const renderContainer = document.createElement('div');
                renderContainer.className = 'renderer-container';
                renderContainer.id = `render-${cell.id}`;
                renderContainer.dataset.cellId = cell.id;
                renderContainer.style.width = `${config.width || 640}px`;
                renderContainer.style.height = `${config.height || 480}px`;
                container.appendChild(renderContainer);

                renderCells.set(cell.id, {
                    id: cell.id,
                    config: config,
                    container: renderContainer,
                });
                containersMap.set(renderContainer.id, cell.id);

                commandBuffer.push({
                    type: 'init',
                    data: {
                        containerId: renderContainer.id,
                        notebook: notebookData,
                        config: config,
                        options: options,
                    },
                });
            }

            // Setup resize observer
            resizeObserver = new ResizeObserver((entries) => {
                entries.forEach((entry) => {
                    const containerWidth = entry.contentRect.width;
                    const container = entry.target;

                    if (container.id && containersMap.has(container.id)) {
                        const cellId = containersMap.get(container.id);
                        const cellInfo = renderCells.get(cellId);

                        if (cellInfo?.config) {
                            const aspectRatio = cellInfo.config.width / cellInfo.config.height;
                            const height = Math.floor(containerWidth / aspectRatio);

                            commandBuffer.push({
                                type: 'resize',
                                data: {
                                    containerId: container.id,
                                    width: containerWidth,
                                    height: height,
                                },
                            });

                            if (engine) {
                                engine.processCommands(commandBuffer);
                                commandBuffer = [];
                            }
                        }
                    }
                });
            });
            resizeObserver.observe(container);

            // Process initial commands
            if (commandBuffer.length > 0) {
                engine.processCommands(commandBuffer);
                commandBuffer = [];
            }

            isRunning = true;
            status = 'running';
            activeRenderers.set(containerId, api);
        } catch (error) {
            status = 'error';
            console.error('Renderer initialization failed:', error);
            container.innerHTML = `<div class="error-message">Renderer initialization failed: ${error.message}</div>`;
            throw error;
        }
    })();

    return api;
}
