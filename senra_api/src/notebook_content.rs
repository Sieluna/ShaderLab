use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Notebook content protocol for ShaderLab, optimized for WebGPU rendering
/// Similar to Jupyter notebook format but with specialized structures for shader rendering
#[cfg_attr(feature = "docs", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotebookContent {
    /// Version string for compatibility support
    pub version: String,
    /// List of cells in the notebook
    pub cells: Vec<Cell>,
    /// Metadata containing notebook-level configuration
    #[serde(default)]
    pub metadata: Value,
}

/// Represents a single cell in the notebook
#[cfg_attr(feature = "docs", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cell {
    /// Unique identifier for the cell
    pub id: String,
    /// Type of cell (markdown, code, or render)
    pub cell_type: CellType,
    /// Content of the cell, format depends on cell_type
    pub content: String,
    /// Metadata for the cell
    pub metadata: CellMetadata,
}

/// Types of cells supported in the notebook
#[cfg_attr(feature = "docs", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CellType {
    /// Markdown text content
    Markdown,
    /// Code content (e.g., WGSL shader code)
    Code,
    /// Render configuration for WebGPU visualization
    Render,
}

/// Metadata for a cell
#[cfg_attr(feature = "docs", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellMetadata {
    /// Whether the cell is collapsed in the UI
    #[serde(default)]
    pub collapsed: bool,
}

/// Configuration for WebGPU rendering in a render cell
#[cfg_attr(feature = "docs", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderConfig {
    /// Canvas width in pixels
    pub width: u32,
    /// Canvas height in pixels
    pub height: u32,
    /// References to shader IDs to be used in this render
    pub shader_ids: Vec<i64>,
    /// References to resource IDs to be used in this render
    pub resource_ids: Vec<i64>,
    /// Pipeline configuration including shaders, render passes, and resources
    pub pipeline: PipelineConfig,
    /// Camera configuration for 3D rendering
    pub camera: CameraConfig,
    /// Performance settings for the renderer
    pub performance: PerformanceConfig,
}

/// Configuration for the WebGPU rendering pipeline
#[cfg_attr(feature = "docs", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    /// Configuration of shader bindings, specifying which shaders to use and their stages
    pub shader_bindings: Vec<ShaderBinding>,
    /// Vertex attribute configurations for the pipeline
    pub vertex_attributes: Vec<VertexAttribute>,
    /// Resource binding configurations
    pub resource_bindings: Vec<ResourceBinding>,
    /// Render pass configurations defining the rendering sequence
    #[serde(default)]
    pub render_passes: Vec<RenderPassConfig>,
}

/// Configuration for binding a shader to a specific stage in the pipeline
#[cfg_attr(feature = "docs", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderBinding {
    /// Index into the shader_ids array to reference a specific shader
    pub shader_index: usize,
    /// Stage of the rendering pipeline where this shader will be used
    pub shader_stage: ShaderStage,
    /// Entry point function name in the shader code
    pub entry_point: String,
}

/// Available shader stages in the WebGPU pipeline
#[cfg_attr(feature = "docs", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ShaderStage {
    /// Vertex processing stage
    Vertex,
    /// Fragment (pixel) processing stage
    Fragment,
    /// Compute stage for general purpose GPU computation
    Compute,
}

/// Configuration for a vertex attribute in the pipeline
#[cfg_attr(feature = "docs", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VertexAttribute {
    /// Name of the attribute
    pub name: String,
    /// Format of the attribute data (e.g., "float32x3" for a vec3)
    pub format: String,
    /// Byte offset within the vertex buffer
    pub offset: u64,
    /// Byte stride between consecutive vertices
    pub stride: u64,
}

/// Configuration for binding a resource to the pipeline
#[cfg_attr(feature = "docs", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceBinding {
    /// Index into the resource_ids array to reference a specific resource
    pub resource_index: usize,
    /// Binding group index
    pub group: u32,
    /// Binding index within the group
    pub binding: u32,
    /// Type of binding (uniform, storage, texture, or sampler)
    pub binding_type: BindingType,
}

/// Types of bindings available in WebGPU
#[cfg_attr(feature = "docs", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BindingType {
    /// Uniform buffer binding
    Uniform,
    /// Storage buffer binding
    Storage,
    /// Texture binding
    Texture,
    /// Sampler binding
    Sampler,
}

/// Configuration for a render pass in the pipeline
#[cfg_attr(feature = "docs", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderPassConfig {
    /// Unique identifier for the pass, defaults to "main"
    #[serde(default = "default_pass_id")]
    pub id: String,
    /// Type of render pass (main, intermediate, postprocess, or compute)
    #[serde(default)]
    pub pass_type: RenderPassType,
    /// Optional description of the pass purpose
    #[serde(default)]
    pub description: Option<String>,
    /// Input texture bindings, used to receive textures from previous passes or resources
    #[serde(default)]
    pub input_textures: Vec<InputTextureBinding>,
    /// Output texture configurations, used to define render targets
    #[serde(default)]
    pub output_textures: Vec<OutputTextureConfig>,
    /// Optional geometry configuration, used to customize the draw call
    #[serde(default)]
    pub geometry: Option<GeometryConfig>,
    /// RGBA clear color for the render target
    #[serde(default = "default_clear_color")]
    pub clear_color: [f32; 4],
    /// Whether depth testing is enabled for this pass
    #[serde(default)]
    pub depth_enabled: bool,
    /// Clear value for the depth buffer
    #[serde(default = "default_clear_depth")]
    pub clear_depth: f32,
    /// Clear value for the stencil buffer
    #[serde(default)]
    pub clear_stencil: u32,
    /// Additional shader-specific parameters
    #[serde(default)]
    pub shader_parameters: Value,
}

