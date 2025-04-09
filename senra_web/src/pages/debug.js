import styles from './debug.module.css';
import { appState } from '../state.js';
import { notebookService, authService, userService } from '../services/index.js';
import { createNotebookViewer } from '../components/index.js';

function deepDiff(prev, curr, path = '') {
    const diffs = [];
    if (typeof prev !== 'object' || typeof curr !== 'object' || prev === null || curr === null) {
        if (prev !== curr) {
            diffs.push({ path, prev, curr });
        }
        return diffs;
    }
    if (Array.isArray(prev) || Array.isArray(curr)) {
        if (JSON.stringify(prev) !== JSON.stringify(curr)) {
            diffs.push({ path, prev, curr });
        }
        return diffs;
    }
    const allKeys = new Set([...Object.keys(prev), ...Object.keys(curr)]);
    for (const key of allKeys) {
        const currentPath = path ? `${path}.${key}` : key;
        const prevVal = prev.hasOwnProperty(key) ? prev[key] : undefined;
        const currVal = curr.hasOwnProperty(key) ? curr[key] : undefined;
        diffs.push(...deepDiff(prevVal, currVal, currentPath));
    }
    return diffs;
}

function formatContent(content, diffs = []) {
    const json = JSON.stringify(
        content,
        (_, value) => {
            if (Array.isArray(value) && value.length > 5) {
                return {
                    __collapsed: true,
                    length: value.length,
                    preview: value.slice(0, 5),
                };
            }
            return value;
        },
        2,
    );

    return json.replace(
        /{\s*"__collapsed":\s*true,\s*"length":\s*(\d+),\s*"preview":\s*(\[[\s\S]*?\])\s*}/g,
        (_match, len, preview) =>
            `<span class="collapsed-array" data-len="${len}" data-preview='${preview}'>[${preview.slice(1, -1)} … <em>${len} items</em>]</span>`,
    );
}

function createStateDisplay(id, state) {
    const display = document.createElement('div');
    display.id = id;
    display.className = styles.stateDisplay;

    const toggle = document.createElement('button');
    toggle.className = styles.stateToggle;
    toggle.textContent = '▼';

    const timestamp = document.createElement('div');
    timestamp.className = styles.stateTimestamp;

    const content = document.createElement('pre');
    let previousState = state.getState();

    const updateContent = (newState) => {
        const diffs = deepDiff(previousState, newState);
        previousState = newState;
        content.innerHTML = formatContent(newState, diffs);
        timestamp.textContent = `Last Update: ${new Date().toLocaleTimeString()}`;
    };

    toggle.addEventListener('click', () => {
        content.style.display = content.style.display === 'none' ? 'block' : 'none';
        toggle.textContent = content.style.display === 'none' ? '▶' : '▼';
    });

    state.subscribe(updateContent);
    updateContent(state.getState());

    display.append(toggle, timestamp, content);
    return display;
}

function updateTestResult(elementId, result) {
    const element = document.getElementById(elementId);
    if (element) {
        element.classList.add(styles.updated);
        setTimeout(() => element.classList.remove(styles.updated), 500);

        const statusColor = result?.error ? '#ff4444' : '#44ff44';
        element.style.borderLeft = `4px solid ${statusColor}`;

        element.innerHTML = `
            <div class="${styles.resultMeta}">
                <span>${new Date().toLocaleTimeString()}</span>
                ${result?.duration ? `<span>Duration: ${result.duration}ms</span>` : ''}
            </div>
            <pre>${formatContent(result)}</pre>
        `;
        element.scrollIntoView({ behavior: 'smooth' });
    }
}

