import styles from './notebook-viewer.module.css';

/** @typedef {import('./index.js').RenderConfig} RenderConfig */

/**
 * @typedef {Object} ResolutionControl
 * @property {HTMLInputElement} widthInput - Width input element
 * @property {HTMLInputElement} heightInput - Height input element
 * @property {HTMLSelectElement} selectElement - Resolution select element
 * @property {function(number, number): void} update - Update resolution
 */

/**
 * @typedef {Object} UniformConfig
 * @property {string} name - Uniform name
 * @property {string} label - Uniform label
 * @property {string} type - Uniform type
 * @property {any} default - Default value
 * @property {number} [min] - Minimum value
 * @property {number} [max] - Maximum value
 * @property {number} [step] - Step value
 */

/**
 * Create resolution controls
 * @param {HTMLElement} container - Control container
 * @param {RenderConfig} renderConfig - Rendering configuration
 * @param {string} rendererId - Renderer ID
 * @param {Object} renderer - Renderer instance
 * @returns {ResolutionControl} Resolution control settings
 */
export function createResolutionControls(container, renderConfig, rendererId, renderer) {
    const controlsWrapper = document.createElement('div');
    controlsWrapper.className = styles.renderControls;

    // Resolution control title
    const controlsTitle = document.createElement('h4');
    controlsTitle.textContent = 'Resolution Settings';
    controlsTitle.className = styles.renderControlsTitle;
    controlsWrapper.appendChild(controlsTitle);

    // Resolution selector
    const resolutionSelector = document.createElement('div');
    resolutionSelector.className = styles.resolutionSelector;

    // Preset resolution options
    const presets = [
        { label: '320×240', width: 320, height: 240 },
        { label: '640×480', width: 640, height: 480 },
        { label: '800×600', width: 800, height: 600 },
        { label: '1280×720', width: 1280, height: 720 },
        { label: 'Custom', width: 0, height: 0 },
    ];

    // Create dropdown list
    const select = document.createElement('select');
    select.className = styles.resolutionSelect;

    // Add preset options
    presets.forEach((preset, index) => {
        const option = document.createElement('option');
        option.value = index;
        option.textContent = preset.label;
        select.appendChild(option);

        // If it matches the current resolution, select this option
        if (preset.width === renderConfig.width && preset.height === renderConfig.height) {
            select.selectedIndex = index;
        }
    });

    // Custom resolution inputs
    const customInputs = document.createElement('div');
    customInputs.className = styles.customResolution;

    const widthInput = document.createElement('input');
    widthInput.type = 'number';
    widthInput.min = '50';
    widthInput.max = '4096';
    widthInput.placeholder = 'Width';
    widthInput.value = renderConfig.width;
    widthInput.className = styles.resolutionInput;

    const separator = document.createElement('span');
    separator.textContent = '×';
    separator.className = styles.resolutionSeparator;

    const heightInput = document.createElement('input');
    heightInput.type = 'number';
    heightInput.min = '50';
    heightInput.max = '4096';
    heightInput.placeholder = 'Height';
    heightInput.value = renderConfig.height;
    heightInput.className = styles.resolutionInput;

    customInputs.appendChild(widthInput);
    customInputs.appendChild(separator);
    customInputs.appendChild(heightInput);

    // Apply button
    const applyButton = document.createElement('button');
    applyButton.textContent = 'Apply';
    applyButton.className = styles.resolutionApply;

    resolutionSelector.appendChild(select);
    resolutionSelector.appendChild(customInputs);
    resolutionSelector.appendChild(applyButton);

    // Dropdown list change event
    select.addEventListener('change', () => {
        const selectedIndex = select.selectedIndex;
        const preset = presets[selectedIndex];

        // If "Custom" option is selected, show custom inputs
        if (preset.width === 0 && preset.height === 0) {
            customInputs.style.display = 'flex';
        } else {
            // Otherwise, update input values
            widthInput.value = preset.width;
            heightInput.value = preset.height;
            // Keep custom inputs visible so the user can see current values
            customInputs.style.display = 'flex';
        }
    });

    // Apply button click event
    applyButton.addEventListener('click', () => {
        const width = parseInt(widthInput.value, 10);
        const height = parseInt(heightInput.value, 10);

        if (width >= 50 && height >= 50 && width <= 4096 && height <= 4096) {
            try {
                console.log(
                    `Applying new resolution: ${width}x${height}, Renderer ID: ${rendererId}`,
                );

                // Correctly use notebook-renderer API
                renderer.update({
                    config: {
                        width: width,
                        height: height,
                    },
                });

                // Find direct rendering container and update its CSS dimensions
                const renderContainer = document.getElementById(rendererId);
                if (renderContainer) {
                    renderContainer.style.width = `${width}px`;
                    renderContainer.style.height = `${height}px`;
                    console.log(`Updated container dimensions to: ${width}x${height}`);
                } else {
                    console.warn(`Rendering container not found: #${rendererId}`);
                }
            } catch (error) {
                console.error('Error applying resolution:', error);
                alert(`Resolution update failed: ${error.message}`);
            }
        } else {
            alert('Please enter valid resolution (width and height must be between 50-4096)');
        }
    });

    controlsWrapper.appendChild(resolutionSelector);

    // Add to container
    container.appendChild(controlsWrapper);

    // Return controller reference for subsequent operations
    return {
        widthInput,
        heightInput,
        selectElement: select,
        update: (width, height) => {
            widthInput.value = width;
            heightInput.value = height;

            // Try to match presets
            for (let i = 0; i < presets.length - 1; i++) {
                if (presets[i].width === width && presets[i].height === height) {
                    select.selectedIndex = i;
                    break;
                } else {
                    // If no matching preset, select "Custom"
                    select.selectedIndex = presets.length - 1;
                }
            }
        },
    };
}