/// Types of render passes supported in the pipeline
#[cfg_attr(feature = "docs", derive(utoipa::ToSchema))]
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RenderPassType {
    /// Main render pass, outputs to screen
    #[default]
    Main,
    /// Intermediate render pass, outputs to a texture
    Intermediate,
    /// Post-processing pass, typically applies effects to a previous pass result
    PostProcess,
    /// Compute pass for general-purpose computation
    Compute,
}

/// Configuration for binding an input texture to a render pass
#[cfg_attr(feature = "docs", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputTextureBinding {
    /// Texture ID, can be a pass ID, resource ID, or special value (e.g., "previous")
    pub texture_id: String,
    /// Binding group index
    pub group: u32,
    /// Binding index within the group
    pub binding: u32,
    /// Optional sampler configuration
    #[serde(default)]
    pub sampler_config: Option<SamplerConfig>,
}

/// Configuration for a texture sampler
#[cfg_attr(feature = "docs", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplerConfig {
    /// Magnification filter ("linear" or "nearest")
    #[serde(default = "default_filter_linear")]
    pub mag_filter: String,
    /// Minification filter ("linear" or "nearest")
    #[serde(default = "default_filter_linear")]
    pub min_filter: String,
    /// Texture addressing mode for U coordinate
    #[serde(default = "default_address_clamp")]
    pub address_mode_u: String,
    /// Texture addressing mode for V coordinate
    #[serde(default = "default_address_clamp")]
    pub address_mode_v: String,
}

/// Configuration for an output texture in a render pass
#[cfg_attr(feature = "docs", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputTextureConfig {
    /// Texture ID for referencing in subsequent passes
    #[serde(default = "default_output_id")]
    pub id: String,
    /// Texture format (e.g., "rgba8unorm")
    #[serde(default = "default_texture_format")]
    pub format: String,
    /// Width scale relative to the renderer width
    #[serde(default = "default_one_float")]
    pub width_scale: f32,
    /// Height scale relative to the renderer height
    #[serde(default = "default_one_float")]
    pub height_scale: f32,
    /// Optional blend configuration
    #[serde(default)]
    pub blend: Option<BlendConfig>,
}

/// Configuration for blending in render targets
#[cfg_attr(feature = "docs", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlendConfig {
    /// Source blend factor
    pub src_factor: String,
    /// Destination blend factor
    pub dst_factor: String,
    /// Blend operation
    #[serde(default = "default_blend_op")]
    pub operation: String,
}

/// Configuration for geometry in a render pass
#[cfg_attr(feature = "docs", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", tag = "type")]
pub enum GeometryConfig {
    /// Standard built-in geometry
    Standard {
        /// Type of primitive (e.g., "quad", "triangle")
        primitive: String,
    },
    /// Indexed geometry with custom indices
    Indexed {
        /// Number of indices to draw
        index_count: u32,
        /// Number of instances to draw
        #[serde(default = "default_one_u32")]
        instance_count: u32,
    },
    /// Non-indexed geometry with custom vertices
    NonIndexed {
        /// Number of vertices to draw
        vertex_count: u32,
        /// Number of instances to draw
        #[serde(default = "default_one_u32")]
        instance_count: u32,
    },
}

/// Configuration for a 3D camera
#[cfg_attr(feature = "docs", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraConfig {
    /// Camera position in 3D space
    pub position: [f32; 3],
    /// Camera look-at target in 3D space
    pub target: [f32; 3],
    /// Camera up direction vector
    pub up: [f32; 3],
    /// Field of view angle in degrees
    pub fov: f32,
    /// Near clipping plane distance
    pub near: f32,
    /// Far clipping plane distance
    pub far: f32,
}

/// Performance configuration for the renderer
#[cfg_attr(feature = "docs", derive(utoipa::ToSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Whether to use hardware acceleration
    #[serde(default = "default_true")]
    pub hardware_acceleration: bool,
    /// Whether to enable anti-aliasing
    #[serde(default = "default_true")]
    pub antialias: bool,
    /// Whether to automatically adjust resolution to maintain performance
    #[serde(default = "default_true")]
    pub adaptive_resolution: bool,
    /// Maximum frame rate (0 means unlimited)
    #[serde(default)]
    pub max_fps: u32,
}

fn default_true() -> bool {
    true
}

fn default_pass_id() -> String {
    "main".to_string()
}

fn default_output_id() -> String {
    "output".to_string()
}

fn default_texture_format() -> String {
    "rgba8unorm".to_string()
}

fn default_clear_color() -> [f32; 4] {
    [0.0, 0.0, 0.0, 1.0]
}

fn default_clear_depth() -> f32 {
    1.0
}

fn default_one_float() -> f32 {
    1.0
}

fn default_one_u32() -> u32 {
    1
}

fn default_filter_linear() -> String {
    "linear".to_string()
}

fn default_address_clamp() -> String {
    "clamp-to-edge".to_string()
}

fn default_blend_op() -> String {
    "add".to_string()
}
