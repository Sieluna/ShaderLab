use std::sync::Arc;

use iced::Rectangle;
use iced::advanced::graphics::Viewport;
use iced::widget::shader;
use iced::widget::shader::Storage;
use iced::widget::shader::wgpu::{CommandEncoder, Device, Queue, TextureFormat, TextureView};

use super::pipeline::Pipeline;
use super::uniforms::Uniforms;

#[derive(Debug)]
pub struct Primitive {
    pub uniforms: Uniforms,
    pub shader: Arc<String>,
    pub version: usize,
}

impl shader::Primitive for Primitive {
    fn prepare(
        &self,
        device: &Device,
        queue: &Queue,
        format: TextureFormat,
        storage: &mut Storage,
        _bounds: &Rectangle,
        viewport: &Viewport,
    ) {
        let should_store = storage
            .get::<Pipeline>()
            .map(|pipeline| pipeline.version < self.version)
            .unwrap_or(true);

        if should_store {
            storage.store(Pipeline::new(device, format, &self.shader, self.version));
        }

        let pipeline = storage.get_mut::<Pipeline>().unwrap();

        pipeline.prepare(
            queue,
            &self
                .uniforms
                .to_raw(viewport.scale_factor() as f32, viewport.projection()),
        );
    }

    fn render(
        &self,
        encoder: &mut CommandEncoder,
        storage: &Storage,
        target: &TextureView,
        clip_bounds: &Rectangle<u32>,
    ) {
        let pipeline = storage.get::<Pipeline>().unwrap();

        pipeline.render(encoder, target, clip_bounds);
    }
}
