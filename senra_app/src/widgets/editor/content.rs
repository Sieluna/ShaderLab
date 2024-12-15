use std::cell::RefCell;
use std::fmt;

use iced::advanced::text::{self, Editor, editor};

pub struct Content<R = iced::Renderer>(pub RefCell<Internal<R>>)
where
    R: text::Renderer;

pub struct Internal<R>
where
    R: text::Renderer,
{
    pub editor: R::Editor,
    pub is_dirty: bool,
}

impl<R> Content<R>
where
    R: text::Renderer,
{
    pub fn new() -> Self {
        Self::with_text("")
    }

    pub fn with_text(text: &str) -> Self {
        Self(RefCell::new(Internal {
            editor: R::Editor::with_text(text),
            is_dirty: true,
        }))
    }

    pub fn perform(&mut self, action: editor::Action) {
        let internal = self.0.get_mut();
        internal.editor.perform(action);
        internal.is_dirty = true;
    }

    pub fn line_count(&self) -> usize {
        self.0.borrow().editor.line_count()
    }

    pub fn line(&self, index: usize) -> Option<impl std::ops::Deref<Target = str> + '_> {
        std::cell::Ref::filter_map(self.0.borrow(), |internal| internal.editor.line(index)).ok()
    }

    pub fn lines(&self) -> impl Iterator<Item = impl std::ops::Deref<Target = str> + '_> {
        struct Lines<'a, Renderer: text::Renderer> {
            internal: std::cell::Ref<'a, Internal<Renderer>>,
            current: usize,
        }

        impl<'a, Renderer: text::Renderer> Iterator for Lines<'a, Renderer> {
            type Item = std::cell::Ref<'a, str>;

            fn next(&mut self) -> Option<Self::Item> {
                let line =
                    std::cell::Ref::filter_map(std::cell::Ref::clone(&self.internal), |internal| {
                        internal.editor.line(self.current)
                    })
                    .ok()?;

                self.current += 1;
                Some(line)
            }
        }

        Lines {
            internal: self.0.borrow(),
            current: 0,
        }
    }

    pub fn text(&self) -> String {
        let mut text = self
            .lines()
            .enumerate()
            .fold(String::new(), |mut contents, (i, line)| {
                if i > 0 {
                    contents.push('\n');
                }
                contents.push_str(&line);
                contents
            });

        if !text.ends_with('\n') {
            text.push('\n');
        }

        text
    }

    pub fn selection(&self) -> Option<String> {
        self.0.borrow().editor.selection()
    }

    pub fn cursor_position(&self) -> (usize, usize) {
        self.0.borrow().editor.cursor_position()
    }
}

impl<Renderer> Default for Content<Renderer>
where
    Renderer: text::Renderer,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<Renderer> fmt::Debug for Content<Renderer>
where
    Renderer: text::Renderer,
    Renderer::Editor: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let internal = self.0.borrow();
        f.debug_struct("Content")
            .field("editor", &internal.editor)
            .field("is_dirty", &internal.is_dirty)
            .finish()
    }
}
