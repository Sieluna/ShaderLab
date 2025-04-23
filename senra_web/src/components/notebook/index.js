export { createNotebookViewer } from './notebook-viewer.js';

/**
 * @typedef {Object} Notebook
 * @property {string} id - Notebook ID
 * @property {string} title - Notebook title
 * @property {string} description - Notebook description
 * @property {string} created_at - Creation timestamp
 * @property {string} updated_at - Last update timestamp
 * @property {string[]} tags - Notebook tags
 * @property {NotebookContent|string} content - Notebook content
 * @property {NotebookResource[]} resources - Notebook resources
 * @property {NotebookShader[]} shaders - Notebook shaders
 * @property {string} visibility - Notebook visibility
 * @property {number} version - Notebook version
 */

/**
 * @typedef {Object} NotebookContent
 * @property {NotebookCell[]} cells - Array of notebook cells
 */

/**
 * @typedef {Object} NotebookCell
 * @property {number} id - Cell ID
 * @property {string} cell_type - Cell type ('markdown' | 'code' | 'render')
 * @property {string|Object} content - Cell content
 * @property {Object} [metadata] - Cell metadata
 */

/**
 * @typedef {Object} NotebookShader
 * @property {number} id - Shader ID
 * @property {number} notebook_id - Notebook ID
 * @property {string} name - Shader name
 * @property {string} shader_type - Shader type ('vertex' | 'fragment')
 * @property {string} code - Shader code
 */

/**
 * @typedef {Object} NotebookResource
 * @property {number} id - Resource ID
 * @property {number} notebook_id - Notebook ID
 * @property {string} name - Resource name
 * @property {string} resource_type - Resource type
 * @property {number[]} data - Resource data
 * @property {Object} [metadata] - Resource metadata
 */

/**
 * @typedef {Object} RenderConfig
 * @property {number} width - Render width
 * @property {number} height - Render height
 * @property {number[]} shader_ids - Shader IDs
 * @property {number[]} resource_ids - Resource IDs
 * @property {Object} pipeline - Pipeline configuration
 * @property {Object[]} uniforms - Uniform configurations
 * @property {Object} camera - Camera configuration
 * @property {Object} performance - Performance settings
 */

/**
 * @typedef {Object} ViewerOptions
 * @property {boolean} [renderMath=true] - Whether to render math expressions
 * @property {boolean} [codeSyntaxHighlight=true] - Whether to highlight code syntax
 * @property {boolean} [autoRunShaders=true] - Whether to automatically run shaders
 * @property {boolean} [readOnlyEditors=false] - Whether editors are read-only
 * @property {boolean} [enableShaderEditing=true] - Whether to enable shader editing
 */
