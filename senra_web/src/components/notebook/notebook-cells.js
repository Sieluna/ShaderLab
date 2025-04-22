import styles from './notebook-viewer.module.css';
import { marked } from 'marked';
import { createNotebookRenderer } from './notebook-renderer.js';
import { createRendererControls } from './notebook-controls.js';
import { createShaderTabs } from './code/index.js';

/** @typedef {import('./index.js').NotebookCell} NotebookCell */
/** @typedef {import('./index.js').Notebook} Notebook */
/** @typedef {import('./index.js').ViewerOptions} ViewerOptions */

/**
 * @typedef {Object} CellInfo
 * @property {number} id - Cell ID
 * @property {Object} config - Cell configuration
 * @property {HTMLElement} container - Cell container element
 */

/**
 * @typedef {Map<string, Object>} RenderersMap
 */

/**
 * Render Markdown cell
 * @param {HTMLElement} container - Container element
 * @param {NotebookCell} cell - Cell data
 */
export function renderMarkdownCell(container, cell) {
    if (!cell.content) {
        container.textContent = 'Empty Markdown cell';
        return;
    }

    // Render Markdown using marked library
    container.innerHTML = marked(cell.content);
}

/**
 * Render code cell
 * @param {HTMLElement} container - Container element
 * @param {NotebookCell} cell - Cell data
 * @param {Notebook} notebook - Notebook data
 * @param {ViewerOptions} options - Viewer options
 */
export function renderCodeCell(container, cell, notebook, options = {}) {
    if (!cell.content) {
        container.textContent = 'Empty code cell';
        return;
    }

    // Check if content has shader_ids
    if (
        typeof cell.content === 'object' &&
        cell.content.shader_ids &&
        Array.isArray(cell.content.shader_ids)
    ) {
        // Shader code editing
        const shaderIds = cell.content.shader_ids;

        if (shaderIds.length === 0) {
            container.textContent = 'No shaders associated with this cell';
            return;
        }

        // Find the referenced shaders
        const shaders = [];
        if (notebook && notebook.shaders && Array.isArray(notebook.shaders)) {
            shaderIds.forEach((id) => {
                const shader = notebook.shaders.find((s) => s.id === id);
                if (shader) {
                    shaders.push(shader);
                }
            });
        }

        if (shaders.length === 0) {
            container.textContent = 'Referenced shaders not found in the notebook';
            return;
        }

        // Create shader tabs with editors
        createShaderTabs(container, shaders, notebook, {
            readOnly: options.readOnlyEditors || false,
            onChange: (data) => {
                console.log('Shader content changed:', data);
                // Here you could implement saving the changes back to the notebook
            },
        });
    } else {
        // Regular code display (non-shader)
        const pre = document.createElement('pre');
        const code = document.createElement('code');

        // Convert object content to string (if needed)
        const codeContent =
            typeof cell.content === 'object' ? JSON.stringify(cell.content, null, 2) : cell.content;

        code.textContent = codeContent;
        pre.appendChild(code);
        container.appendChild(pre);
    }
}

/**
 * Render shader cell
 * @param {HTMLElement} container - Container element
 * @param {NotebookCell} cell - Cell data
 * @param {Notebook} notebook - Notebook data
 * @param {RenderersMap} renderers - Renderers map
 * @param {ViewerOptions} options - Viewer options
 */
