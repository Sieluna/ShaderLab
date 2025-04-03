import styles from './notebook-viewer.module.css';
import { createNotebookRenderer } from './notebook-renderer.js';
import { marked } from 'marked';

/**
 * Creates a notebook viewer
 * @param {HTMLElement} container - The container element
 * @param {Object} options - Viewer options
 * @returns {Object} Notebook viewer API
 */
export function createNotebookViewer(container, options = {}) {
    // Internal state
    let notebook = null;
    let renderers = new Map();
    
    // Merged options
    const viewerOptions = {
        renderMath: true,
        codeSyntaxHighlight: true,
        autoRunShaders: true,
        ...options,
    };
    
    /**
     * Clears the container
     */
    const clearContainer = () => {
        while (container.firstChild) {
            container.removeChild(container.firstChild);
        }
    };
    
    /**
     * Shows error message
     * @param {string} message - Error message
     */
    const showError = (message) => {
        clearContainer();
        
        const errorElement = document.createElement('div');
        errorElement.className = styles.notebookError;
        errorElement.textContent = message;
        
        container.appendChild(errorElement);
    };
    
    /**
     * Get cell type label
     * @param {string} cellType - Cell type
     * @returns {string} Cell type label
     */
    const getCellTypeLabel = (cellType) => {
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
    };
    
    /**
     * Render Markdown cell
     * @param {HTMLElement} container - Container element
     * @param {Object} cell - Cell data
     */
    const renderMarkdownCell = (container, cell) => {
        if (!cell.content) {
            container.textContent = 'Empty Markdown cell';
            return;
        }
        
        // Use marked library to render Markdown
        container.innerHTML = marked(cell.content);
    };
    
    /**
     * Render code cell
     * @param {HTMLElement} container - Container element
     * @param {Object} cell - Cell data
     */
    const renderCodeCell = (container, cell) => {
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
    };
    
    /**
     * Render shader cell
     * @param {HTMLElement} container - Container element
     * @param {Object} cell - Cell data
     */
    const renderShaderCell = (container, cell) => {
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
        if (viewerOptions.autoRunShaders) {
            // Delay rendering to ensure DOM is ready
            setTimeout(() => {
                try {
                    const renderer = createNotebookRenderer(rendererId, notebook, {
                        powerPreference: renderConfig.performance?.hardware_acceleration
                            ? 'high-performance'
                            : 'low-power',
                    });
                    
                    if (renderer) {
                        renderers.set(rendererId, renderer);
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
                    const renderer = createNotebookRenderer(rendererId, notebook, {
                        powerPreference: renderConfig.performance?.hardware_acceleration
                            ? 'high-performance'
                            : 'low-power',
                    });
                    
                    if (renderer) {
                        renderers.set(rendererId, renderer);
                    }
                } catch (error) {
                    console.error('Failed to create renderer:', error);
                    renderContainer.innerHTML = `<div class="${styles.errorMessage}">Renderer initialization failed: ${error.message}</div>`;
                }
            });
            
            renderContainer.appendChild(startButton);
        }
    };
    
    /**
     * Create cell element
     * @param {Object} cell - Cell data
     * @returns {HTMLElement} Cell element
     */
    const createCellElement = (cell) => {
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
                renderCodeCell(content, cell);
                break;
            case 'render':
                renderShaderCell(content, cell);
                break;
            default:
                content.textContent = `Unsupported cell type: ${cell.cell_type}`;
        }
        
        cellElement.appendChild(content);
        return cellElement;
    };
    
    /**
     * Create notebook header
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
        
        // Updated time
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
            typeof notebook.content === 'string'
                ? JSON.parse(notebook.content)
                : notebook.content;
            
        if (!content || !content.cells || !Array.isArray(content.cells)) {
            showError('Invalid notebook content format');
            return;
        }
        
        // Render cells
        for (const cell of content.cells) {
            const cellElement = createCellElement(cell);
            if (cellElement) {
                contentContainer.appendChild(cellElement);
            }
        }
    };
    
    // Initialize the container
    const init = () => {
        clearContainer();
        
        // Set container styles
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
         * @returns {Promise<boolean>} Whether loading was successful
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
            
            // Remove class names
            container.classList.remove(styles.notebookViewer);
        }
    };
}