function createInputForm(id, fields, resultId, submitAction) {
    const form = document.createElement('form');
    form.id = id;
    form.className = styles.inputForm;

    fields.forEach((field) => {
        const fieldContainer = document.createElement('div');
        fieldContainer.className = styles.formField;

        const label = document.createElement('label');
        label.htmlFor = `${id}-${field.name}`;
        label.textContent = field.label;

        const input = document.createElement('input');
        input.type = field.type || 'text';
        input.id = `${id}-${field.name}`;
        input.name = field.name;
        input.value = field.value || '';
        input.required = field.required || false;
        input.placeholder = field.placeholder || '';

        fieldContainer.append(label, input);
        form.appendChild(fieldContainer);
    });

    const submitBtn = document.createElement('button');
    submitBtn.type = 'submit';
    submitBtn.textContent = 'Submit';
    form.appendChild(submitBtn);

    form.addEventListener('submit', async (e) => {
        e.preventDefault();
        const start = performance.now();
        try {
            const formData = {};
            fields.forEach((field) => {
                formData[field.name] = form.querySelector(`#${id}-${field.name}`).value;
            });
            const result = await submitAction(formData);
            updateTestResult(resultId, { ...result, duration: performance.now() - start });
        } catch (error) {
            updateTestResult(resultId, { error: error.message });
        }
    });

    return form;
}

function createTestSection(title, tests) {
    const section = document.createElement('div');
    section.className = styles.testSection;
    const resultId = `${title.toLowerCase().replace(/\s+/g, '-')}-result`;

    const controls = tests
        .filter((test) => !test.formFields)
        .map((test) => `<button id="${test.id}">${test.label}</button>`)
        .join('');

    section.innerHTML = `
        <h2>${title}</h2>
        <div class="${styles.testControls}">${controls}</div>
        <div id="${resultId}" class="${styles.testResult}"></div>
    `;

    tests.forEach((test) => {
        if (test.formFields) {
            const form = createInputForm(`${test.id}-form`, test.formFields, resultId, test.action);
            const formContainer = document.createElement('div');
            formContainer.className = styles.formContainer;

            const formTitle = document.createElement('div');
            formTitle.className = styles.formTitle;
            formTitle.textContent = test.label;

            formContainer.appendChild(formTitle);
            formContainer.appendChild(form);

            section.querySelector(`.${styles.testControls}`).appendChild(formContainer);
        } else {
            section.querySelector(`#${test.id}`).addEventListener('click', async () => {
                const start = performance.now();
                try {
                    const result = await test.action();
                    updateTestResult(resultId, { ...result, duration: performance.now() - start });
                } catch (error) {
                    updateTestResult(resultId, { error: error.message });
                }
            });
        }
    });

    return section;
}