export function renderShaderCell(container, cell, notebook, renderers, options = {}) {
    if (!cell.content) {
        container.textContent = 'Empty render cell';
        return;
    }

    // Parse content
    const renderConfig = typeof cell.content === 'string' ? JSON.parse(cell.content) : cell.content;

    // Create render area outer container (for centering)
    const renderWrapper = document.createElement('div');
    renderWrapper.className = styles.renderWrapper;
    container.appendChild(renderWrapper);

    // Create render container
    const renderContainer = document.createElement('div');
    renderContainer.className = styles.renderContainer;
    renderContainer.style.width = `${renderConfig.width || 400}px`;
    renderContainer.style.height = `${renderConfig.height || 300}px`;
    renderWrapper.appendChild(renderContainer);

    // Generate unique ID
    const rendererId = `renderer-${cell.id || Math.random().toString(36).substring(2, 9)}`;
    renderContainer.id = rendererId;

    // Create controls container
    const controlsContainer = document.createElement('div');
    controlsContainer.className = styles.renderControlsContainer;
    renderWrapper.appendChild(controlsContainer);

    // Auto-run shader or display start button
    if (options.autoRunShaders) {
        // Delay rendering to ensure DOM is ready
        setTimeout(() => {
            try {
                const renderer = createNotebookRenderer(rendererId, notebook, {
                    powerPreference: renderConfig.performance?.hardware_acceleration
                        ? 'high-performance'
                        : 'low-power',
                });

                if (renderer) {
                    // Store renderer reference
                    renderers.set(rendererId, renderer);

                    // Create a proxy object to extend renderer API to handle updateUniform calls
                    const rendererProxy = {
                        ...renderer,
                        // Add an adapter method to map updateUniform calls to appropriate notebook-renderer API
                        updateUniform: (name, value) => {
                            try {
                                // Simply use the renderer's updateUniform method
                                if (renderer && typeof renderer.updateUniform === 'function') {
                                    renderer.updateUniform(name, value);
                                } else {
                                    // Fallback to update method if updateUniform is not available
                                    renderer.update({
                                        uniform: { name, value },
                                    });
                                }
                            } catch (error) {
                                console.error(`Failed to update Uniform "${name}":`, error);
                            }
                        },
                    };

                    // Create renderer control panel using extended proxy object
                    createRendererControls(renderWrapper, renderConfig, rendererId, rendererProxy);
                }
            } catch (error) {
                console.error('Failed to create renderer:', error);
                renderContainer.innerHTML = `<div class="${styles.errorMessage}">Renderer initialization failed: ${error.message}</div>`;
            }
        }, 100);
    } else {
        // Display start button
        const startButton = document.createElement('button');
        startButton.className = styles.renderStartButton;
        startButton.textContent = 'Start Renderer';
        startButton.addEventListener('click', () => {
            startButton.remove();

            try {
                const renderer = createNotebookRenderer(rendererId, notebook, {
                    powerPreference: renderConfig.performance?.hardware_acceleration
                        ? 'high-performance'
                        : 'low-power',
                });

                if (renderer) {
                    // Store renderer reference
                    renderers.set(rendererId, renderer);

                    // Create a proxy object to extend renderer API to handle updateUniform calls
                    const rendererProxy = {
                        ...renderer,
                        // Add an adapter method to map updateUniform calls to appropriate notebook-renderer API
                        updateUniform: (name, value) => {
                            try {
                                // Simply use the renderer's updateUniform method
                                if (renderer && typeof renderer.updateUniform === 'function') {
                                    renderer.updateUniform(name, value);
                                } else {
                                    // Fallback to update method if updateUniform is not available
                                    renderer.update({
                                        uniform: { name, value },
                                    });
                                }
                            } catch (error) {
                                console.error(`Failed to update Uniform "${name}":`, error);
                            }
                        },
                    };

                    // Create renderer control panel using extended proxy object
                    createRendererControls(renderWrapper, renderConfig, rendererId, rendererProxy);
                }
            } catch (error) {
                console.error('Failed to create renderer:', error);
                renderContainer.innerHTML = `<div class="${styles.errorMessage}">Renderer initialization failed: ${error.message}</div>`;
            }
        });

        renderContainer.appendChild(startButton);
    }
}

/**
 * Create cell element
 * @param {NotebookCell} cell - Cell data
 * @param {Notebook} notebook - Notebook data
 * @param {RenderersMap} renderers - Renderers map
 * @param {ViewerOptions} options - Viewer options
 * @returns {HTMLElement|null} Cell element
 */
export function createCellElement(cell, notebook, renderers, options = {}) {
    if (!cell || !cell.cell_type) {
        return null;
    }

    const cellElement = document.createElement('div');
    cellElement.className = `${styles.notebookCell} ${styles[`cell${cell.cell_type.charAt(0).toUpperCase() + cell.cell_type.slice(1)}`]}`;
    if (cell.id) {
        cellElement.dataset.cellId = cell.id;
    }

    // If cell is collapsed, add collapsed style
    if (cell.metadata && cell.metadata.collapsed) {
        cellElement.classList.add(styles.collapsed);
    }

    // Create cell toolbar
    const toolbar = document.createElement('div');
    toolbar.className = styles.cellToolbar;

    // Add cell type indicator
    const typeIndicator = document.createElement('span');
    typeIndicator.className = styles.cellTypeIndicator;
    typeIndicator.textContent = getCellTypeLabel(cell.cell_type);
    toolbar.appendChild(typeIndicator);

    // Add collapse button
    const collapseButton = document.createElement('button');
    collapseButton.className = styles.cellCollapseButton;
    collapseButton.innerHTML = cell.metadata?.collapsed ? 'Expand' : 'Collapse';
    collapseButton.addEventListener('click', () => {
        cellElement.classList.toggle(styles.collapsed);
        collapseButton.innerHTML = cellElement.classList.contains(styles.collapsed)
            ? 'Expand'
            : 'Collapse';
    });
    toolbar.appendChild(collapseButton);

    cellElement.appendChild(toolbar);

    // Create cell content
    const content = document.createElement('div');
    content.className = styles.cellContent;

    // Render content based on cell type
    switch (cell.cell_type) {
        case 'markdown':
            renderMarkdownCell(content, cell);
            break;
        case 'code':
            renderCodeCell(content, cell, notebook, options);
            break;
        case 'render':
            renderShaderCell(content, cell, notebook, renderers, options);
            break;
        default:
            content.textContent = `Unsupported cell type: ${cell.cell_type}`;
    }

    cellElement.appendChild(content);
    return cellElement;
}

/**
 * Get cell type label
 * @param {string} cellType - Cell type
 * @returns {string} Cell type label
 * @private
 */
function getCellTypeLabel(cellType) {
    switch (cellType) {
        case 'markdown':
            return 'Markdown';
        case 'code':
            return 'Code';
        case 'render':
            return 'Renderer';
        default:
            return cellType;
    }
}
