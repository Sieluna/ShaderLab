mod pipeline;
mod primitive;
mod uniforms;

use std::sync::Arc;
use std::time::Instant;

use iced::advanced::Shell;
use iced::widget::shader;
use iced::{Point, Rectangle, event, mouse, window};
use primitive::Primitive;
use uniforms::Uniforms;

pub struct Viewer {
    start: Instant,
    pub last_valid_shader: Arc<String>,
    pub version: usize,
}

impl Default for Viewer {
    fn default() -> Self {
        Self {
            start: Instant::now(),
            last_valid_shader: Arc::new(include_str!("shaders/default_frag.wgsl").to_string()),
            version: 0,
        }
    }
}

impl<Message> shader::Program<Message> for Viewer {
    type State = ();
    type Primitive = Primitive;

    fn update(
        &self,
        _state: &mut Self::State,
        _event: shader::Event,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
        shell: &mut Shell<'_, Message>,
    ) -> (event::Status, Option<Message>) {
        shell.request_redraw(window::RedrawRequest::NextFrame);

        (event::Status::Ignored, None)
    }

    fn draw(
        &self,
        _state: &Self::State,
        cursor: mouse::Cursor,
        bounds: Rectangle,
    ) -> Self::Primitive {
        Primitive {
            uniforms: Uniforms {
                time: Instant::now() - self.start,
                mouse: match cursor {
                    mouse::Cursor::Available(pt) => pt,
                    mouse::Cursor::Unavailable => Point::new(-1.0, -1.0),
                },
                bounds,
            },
            shader: self.last_valid_shader.clone(),
            version: self.version,
        }
    }
}