function createStateTest() {
    const container = document.createElement('div');
    container.className = styles.stateTest;

    const stateMonitor = document.createElement('div');
    stateMonitor.className = styles.stateMonitor;
    stateMonitor.innerHTML = '<h2>Real-time State Monitor</h2>';
    [
        { id: 'app-state', state: appState },
        { id: 'notebook-state', state: notebookService.notebookState },
    ].forEach(({ id, state }) => stateMonitor.appendChild(createStateDisplay(id, state)));

    const testConfig = {
        auth: [
            {
                id: 'test-login',
                label: 'User Login',
                formFields: [
                    { name: 'username', label: 'Username', value: 'test_user', required: true },
                    { name: 'password', label: 'Password', type: 'password', value: 'test_password', required: true },
                ],
                action: ({ username, password }) => authService.login(username, password),
            },
            {
                id: 'test-register',
                label: 'User Registration',
                formFields: [
                    { name: 'username', label: 'Username', value: 'test_user', required: true },
                    { name: 'email', label: 'Email', type: 'email', value: 'test_email@test.com', required: true },
                    { name: 'password', label: 'Password', type: 'password', value: 'test_password', required: true },
                ],
                action: ({ username, email, password }) => authService.register(username, email, password),
            },
            {
                id: 'test-check-auth',
                label: 'Check Authentication Status',
                action: authService.checkAuthStatus,
            },
        ],
        user: [
            {
                id: 'test-get-user',
                label: 'Get User Profile',
                formFields: [{ name: 'userId', label: 'User ID' }],
                action: ({ userId }) => userService.getUserProfile(userId),
            },
            {
                id: 'test-update-profile',
                label: 'Update Profile',
                formFields: [
                    { name: 'username', label: 'New Username', value: 'test_user_updated' },
                    { name: 'email', label: 'New Email', type: 'email', value: 'test_email_updated@test.com' },
                    { name: 'password', label: 'New Password', type: 'password', value: 'test_password_updated' },
                ],
                action: (data) => userService.updateUserProfile(data),
            },
        ],
        notebook: [
            {
                id: 'test-create-notebook',
                label: 'Create Notebook',
                formFields: [
                    { name: 'title', label: 'Title', value: 'Test Notebook', required: true },
                    { name: 'description', label: 'Description', value: 'This is a test notebook' },
                    { name: 'visibility', label: 'Visibility', value: 'public', placeholder: 'public/private' },
                ],
                action: (data) =>
                    notebookService.createNotebook({
                        ...data,
                        content: JSON.stringify({ cells: [] }),
                        tags: ['Test', 'Example'],
                        resources: [],
                        shaders: [],
                    }),
            },
            {
                id: 'test-update-notebook',
                label: 'Update Notebook',
                formFields: [
                    { name: 'id', label: 'Notebook ID', value: '', required: true },
                    { name: 'title', label: 'New Title', value: 'Updated Test Notebook' },
                    { name: 'description', label: 'New Description', value: 'This is an updated test notebook' },
                    { name: 'visibility', label: 'New Visibility', value: 'private' },
                ],
                action: ({ id, ...data }) => notebookService.updateNotebook(id, data),
            },
            {
                id: 'test-delete-notebook',
                label: 'Delete Notebook',
                formFields: [{ name: 'id', label: 'Notebook ID', value: '', required: true }],
                action: ({ id }) => notebookService.deleteNotebook(id),
            },
            {
                id: 'test-get-notebook',
                label: 'Get Notebook Details',
                formFields: [{ name: 'id', label: 'Notebook ID', value: '', required: true }],
                action: ({ id }) => notebookService.loadNotebookDetails(id),
            },
            {
                id: 'test-get-trending',
                label: 'Get Trending Notebooks',
                action: () => notebookService.loadTrendingNotebooks(),
            },
            {
                id: 'test-get-versions',
                label: 'Get Notebook Versions',
                formFields: [{ name: 'id', label: 'Notebook ID', value: '', required: true }],
                action: ({ id }) => notebookService.loadVersions(id),
            },
            {
                id: 'test-create-comment',
                label: 'Add Comment',
                formFields: [
                    { name: 'notebookId', label: 'Notebook ID', value: '', required: true },
                    { name: 'content', label: 'Comment Content', value: 'This is a test comment', required: true },
                ],
                action: ({ notebookId, content }) => notebookService.createComment(notebookId, content),
            },
            {
                id: 'test-get-comments',
                label: 'Get Comments',
                formFields: [
                    { name: 'notebookId', label: 'Notebook ID', value: '', required: true },
                ],
                action: ({ notebookId }) => notebookService.loadComments(notebookId),
            },
            {
                id: 'test-delete-comment',
                label: 'Delete Comment',
                formFields: [
                    { name: 'notebookId', label: 'Notebook ID', value: '', required: true },
                    { name: 'commentId', label: 'Comment ID', value: '', required: true },
                ],
                action: ({ notebookId, commentId }) => notebookService.deleteComment(notebookId, commentId),
            },
        ],
    };

    container.appendChild(createTestSection('Authentication', testConfig.auth));
    container.appendChild(createTestSection('User Service', testConfig.user));
    container.appendChild(createTestSection('Notebook Service', testConfig.notebook));
    container.appendChild(stateMonitor);

    return container;
}

const DEFAULT_VERTEX_SHADER = `
struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(0) uv: vec2f,
};

@group(0) @binding(0) var<uniform> time: vec4f;
@group(0) @binding(1) var<uniform> resolution: vec4f;

@vertex
fn main(@location(0) position: vec3f, @location(1) uv: vec2f) -> VertexOutput {
    var output: VertexOutput;
    output.position = vec4f(position, 1.0);
    output.uv = uv;
    return output;
}`;

