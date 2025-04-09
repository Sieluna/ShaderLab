import styles from './notebook-viewer.module.css';
import { createCellElement } from './notebook-cells.js';

/**
 * Create notebook viewer
 * @param {HTMLElement} container - Container element
 * @param {Object} options - Viewer options
 * @returns {Object} Notebook viewer API
 */
export function createNotebookViewer(container, options = {}) {
    // Internal state
    let notebook = null;
    let renderers = new Map();

    // Merge options
    const viewerOptions = {
        renderMath: true,
        codeSyntaxHighlight: true,
        autoRunShaders: true,
        ...options,
    };

    /**
     * Clear container
     * @private
     */
    const clearContainer = () => {
        while (container.firstChild) {
            container.removeChild(container.firstChild);
        }
    };

    /**
     * Show error message
     * @param {string} message - Error message
     * @private
     */
    const showError = (message) => {
        clearContainer();

        const errorElement = document.createElement('div');
        errorElement.className = styles.notebookError;
        errorElement.textContent = message;

        container.appendChild(errorElement);
    };

    /**
     * Render notebook content
     * @private
     */
    const renderNotebook = async () => {
        // Clear container
        clearContainer();

        // Create notebook content container
        const contentContainer = document.createElement('div');
        contentContainer.className = styles.notebookContent;
        container.appendChild(contentContainer);

        // Parse content
        const content =
            typeof notebook.content === 'string' ? JSON.parse(notebook.content) : notebook.content;

        if (!content || !content.cells || !Array.isArray(content.cells)) {
            showError('Invalid notebook content format');
            return;
        }

        // Render cells
        for (const cell of content.cells) {
            const cellElement = createCellElement(cell, notebook, renderers, viewerOptions);
            if (cellElement) {
                contentContainer.appendChild(cellElement);
            }
        }
    };

    // Initialize container
    const init = () => {
        clearContainer();

        // Set container style
        container.classList.add(styles.notebookViewer);

        // Create loading indicator
        const loadingIndicator = document.createElement('div');
        loadingIndicator.className = styles.notebookLoading;
        loadingIndicator.textContent = 'Loading...';
        container.appendChild(loadingIndicator);
    };

    // Initialize on creation
    init();

    // Return public API
    return {
        /**
         * Load notebook data
         * @param {Object|string} notebookData - Notebook data object or JSON string
         * @returns {Promise<boolean>} Whether loading succeeded
         */
        loadNotebook: async (notebookData) => {
            try {
                // Parse notebook data
                if (typeof notebookData === 'string') {
                    try {
                        notebookData = JSON.parse(notebookData);
                    } catch (e) {
                        console.error('Notebook JSON parsing failed:', e);
                        showError('Invalid notebook format');
                        return false;
                    }
                }

                notebook = notebookData;

                // Render notebook
                await renderNotebook();
                return true;
            } catch (error) {
                console.error('Failed to load notebook:', error);
                showError(`Failed to load notebook: ${error.message}`);
                return false;
            }
        },

        /**
         * Destroy viewer and clean up resources
         */
        destroy: () => {
            // Destroy all renderers
            for (const [, renderer] of renderers) {
                renderer.destroy();
            }
            renderers.clear();

            // Clear container
            clearContainer();

            // Remove class name
            container.classList.remove(styles.notebookViewer);
        },
    };
}
