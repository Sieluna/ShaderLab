import styles from './debug.module.css';
import { createNotebookViewer } from '../components/index.js';
import { updateTestResult } from './debug.js';

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

export function createRendererTest() {
    const container = document.createElement('div');
    container.className = styles.section;

    const header = document.createElement('h2');
    header.textContent = 'Notebook Viewer Test';
    container.appendChild(header);

    const controls = document.createElement('div');
    controls.className = styles.controls;
    container.appendChild(controls);

    const viewerContainer = document.createElement('div');
    container.appendChild(viewerContainer);

    const resultDisplay = document.createElement('div');
    resultDisplay.id = 'viewer-test-result';
    container.appendChild(resultDisplay);

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

    const createViewerButton = document.createElement('button');
    createViewerButton.textContent = 'Create Viewer';
    createViewerButton.addEventListener('click', () => {
        try {
            if (notebookViewer) {
                notebookViewer.destroy();
                notebookViewer = null;
            }

            viewerContainer.innerHTML = '';

            notebookViewer = createNotebookViewer(viewerContainer, {
                autoRunShaders: false,
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