const DEFAULT_FRAGMENT_SHADER = `
@group(0) @binding(0) var<uniform> time: vec4f;
@group(0) @binding(1) var<uniform> resolution: vec4f;

@fragment
fn main(@location(0) uv: vec2f) -> @location(0) vec4f {
    let color = vec3f(
        0.5 + 0.5 * sin(time.x + uv.x * 3.0),
        0.5 + 0.5 * sin(time.x * 0.7 + uv.y * 3.0),
        0.5 + 0.5 * sin(time.x * 1.3 + (uv.x + uv.y) * 3.0)
    );
    return vec4f(color, 1.0);
}`;

const POST_PROCESS_VERTEX_SHADER = `
struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(0) uv: vec2f,
};

@vertex
fn main(@location(0) position: vec3f, @location(1) uv: vec2f) -> VertexOutput {
    var output: VertexOutput;
    output.position = vec4f(position, 1.0);
    output.uv = uv;
    return output;
}`;

const POST_PROCESS_FRAGMENT_SHADER = `
@group(0) @binding(0) var<uniform> time: vec4f;
@group(0) @binding(1) var<uniform> resolution: vec4f;
@group(0) @binding(2) var<uniform> effects: vec4f;
@group(0) @binding(3) var<uniform> colorTint: vec4f;
@group(1) @binding(0) var inputTexture: texture_2d<f32>;
@group(1) @binding(1) var inputSampler: sampler;

@fragment
fn main(@location(0) uv: vec2f) -> @location(0) vec4f {
    let originalColor = textureSample(inputTexture, inputSampler, uv);

    let distortion = sin(time.x * 2.0 + uv.y * effects.y) * effects.x;
    let distortedUV = vec2f(uv.x + distortion, uv.y);

    let distortedColor = textureSample(inputTexture, inputSampler, distortedUV);

    let waveAmount = 0.3 + sin(time.x) * effects.z;
    var color = mix(originalColor, distortedColor, waveAmount);

    let distance = length(uv - 0.5);
    let vignette = 1.0 - distance * effects.w;
    color = color * vignette * colorTint;

    return color;
}`;

