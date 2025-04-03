import styles from './notebook-viewer.module.css';
import { createNotebookRenderer } from './notebook-renderer.js';
import { marked } from 'marked';

export class NotebookViewer {
    constructor(container, options = {}) {
        this.container = container;
        this.options = options;
        this.notebook = null;
        this.renderers = new Map();

        // Default options
        this.options = {
            theme: 'light',
            renderMath: true,
            codeSyntaxHighlight: true,
            autoRunShaders: true,
            ...options,
        };

        // Initialize container
        this._initContainer();
    }

    /**
     * Initialize container
     * @private
     */
    _initContainer() {
        // Clear container
        while (this.container.firstChild) {
            this.container.removeChild(this.container.firstChild);
        }

        // Set container styles
        this.container.classList.add(styles.notebookViewer);
        if (this.options.theme) {
            this.container.classList.add(
                styles[
                    `theme${this.options.theme.charAt(0).toUpperCase() + this.options.theme.slice(1)}`
                ],
            );
        }

        // Create loading indicator
        const loadingIndicator = document.createElement('div');
        loadingIndicator.className = styles.notebookLoading;
        loadingIndicator.textContent = 'Loading...';
        this.container.appendChild(loadingIndicator);
    }

    /**
     * Load notebook data
     * @param {Object|string} notebook - Notebook data object or JSON string
     * @returns {Promise<boolean>} Whether loading was successful
     */
    async loadNotebook(notebook) {
        try {
            // Parse notebook data
            if (typeof notebook === 'string') {
                try {
                    notebook = JSON.parse(notebook);
                } catch (e) {
                    console.error('Notebook JSON parsing failed:', e);
                    this._showError('Invalid notebook format');
                    return false;
                }
            }

            this.notebook = notebook;

            // Render notebook
            await this._renderNotebook();
            return true;
        } catch (error) {
            console.error('Failed to load notebook:', error);
            this._showError(`Failed to load notebook: ${error.message}`);
            return false;
        }
    }

    /**
     * Render notebook content
     * @private
     */
    async _renderNotebook() {
        // Clear container
        while (this.container.firstChild) {
            this.container.removeChild(this.container.firstChild);
        }

        // Create notebook title
        this._createHeader();

        // Create notebook content container
        const contentContainer = document.createElement('div');
        contentContainer.className = styles.notebookContent;
        this.container.appendChild(contentContainer);

        // Parse content
        const content =
            typeof this.notebook.content === 'string'
                ? JSON.parse(this.notebook.content)
                : this.notebook.content;

        if (!content || !content.cells || !Array.isArray(content.cells)) {
            this._showError('Invalid notebook content format');
            return;
        }

        // Render cells
        for (const cell of content.cells) {
            const cellElement = this._createCellElement(cell);
            if (cellElement) {
                contentContainer.appendChild(cellElement);
            }
        }
    }

    /**
     * Create notebook header
     * @private
     */
    _createHeader() {
        const header = document.createElement('div');
        header.className = styles.notebookHeader;

        // Title
        const title = document.createElement('h1');
        title.className = styles.notebookTitle;
        title.textContent = this.notebook.title || 'Untitled Notebook';
        header.appendChild(title);

        // Description
        if (this.notebook.description) {
            const description = document.createElement('div');
            description.className = styles.notebookDescription;
            description.textContent = this.notebook.description;
            header.appendChild(description);
        }

        // Metadata
        const meta = document.createElement('div');
        meta.className = styles.notebookMeta;

        // Author
        if (this.notebook.author) {
            const author = document.createElement('span');
            author.className = styles.notebookAuthor;
            author.textContent = `Author: ${this.notebook.author.username || 'Unknown'}`;
            meta.appendChild(author);
        }

        // Updated time
        if (this.notebook.updated_at) {
            const updated = document.createElement('span');
            updated.className = styles.notebookUpdated;
            updated.textContent = `Updated: ${new Date(this.notebook.updated_at).toLocaleString()}`;
            meta.appendChild(updated);
        }

        // Tags
        if (this.notebook.tags && this.notebook.tags.length > 0) {
            const tags = document.createElement('div');
            tags.className = styles.notebookTags;

            for (const tag of this.notebook.tags) {
                const tagElement = document.createElement('span');
                tagElement.className = styles.notebookTag;
                tagElement.textContent = tag;
                tags.appendChild(tagElement);
            }

            meta.appendChild(tags);
        }

        header.appendChild(meta);
        this.container.appendChild(header);
    }