/**
 * Create Uniform control panel
 * @param {HTMLElement} container - Control container
 * @param {RenderConfig} renderConfig - Rendering configuration
 * @param {string} rendererId - Renderer ID
 * @param {Object} renderer - Renderer instance
 */
export function createUniformControls(container, renderConfig, rendererId, renderer) {
    const uniformWrapper = document.createElement('div');
    uniformWrapper.className = styles.uniformControls;

    // Create Uniform control title
    const uniformTitle = document.createElement('h4');
    uniformTitle.textContent = 'Uniform Value Controls';
    uniformTitle.className = styles.renderControlsTitle;
    uniformWrapper.appendChild(uniformTitle);

    // Create default basic uniform controllers (time pause/resume)
    const timeControlRow = document.createElement('div');
    timeControlRow.className = styles.uniformControlRow;

    const timeLabel = document.createElement('div');
    timeLabel.textContent = 'Time Control:';
    timeLabel.className = styles.uniformLabel;

    const timeControl = document.createElement('div');
    timeControl.className = styles.timeControls;

    // Pause/Resume button
    const pauseButton = document.createElement('button');
    pauseButton.textContent = 'Pause';
    pauseButton.className = styles.uniformButton;

    let isPaused = false;
    pauseButton.addEventListener('click', () => {
        if (!renderer) return;

        if (isPaused) {
            // Resume renderer
            if (typeof renderer.resume === 'function') {
                renderer.resume();
                pauseButton.textContent = 'Pause';
            } else {
                console.warn('Renderer does not support resume method');
            }
        } else {
            // Pause renderer
            if (typeof renderer.pause === 'function') {
                renderer.pause();
                pauseButton.textContent = 'Resume';
            } else {
                console.warn('Renderer does not support pause method');
            }
        }
        isPaused = !isPaused;
    });

    // Reset button
    const resetButton = document.createElement('button');
    resetButton.textContent = 'Reset';
    resetButton.className = styles.uniformButton;
    resetButton.addEventListener('click', () => {
        if (renderer) {
            // Reset renderer
            if (typeof renderer.reset === 'function') {
                renderer.reset();
                // Reset pause state if needed
                if (isPaused) {
                    isPaused = false;
                    pauseButton.textContent = 'Pause';
                }
            } else {
                console.warn('Renderer does not support reset method');
            }
        }
    });

    timeControl.appendChild(pauseButton);
    timeControl.appendChild(resetButton);

    timeControlRow.appendChild(timeLabel);
    timeControlRow.appendChild(timeControl);
    uniformWrapper.appendChild(timeControlRow);

    // If render config has custom uniforms, add their controllers
    if (renderConfig.uniforms && Array.isArray(renderConfig.uniforms)) {
        renderConfig.uniforms.forEach((uniform) => {
            const uniformRow = createUniformControl(uniform, rendererId, renderer);
            if (uniformRow) {
                uniformWrapper.appendChild(uniformRow);
            }
        });
    }

    // Add to container
    container.appendChild(uniformWrapper);
}

/**
 * Create a single Uniform controller
 * @param {UniformConfig} uniform - Uniform configuration
 * @param {string} rendererId - Renderer ID
 * @param {Object} renderer - Renderer instance
 * @returns {HTMLElement|null} Controller element
 */
