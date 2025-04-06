import styles from './notebook-viewer.module.css';
import { createCellElement } from './notebook-cell-renderer.js';

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
     * Create notebook header
     * @private
     */
    const createHeader = () => {
        const header = document.createElement('div');
        header.className = styles.notebookHeader;

        // Title
        const title = document.createElement('h1');
        title.className = styles.notebookTitle;
        title.textContent = notebook.title || 'Untitled Notebook';
        header.appendChild(title);

        // Description
        if (notebook.description) {
            const description = document.createElement('div');
            description.className = styles.notebookDescription;
            description.textContent = notebook.description;
            header.appendChild(description);
        }

        // Metadata
        const meta = document.createElement('div');
        meta.className = styles.notebookMeta;

        // Author
        if (notebook.author) {
            const author = document.createElement('span');
            author.className = styles.notebookAuthor;
            author.textContent = `Author: ${notebook.author.username || 'Unknown'}`;
            meta.appendChild(author);
        }

        // Update time
        if (notebook.updated_at) {
            const updated = document.createElement('span');
            updated.className = styles.notebookUpdated;
            updated.textContent = `Updated: ${new Date(notebook.updated_at).toLocaleString()}`;
            meta.appendChild(updated);
        }

        // Tags
        if (notebook.tags && notebook.tags.length > 0) {
            const tags = document.createElement('div');
            tags.className = styles.notebookTags;

            for (const tag of notebook.tags) {
                const tagElement = document.createElement('span');
                tagElement.className = styles.notebookTag;
                tagElement.textContent = tag;
                tags.appendChild(tagElement);
            }

            meta.appendChild(tags);
        }

        header.appendChild(meta);
        container.appendChild(header);
    };

    /**
     * Render notebook content
     * @private
     */
    const renderNotebook = async () => {
        // Clear container
        clearContainer();

        // Create notebook title
        createHeader();

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