    /**
     * Create cell element
     * @param {Object} cell - Cell data
     * @returns {HTMLElement} Cell element
     * @private
     */
    _createCellElement(cell) {
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
        typeIndicator.textContent = this._getCellTypeLabel(cell.cell_type);
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
                this._renderMarkdownCell(content, cell);
                break;
            case 'code':
                this._renderCodeCell(content, cell);
                break;
            case 'render':
                this._renderShaderCell(content, cell);
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
    _getCellTypeLabel(cellType) {
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

    /**
     * Render Markdown cell
     * @param {HTMLElement} container - Container element
     * @param {Object} cell - Cell data
     * @private
     */
    _renderMarkdownCell(container, cell) {
        if (!cell.content) {
            container.textContent = 'Empty Markdown cell';
            return;
        }

        // Use marked library to render Markdown
        container.innerHTML = marked(cell.content);
    }

    /**
     * Render code cell
     * @param {HTMLElement} container - Container element
     * @param {Object} cell - Cell data
     * @private
     */
    _renderCodeCell(container, cell) {
        if (!cell.content) {
            container.textContent = 'Empty code cell';
            return;
        }

        // Create code block
        const pre = document.createElement('pre');
        const code = document.createElement('code');

        // Convert object content to string if necessary
        const codeContent =
            typeof cell.content === 'object' ? JSON.stringify(cell.content, null, 2) : cell.content;

        code.textContent = codeContent;
        pre.appendChild(code);
        container.appendChild(pre);
    }

    /**
     * Render shader cell
     * @param {HTMLElement} container - Container element
     * @param {Object} cell - Cell data
     * @private
     */
    _renderShaderCell(container, cell) {
        if (!cell.content) {
            container.textContent = 'Empty render cell';
            return;
        }

        // Parse content
        const renderConfig =
            typeof cell.content === 'string' ? JSON.parse(cell.content) : cell.content;

        // Create render container
        const renderContainer = document.createElement('div');
        renderContainer.className = styles.renderContainer;
        renderContainer.style.width = `${renderConfig.width || 400}px`;
        renderContainer.style.height = `${renderConfig.height || 300}px`;
        container.appendChild(renderContainer);

        // Generate unique ID
        const rendererId = `renderer-${cell.id || Math.random().toString(36).substring(2, 9)}`;
        renderContainer.id = rendererId;

        // If auto-run shaders is enabled, create renderer
        if (this.options.autoRunShaders) {
            // Delay rendering to ensure DOM is ready
            setTimeout(() => {
                try {
                    const renderer = createNotebookRenderer(rendererId, this.notebook, {
                        powerPreference: renderConfig.performance?.hardware_acceleration
                            ? 'high-performance'
                            : 'low-power',
                    });

                    if (renderer) {
                        this.renderers.set(rendererId, renderer);
                    }
                } catch (error) {
                    console.error('Failed to create renderer:', error);
                    renderContainer.innerHTML = `<div class="${styles.errorMessage}">Renderer initialization failed: ${error.message}</div>`;
                }
            }, 100);
        } else {
            // Show start button
            const startButton = document.createElement('button');
            startButton.className = styles.renderStartButton;
            startButton.textContent = 'Start Renderer';
            startButton.addEventListener('click', () => {
                startButton.remove();

                try {
                    const renderer = createNotebookRenderer(rendererId, this.notebook, {
                        powerPreference: renderConfig.performance?.hardware_acceleration
                            ? 'high-performance'
                            : 'low-power',
                    });

                    if (renderer) {
                        this.renderers.set(rendererId, renderer);
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
     * Show error message
     * @param {string} message - Error message
     * @private
     */
    _showError(message) {
        // Clear container
        while (this.container.firstChild) {
            this.container.removeChild(this.container.firstChild);
        }

        // Create error message
        const errorElement = document.createElement('div');
        errorElement.className = styles.notebookError;
        errorElement.textContent = message;

        this.container.appendChild(errorElement);
    }

    /**
     * Destroy viewer
     */
    destroy() {
        // Destroy all renderers
        for (const [, renderer] of this.renderers) {
            renderer.destroy();
        }
        this.renderers.clear();

        // Clear container
        while (this.container.firstChild) {
            this.container.removeChild(this.container.firstChild);
        }

        // Remove class names
        this.container.classList.remove(styles.notebookViewer);
        if (this.options.theme) {
            this.container.classList.remove(
                styles[
                    `theme${this.options.theme.charAt(0).toUpperCase() + this.options.theme.slice(1)}`
                ],
            );
        }
    }
}