function createUniformControl(uniform, rendererId, renderer) {
    if (!uniform.name || !uniform.type) return null;

    const controlRow = document.createElement('div');
    controlRow.className = styles.uniformControlRow;

    const label = document.createElement('div');
    label.textContent = uniform.label || uniform.name;
    label.title = uniform.name;
    label.className = styles.uniformLabel;

    const control = document.createElement('div');
    control.className = styles.uniformControl;

    let inputElement;

    switch (uniform.type.toLowerCase()) {
        case 'float':
        case 'number':
            inputElement = document.createElement('input');
            inputElement.type = 'range';
            inputElement.min = uniform.min !== undefined ? uniform.min : 0;
            inputElement.max = uniform.max !== undefined ? uniform.max : 1;
            inputElement.step = uniform.step !== undefined ? uniform.step : 0.01;
            inputElement.value = uniform.default !== undefined ? uniform.default : 0.5;

            const valueDisplay = document.createElement('span');
            valueDisplay.className = styles.uniformValue;
            valueDisplay.textContent = inputElement.value;

            inputElement.addEventListener('input', () => {
                valueDisplay.textContent = inputElement.value;
                updateUniformValue(renderer, uniform.name, parseFloat(inputElement.value));
            });

            control.appendChild(inputElement);
            control.appendChild(valueDisplay);
            break;

        case 'bool':
        case 'boolean':
            inputElement = document.createElement('input');
            inputElement.type = 'checkbox';
            inputElement.checked = uniform.default || false;

            inputElement.addEventListener('change', () => {
                updateUniformValue(renderer, uniform.name, inputElement.checked);
            });

            control.appendChild(inputElement);
            break;

        case 'color':
            inputElement = document.createElement('input');
            inputElement.type = 'color';
            inputElement.value = uniform.default || '#ffffff';

            inputElement.addEventListener('input', () => {
                // Convert hex color to RGB array [0-1]
                const hexToRgb = (hex) => {
                    const r = parseInt(hex.slice(1, 3), 16) / 255;
                    const g = parseInt(hex.slice(3, 5), 16) / 255;
                    const b = parseInt(hex.slice(5, 7), 16) / 255;
                    return [r, g, b];
                };

                updateUniformValue(renderer, uniform.name, hexToRgb(inputElement.value));
            });

            control.appendChild(inputElement);
            break;

        case 'vec2':
        case 'vec3':
        case 'vec4':
            const dimension = parseInt(uniform.type.slice(3), 10);
            if (isNaN(dimension) || dimension < 2 || dimension > 4) {
                console.warn(`Invalid vector dimension: ${uniform.type}`);
                return null;
            }

            const components = ['X', 'Y', 'Z', 'W'].slice(0, dimension);

            const vecContainer = document.createElement('div');
            vecContainer.className = dimension === 4 ? styles.vec4Container : styles.vecContainer;

            const inputs = [];
            const defaultValues = uniform.default || Array(dimension).fill(0);

            const rowCount = dimension > 2 ? 2 : 1;
            const componentsPerRow = Math.ceil(dimension / rowCount);

            for (let row = 0; row < rowCount; row++) {
                const vecRow = document.createElement('div');
                vecRow.className = styles.vecRow;

                for (let i = 0; i < componentsPerRow; i++) {
                    const componentIndex = row * componentsPerRow + i;
                    if (componentIndex >= dimension) break;

                    const componentLabel = document.createElement('span');
                    componentLabel.textContent = `${components[componentIndex]}:`;
                    componentLabel.className = styles.vecLabel;
                    vecRow.appendChild(componentLabel);

                    const input = document.createElement('input');
                    input.type = 'number';
                    input.step = uniform.step || 0.1;
                    input.value =
                        defaultValues[componentIndex] !== undefined
                            ? defaultValues[componentIndex]
                            : 0;
                    input.className = styles.vecInput;
                    vecRow.appendChild(input);

                    inputs.push(input);
                }

                vecContainer.appendChild(vecRow);
            }

            const updateVec = () => {
                const values = inputs.map((input) => parseFloat(input.value));
                updateUniformValue(renderer, uniform.name, values);
            };

            inputs.forEach((input) => {
                input.addEventListener('change', updateVec);
            });

            control.appendChild(vecContainer);

            updateVec();
            break;

        default:
            console.warn(`Unsupported uniform type: ${uniform.type}`);
            return null;
    }

    controlRow.appendChild(label);
    controlRow.appendChild(control);

    return controlRow;
}

/**
 * Update Uniform value
 * @param {Object} renderer - Renderer instance
 * @param {string} name - Uniform name
 * @param {any} value - New value
 */
function updateUniformValue(renderer, name, value) {
    if (!renderer) return;

    // Use updateUniform if available
    if (typeof renderer.updateUniform === 'function') {
        renderer.updateUniform(name, value);
    } else if (typeof renderer.update === 'function') {
        // Fallback to update method with uniform parameter
        renderer.update({
            uniform: { name, value },
        });
    } else {
        console.warn(`Unable to update uniform ${name}: renderer API methods not available`);
    }
}

/**
 * Create renderer control panel
 * @param {HTMLElement} container - Parent container
 * @param {RenderConfig} renderConfig - Rendering configuration
 * @param {string} rendererId - Renderer ID
 * @param {Object} renderer - Renderer instance
 * @returns {HTMLElement} Control panel container
 */
export function createRendererControls(container, renderConfig, rendererId, renderer) {
    // Create control panel container
    const controlsContainer = document.createElement('div');
    controlsContainer.className = styles.renderControlsContainer;
    container.appendChild(controlsContainer);

    // Create resolution controls
    createResolutionControls(controlsContainer, renderConfig, rendererId, renderer);

    // Create Uniform controls
    createUniformControls(controlsContainer, renderConfig, rendererId, renderer);

    return controlsContainer;
}
