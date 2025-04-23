import styles from './shader-tabs.module.css';
import { createShaderEditor } from './shader-editor.js';

/**
 * @typedef {Object} ShaderTabsOptions
 * @property {boolean} [readOnly=false] - Whether the editor is read-only
 * @property {function(Object): void} [onChange] - Callback when shader content changes
 */

/**
 * Create shader tabs component
 * @param {HTMLElement} container - Container element
 * @param {Array} shaders - Array of shader objects
 * @param {Object} notebook - Notebook data
 * @param {ShaderTabsOptions} options - Component options
 * @returns {Object} Component API
 */
export function createShaderTabs(container, shaders, notebook, options = {}) {
    // Default options
    const componentOptions = {
        readOnly: false,
        onChange: null,
        ...options,
    };

    // Internal state
    let activeShader = shaders && shaders.length > 0 ? shaders[0] : null;
    let editors = new Map();

    // Create component wrapper
    const wrapper = document.createElement('div');
    wrapper.className = styles.editor;
    container.appendChild(wrapper);

    // Create tabs container
    const tabsContainer = document.createElement('div');
    tabsContainer.className = styles.tabs;
    wrapper.appendChild(tabsContainer);

    // Create content container
    const contentContainer = document.createElement('div');
    contentContainer.className = styles.container;
    wrapper.appendChild(contentContainer);

    // Create controls container if not readonly
    let controlsContainer = null;
    if (!componentOptions.readOnly) {
        controlsContainer = document.createElement('div');
        controlsContainer.className = styles.controls;

        // Add save button
        const saveButton = document.createElement('button');
        saveButton.className = `${styles.btn} ${styles.primary}`;
        saveButton.textContent = 'Save Changes';
        saveButton.addEventListener('click', () => {
            const activeEditor = activeShader ? editors.get(activeShader.id) : null;
            if (activeEditor?.editor && componentOptions.onChange) {
                componentOptions.onChange({
                    shader: activeShader,
                    content: activeEditor.editor.getContent(),
                    action: 'save',
                });
            }
        });

        // Add reset button
        const resetButton = document.createElement('button');
        resetButton.className = styles.btn;
        resetButton.textContent = 'Reset';
        resetButton.addEventListener('click', () => {
            const activeEditor = activeShader ? editors.get(activeShader.id) : null;
            if (activeEditor?.editor) {
                // Reset to original code
                activeEditor.editor.setContent(activeShader.code || '');
                if (componentOptions.onChange) {
                    componentOptions.onChange({
                        shader: activeShader,
                        content: activeShader.code || '',
                        action: 'reset',
                    });
                }
            }
        });

        controlsContainer.appendChild(resetButton);
        controlsContainer.appendChild(saveButton);
        wrapper.appendChild(controlsContainer);
    }

    /**
     * Create tab element
     * @param {Object} shader - Shader data
     * @returns {HTMLElement} Tab element
     */
    const createTab = (shader) => {
        const tab = document.createElement('div');
        tab.className = styles.tab;
        tab.dataset.shaderId = shader.id;
        if (shader.id === activeShader?.id) {
            tab.classList.add(styles.active);
        }

        // Create shader label
        const labelContainer = document.createElement('div');
        labelContainer.className = styles.label;

        // Add shader icon based on type
        const icon = document.createElement('span');
        icon.className = styles.icon;
        icon.textContent = shader.shader_type === 'vertex' ? '△' : '◆';
        icon.title = shader.shader_type === 'vertex' ? 'Vertex Shader' : 'Fragment Shader';
        labelContainer.appendChild(icon);

        // Add shader name
        const name = document.createElement('span');
        name.textContent = shader.name || `Shader ${shader.id}`;
        labelContainer.appendChild(name);

        tab.appendChild(labelContainer);

        // Add type indicator badge
        const typeIndicator = document.createElement('span');
        typeIndicator.className = styles.type;
        typeIndicator.textContent = shader.shader_type === 'vertex' ? 'VERT' : 'FRAG';
        tab.appendChild(typeIndicator);

        tab.addEventListener('click', () => {
            activateShader(shader);
        });

        return tab;
    };

    /**
     * Create editor for shader
     * @param {Object} shader - Shader data
     */
    const createEditor = (shader) => {
        if (editors.has(shader.id)) {
            return editors.get(shader.id);
        }

        const editorContainer = document.createElement('div');
        editorContainer.style.display = shader.id === activeShader?.id ? 'block' : 'none';
        editorContainer.style.height = '100%';

        // Add editor info header
        const editorInfo = document.createElement('div');
        editorInfo.className = styles.info;
        editorInfo.textContent = `${shader.name || `Shader ${shader.id}`} (${shader.shader_type === 'vertex' ? 'Vertex Shader' : 'Fragment Shader'})`;
        editorContainer.appendChild(editorInfo);

        contentContainer.appendChild(editorContainer);

        const editor = createShaderEditor(editorContainer, shader.code || '', {
            readOnly: componentOptions.readOnly,
            language: shader.shader_type === 'fragment' ? 'wgsl' : 'wgsl',
            onChange: (content) => {
                if (componentOptions.onChange) {
                    componentOptions.onChange({
                        shader,
                        content,
                        action: 'change',
                    });
                }
            },
        });

        editors.set(shader.id, {
            container: editorContainer,
            editor,
        });

        return editors.get(shader.id);
    };

    /**
     * Activate shader
     * @param {Object} shader - Shader to activate
     */
    const activateShader = (shader) => {
        if (!shader || shader.id === activeShader?.id) {
            return;
        }

        // Update active tab
        const tabs = tabsContainer.querySelectorAll(`.${styles.tab}`);
        tabs.forEach((tab) => {
            if (tab.dataset.shaderId === shader.id.toString()) {
                tab.classList.add(styles.active);
            } else {
                tab.classList.remove(styles.active);
            }
        });

        // Update editor visibility
        editors.forEach((editorInfo, shaderId) => {
            editorInfo.container.style.display = shaderId === shader.id ? 'block' : 'none';
        });

        activeShader = shader;

        // Create editor if needed
        if (!editors.has(shader.id)) {
            createEditor(shader);
        }
    };

    /**
     * Initialize component
     */
    const initialize = () => {
        // Create tabs
        if (shaders && shaders.length > 0) {
            shaders.forEach((shader) => {
                const tab = createTab(shader);
                tabsContainer.appendChild(tab);
            });

            // Create editor for active shader
            createEditor(activeShader);
        } else {
            const emptyMessage = document.createElement('div');
            emptyMessage.className = styles.empty;
            emptyMessage.textContent = 'No shaders available';
            contentContainer.appendChild(emptyMessage);
        }
    };

    // Initialize component
    initialize();

    // Return API
    return {
        /**
         * Get all editors
         * @returns {Map} Editors map
         */
        getEditors: () => editors,

        /**
         * Get active shader
         * @returns {Object} Active shader
         */
        getActiveShader: () => activeShader,

        /**
         * Set active shader
         * @param {number} shaderId - Shader ID
         */
        setActiveShader: (shaderId) => {
            const shader = shaders.find((s) => s.id === shaderId);
            if (shader) {
                activateShader(shader);
            }
        },

        /**
         * Get shader content
         * @param {number} shaderId - Shader ID
         * @returns {string} Shader content
         */
        getShaderContent: (shaderId) => {
            const editorInfo = editors.get(shaderId);
            if (editorInfo?.editor) {
                return editorInfo.editor.getContent();
            }
            return null;
        },

        /**
         * Update shader content
         * @param {number} shaderId - Shader ID
         * @param {string} content - New content
         */
        updateShaderContent: (shaderId, content) => {
            const editorInfo = editors.get(shaderId);
            if (editorInfo?.editor) {
                editorInfo.editor.setContent(content);
            }
        },

        /**
         * Destroy component
         */
        destroy: () => {
            // Clean up editors
            editors.forEach((editorInfo) => {
                if (editorInfo.editor && typeof editorInfo.editor.destroy === 'function') {
                    editorInfo.editor.destroy();
                }
            });
            editors.clear();

            // Remove element
            if (wrapper.parentElement) {
                wrapper.parentElement.removeChild(wrapper);
            }
        },
    };
}