function createRendererTest() {
    const container = document.createElement('div');
    container.className = styles.rendererTest;

    // Add notebook viewer test section
    const header = document.createElement('h2');
    header.textContent = 'Notebook Viewer Test';
    container.appendChild(header);

    // Controls section
    const controls = document.createElement('div');
    controls.className = styles.testControls;
    container.appendChild(controls);

    // Add viewer container
    const viewerContainer = document.createElement('div');
    container.appendChild(viewerContainer);

    // Result display
    const resultDisplay = document.createElement('div');
    resultDisplay.id = 'viewer-test-result';
    container.appendChild(resultDisplay);

    // Create notebook viewer instance
    let notebookViewer = null;

    // Sample notebook data
    const sampleNotebook = {
        id: 'test-notebook-id',
        title: 'Test Notebook',
        description: 'This is a test notebook for the viewer component',
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
        tags: ['Test', 'Example', 'Viewer'],
        content: {
            cells: [
                {
                    id: 1,
                    cell_type: 'markdown',
                    content:
                        '# Shader Demo Notebook\n\n## This is a simplified notebook viewer demo\n\nThis demo implements notebook content rendering in a simplified way:\n\n* Markdown cells- parsed with **marked**\n* Code cells - show code\n* Render cells - run WebGPU shaders',
                    metadata: {
                        collapsed: false,
                    },
                },
                {
                    id: 2,
                    cell_type: 'code',
                    content: {
                        shader_ids: [1, 2, 3, 4],
                    },
                    metadata: {
                        collapsed: false,
                    },
                },
                {
                    id: 3,
                    cell_type: 'markdown',
                    content: '## Render Cell\n\nBelow is a render cell that uses a WebGPU shader:',
                },
                {
                    id: 4,
                    cell_type: 'render',
                    content: {
                        width: 800,
                        height: 600,
                        shader_ids: [1, 2, 3, 4],
                        resource_ids: [],
                        pipeline: {
                            shader_bindings: [
                                {
                                    shader_index: 0,
                                    shader_stage: 'vertex',
                                    entry_point: 'main',
                                },
                                {
                                    shader_index: 1,
                                    shader_stage: 'fragment',
                                    entry_point: 'main',
                                },
                                {
                                    shader_index: 2,
                                    shader_stage: 'vertex',
                                    entry_point: 'main',
                                },
                                {
                                    shader_index: 3,
                                    shader_stage: 'fragment',
                                    entry_point: 'main',
                                },
                            ],
                            vertex_attributes: [
                                {
                                    name: 'position',
                                    format: 'float32x3',
                                    offset: 0,
                                    stride: 20,
                                },
                                {
                                    name: 'uv',
                                    format: 'float32x2',
                                    offset: 12,
                                    stride: 20,
                                },
                            ],
                            resource_bindings: [],
                            render_passes: [
                                {
                                    id: 'render_to_texture',
                                    pass_type: 'intermediate',
                                    description: 'Render to intermediate texture',
                                    shader_bindings: [0, 1],
                                    clear_color: [0.1, 0.1, 0.1, 1.0],
                                    depth_enabled: false,
                                    output_textures: [
                                        {
                                            id: 'main_output',
                                            format: 'rgba8unorm',
                                            width_scale: 1.0,
                                            height_scale: 1.0,
                                        },
                                    ],
                                },
                                {
                                    id: 'post_process',
                                    pass_type: 'main',
                                    description: 'Post-process pass',
                                    shader_bindings: [2, 3],
                                    input_textures: [
                                        {
                                            texture_id: 'render_to_texture_main_output',
                                            group: 1,
                                            binding: 0,
                                        },
                                    ],
                                    clear_color: [0.0, 0.0, 0.0, 1.0],
                                    depth_enabled: false,
                                },
                            ],
                        },
                        uniforms: [
                            {
                                name: 'effects',
                                label: 'x: distortion strength, y: distortion frequency, z: wave strength, w: vignette strength',
                                type: 'vec4',
                                default: [0.02, 10.0, 0.2, 1.3],
                            },
                            {
                                name: 'colorTint',
                                label: 'color-tint adjustment',
                                type: 'vec4',
                                default: [1.0, 1.0, 1.0, 1.0],
                            },
                        ],
                        camera: {
                            position: [0, 0, 3],
                            target: [0, 0, 0],
                            up: [0, 1, 0],
                            fov: 45,
                            near: 0.1,
                            far: 100,
                        },
                        performance: {
                            hardware_acceleration: true,
                            antialias: true,
                            adaptive_resolution: true,
                            max_fps: 60,
                            debug: true,
                        },
                    },
                    metadata: {
                        collapsed: false,
                    },
                },
                {
                    cell_type: 'markdown',
                    content:
                        '## Summary\n\nThe notebook viewer supports multiple types of cells and uses the marked library to parse Markdown content. It also supports an embedded WebGPU renderer, enabling interactive visualization.',
                },
            ],
        },
        resources: [
            {
                id: 1,
                notebook_id: 1,
                name: 'Model',
                resource_type: 'gltf',
                data: [],
                metadata: null,
            },
        ],
        shaders: [
            {
                id: 1,
                notebook_id: 1,
                name: 'vertex-shader',
                shader_type: 'vertex',
                code: DEFAULT_VERTEX_SHADER,
            },
            {
                id: 2,
                notebook_id: 1,
                name: 'fragment-shader',
                shader_type: 'fragment',
                code: DEFAULT_FRAGMENT_SHADER,
            },
            {
                id: 3,
                notebook_id: 1,
                name: 'post-process-vertex',
                shader_type: 'vertex',
                code: POST_PROCESS_VERTEX_SHADER,
            },
            {
                id: 4,
                notebook_id: 1,
                name: 'post-process-fragment',
                shader_type: 'fragment',
                code: POST_PROCESS_FRAGMENT_SHADER,
            },
        ],
        visibility: 'public',
        version: 1,
    };

    // Create action buttons
    const createViewerButton = document.createElement('button');
    createViewerButton.textContent = 'Create Viewer';
    createViewerButton.addEventListener('click', () => {
        try {
            // Destroy existing viewer if present
            if (notebookViewer) {
                notebookViewer.destroy();
                notebookViewer = null;
            }

            // Clear container
            viewerContainer.innerHTML = '';

            // Create new viewer
            notebookViewer = createNotebookViewer(viewerContainer, {
                autoRunShaders: false, // Disable auto-run to prevent unnecessary GPU usage
            });

            updateTestResult('viewer-test-result', {
                action: 'Create Viewer',
                status: 'success',
                message: 'Notebook viewer created successfully',
            });
        } catch (error) {
            updateTestResult('viewer-test-result', {
                action: 'Create Viewer',
                status: 'error',
                error: error.message,
            });
        }
    });
    controls.appendChild(createViewerButton);

    const loadNotebookButton = document.createElement('button');
    loadNotebookButton.textContent = 'Load Sample Notebook';
    loadNotebookButton.addEventListener('click', async () => {
        if (!notebookViewer) {
            updateTestResult('viewer-test-result', {
                action: 'Load Notebook',
                status: 'error',
                error: 'Create a viewer first',
            });
            return;
        }

        try {
            const start = performance.now();
            const result = await notebookViewer.loadNotebook(sampleNotebook);

            updateTestResult('viewer-test-result', {
                action: 'Load Notebook',
                status: result ? 'success' : 'error',
                message: result ? 'Notebook loaded successfully' : 'Failed to load notebook',
                duration: performance.now() - start,
            });
        } catch (error) {
            updateTestResult('viewer-test-result', {
                action: 'Load Notebook',
                status: 'error',
                error: error.message,
            });
        }
    });
    controls.appendChild(loadNotebookButton);

    const loadInvalidNotebookButton = document.createElement('button');
    loadInvalidNotebookButton.textContent = 'Load Invalid Notebook';
    loadInvalidNotebookButton.addEventListener('click', async () => {
        if (!notebookViewer) {
            updateTestResult('viewer-test-result', {
                action: 'Load Invalid Notebook',
                status: 'error',
                error: 'Create a viewer first',
            });
            return;
        }

        try {
            const start = performance.now();
            // Create an invalid notebook with missing required fields
            const invalidNotebook = {
                title: 'Invalid Notebook',
                content: '{"cells": [{"cell_type": "invalid"}]}',
            };

            const result = await notebookViewer.loadNotebook(invalidNotebook);

            updateTestResult('viewer-test-result', {
                action: 'Load Invalid Notebook',
                status: 'completed',
                result: result,
                message: 'Test completed',
                duration: performance.now() - start,
            });
        } catch (error) {
            updateTestResult('viewer-test-result', {
                action: 'Load Invalid Notebook',
                status: 'error',
                error: error.message,
            });
        }
    });
    controls.appendChild(loadInvalidNotebookButton);

    const destroyViewerButton = document.createElement('button');
    destroyViewerButton.textContent = 'Destroy Viewer';
    destroyViewerButton.addEventListener('click', () => {
        if (!notebookViewer) {
            updateTestResult('viewer-test-result', {
                action: 'Destroy Viewer',
                status: 'error',
                error: 'No active viewer to destroy',
            });
            return;
        }

        try {
            notebookViewer.destroy();
            notebookViewer = null;

            updateTestResult('viewer-test-result', {
                action: 'Destroy Viewer',
                status: 'success',
                message: 'Notebook viewer destroyed successfully',
            });
        } catch (error) {
            updateTestResult('viewer-test-result', {
                action: 'Destroy Viewer',
                status: 'error',
                error: error.message,
            });
        }
    });
    controls.appendChild(destroyViewerButton);

    return container;
}

export function debugPage() {
    const debugContainer = document.createElement('div');
    debugContainer.className = styles.debugContainer;
    debugContainer.appendChild(createStateTest());
    debugContainer.appendChild(createRendererTest());
    return debugContainer;
}
