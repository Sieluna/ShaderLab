use std::cell::RefCell;
use std::ops::DerefMut;
use std::sync::Arc;

use iced::advanced::text::editor::Editor as _;
use iced::advanced::text::{editor, highlighter};
use iced::advanced::{Shell, clipboard, layout, mouse, renderer, text, widget};
use iced::event::{self, Event};
use iced::time::{Duration, Instant};
use iced::{Element, Length, Padding, Pixels, Point, Rectangle, Size, alignment, window};

use super::bindings::{Binding, KeyPress, Update};
use super::content::Content;
use super::state::{Focus, State};
use super::style::{Catalog, Status};

pub struct TextEditor<'a, Highlighter, Message, Theme = iced::Theme, Renderer = iced::Renderer>
where
    Highlighter: text::Highlighter,
    Theme: Catalog,
    Renderer: text::Renderer,
{
    content: &'a Content<Renderer>,
    placeholder: Option<text::Fragment<'a>>,
    font: Option<Renderer::Font>,
    text_size: Option<Pixels>,
    line_height: text::LineHeight,
    width: Length,
    height: Length,
    padding: Padding,
    wrapping: text::Wrapping,
    class: Theme::Class<'a>,
    key_binding: Option<Box<dyn Fn(KeyPress) -> Option<Binding<Message>> + 'a>>,
    on_edit: Option<Box<dyn Fn(editor::Action) -> Message + 'a>>,
    highlighter_settings: Highlighter::Settings,
    highlighter_format: fn(&Highlighter::Highlight, &Theme) -> highlighter::Format<Renderer::Font>,
}

impl<'a, Message, Theme, Renderer> TextEditor<'a, highlighter::PlainText, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: text::Renderer,
{
    pub fn new(content: &'a Content<Renderer>) -> Self {
        Self {
            content,
            placeholder: None,
            font: None,
            text_size: None,
            line_height: text::LineHeight::default(),
            width: Length::Fill,
            height: Length::Shrink,
            padding: Padding::new(5.0),
            wrapping: text::Wrapping::default(),
            class: Theme::default(),
            key_binding: None,
            on_edit: None,
            highlighter_settings: (),
            highlighter_format: |_highlight, _theme| highlighter::Format::default(),
        }
    }
}

impl<'a, Highlighter, Message, Theme, Renderer>
    TextEditor<'a, Highlighter, Message, Theme, Renderer>
where
    Highlighter: text::Highlighter,
    Theme: Catalog,
    Renderer: text::Renderer,
{
    pub fn placeholder(mut self, placeholder: impl text::IntoFragment<'a>) -> Self {
        self.placeholder = Some(placeholder.into_fragment());
        self
    }

    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    pub fn width(mut self, width: impl Into<Pixels>) -> Self {
        self.width = Length::from(width.into());
        self
    }

    pub fn on_action(mut self, on_edit: impl Fn(editor::Action) -> Message + 'a) -> Self {
        self.on_edit = Some(Box::new(on_edit));
        self
    }

    pub fn font(mut self, font: impl Into<Renderer::Font>) -> Self {
        self.font = Some(font.into());
        self
    }

    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.text_size = Some(size.into());
        self
    }

    pub fn line_height(mut self, line_height: impl Into<text::LineHeight>) -> Self {
        self.line_height = line_height.into();
        self
    }

    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }

    pub fn wrapping(mut self, wrapping: text::Wrapping) -> Self {
        self.wrapping = wrapping;
        self
    }

    pub fn highlight<H: text::Highlighter>(
        self,
        settings: H::Settings,
        to_format: fn(&H::Highlight, &Theme) -> highlighter::Format<Renderer::Font>,
    ) -> TextEditor<'a, H, Message, Theme, Renderer> {
        TextEditor {
            content: self.content,
            placeholder: self.placeholder,
            font: self.font,
            text_size: self.text_size,
            line_height: self.line_height,
            width: self.width,
            height: self.height,
            padding: self.padding,
            wrapping: self.wrapping,
            class: self.class,
            key_binding: self.key_binding,
            on_edit: self.on_edit,
            highlighter_settings: settings,
            highlighter_format: to_format,
        }
    }

    pub fn key_binding(
        mut self,
        key_binding: impl Fn(KeyPress) -> Option<Binding<Message>> + 'a,
    ) -> Self {
        self.key_binding = Some(Box::new(key_binding));
        self
    }

    #[must_use]
    pub fn class(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }
}

impl<'a, Highlighter, Message, Theme, Renderer> widget::Widget<Message, Theme, Renderer>
    for TextEditor<'a, Highlighter, Message, Theme, Renderer>
where
    Highlighter: text::Highlighter,
    Theme: Catalog,
    Renderer: text::Renderer,
{
    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<State<Highlighter>>()
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(State {
            focus: None,
            last_click: None,
            drag_click: None,
            accumulate_scroll: 0.0,
            partial_scroll: 0.0,
            highlighter: RefCell::new(Highlighter::new(&self.highlighter_settings)),
            highlighter_settings: self.highlighter_settings.clone(),
            highlighter_format_address: self.highlighter_format as usize,
        })
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &self,
        tree: &mut widget::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let mut internal = self.content.0.borrow_mut();
        let state = tree.state.downcast_mut::<State<Highlighter>>();

        if state.highlighter_format_address != self.highlighter_format as usize {
            state.highlighter.borrow_mut().change_line(0);

            state.highlighter_format_address = self.highlighter_format as usize;
        }

        if state.highlighter_settings != self.highlighter_settings {
            state
                .highlighter
                .borrow_mut()
                .update(&self.highlighter_settings);

            state.highlighter_settings = self.highlighter_settings.clone();
        }

        let limits = limits.width(self.width).height(self.height);
        let text_size = self.text_size.unwrap_or_else(|| renderer.default_size());
        let font = self.font.unwrap_or_else(|| renderer.default_font());

        let line_count = internal.editor.line_count().max(1);
        let digit_count = (line_count as f32).log10().ceil() as usize;
        let line_number_width = text_size.0 * (digit_count as f32) + self.padding.horizontal();

        internal.editor.update(
            limits.shrink(self.padding).max(),
            font,
            text_size,
            self.line_height,
            self.wrapping,
            state.highlighter.borrow_mut().deref_mut(),
        );

        let editor_width = limits.max().width - line_number_width;
        let editor_height = match self.height {
            Length::Fill | Length::FillPortion(_) | Length::Fixed(_) => limits.max().height,
            Length::Shrink => {
                let min_bounds = internal.editor.min_bounds();
                min_bounds.height + self.padding.vertical()
            }
        };

        layout::Node::with_children(
            Size::new(limits.max().width, editor_height),
            vec![
                layout::Node::new(Size::new(line_number_width, editor_height)),
                layout::Node::new(Size::new(editor_width, editor_height))
                    .move_to(Point::new(line_number_width, 0.0)),
            ],
        )
    }

    fn on_event(
        &mut self,
        tree: &mut widget::Tree,
        event: Event,
        layout: layout::Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn clipboard::Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> event::Status {
        let Some(on_edit) = self.on_edit.as_ref() else {
            return event::Status::Ignored;
        };

        let state = tree.state.downcast_mut::<State<Highlighter>>();
        let children = layout.children();
        let editor_bounds = children.last().unwrap().bounds();

        match event {
            Event::Window(window::Event::Unfocused) => {
                if let Some(focus) = &mut state.focus {
                    focus.is_window_focused = false;
                }
            }
            Event::Window(window::Event::Focused) => {
                if let Some(focus) = &mut state.focus {
                    focus.is_window_focused = true;
                    focus.updated_at = Instant::now();

                    shell.request_redraw(window::RedrawRequest::NextFrame);
                }
            }
            Event::Window(window::Event::RedrawRequested(now)) => {
                if let Some(focus) = &mut state.focus {
                    if focus.is_window_focused {
                        focus.now = now;
                        let millis_until_redraw = Focus::CURSOR_BLINK_INTERVAL_MILLIS
                            - (now - focus.updated_at).as_millis()
                                % Focus::CURSOR_BLINK_INTERVAL_MILLIS;
                        shell.request_redraw(window::RedrawRequest::At(
                            now + Duration::from_millis(millis_until_redraw as u64),
                        ));
                    }
                }
            }
            _ => {}
        }

        let Some(update) = Update::from_event(
            event,
            state,
            editor_bounds,
            self.padding,
            cursor,
            self.key_binding.as_deref(),
        ) else {
            return event::Status::Ignored;
        };

        match update {
            Update::Click(click) => {
                let action = match click.kind() {
                    mouse::click::Kind::Single => editor::Action::Click(click.position()),
                    mouse::click::Kind::Double => editor::Action::SelectWord,
                    mouse::click::Kind::Triple => editor::Action::SelectLine,
                };

                state.focus = Some(Focus::now());
                state.last_click = Some(click);
                state.drag_click = Some(click.kind());

                shell.publish(on_edit(action));
            }
            Update::Drag(position) => {
                shell.publish(on_edit(editor::Action::Drag(position)));
            }
            Update::Release => {
                state.drag_click = None;
            }
            Update::Scroll(lines) => {
                let bounds = self.content.0.borrow().editor.bounds();
                let line_count = self.content.0.borrow().editor.line_count();

                if bounds.height >= i32::MAX as f32 {
                    return event::Status::Ignored;
                }

                let text_size = self.text_size.unwrap_or(renderer.default_size());
                let line_height = self.line_height.to_absolute(text_size).0;
                let max_scroll = (line_count as f32 - (bounds.height / line_height)).max(0.0);

                let lines = lines + state.partial_scroll;
                state.partial_scroll = lines.fract();
                state.accumulate_scroll = (state.accumulate_scroll + lines).clamp(0.0, max_scroll);

                shell.publish(on_edit(editor::Action::Scroll {
                    lines: lines as i32,
                }));
            }
            Update::Binding(binding) => {
                fn apply_binding<H: text::Highlighter, R: text::Renderer, Message>(
                    binding: Binding<Message>,
                    content: &Content<R>,
                    state: &mut State<H>,
                    on_edit: &dyn Fn(editor::Action) -> Message,
                    clipboard: &mut dyn clipboard::Clipboard,
                    shell: &mut Shell<'_, Message>,
                ) {
                    let mut publish = |action| shell.publish(on_edit(action));

                    match binding {
                        Binding::Unfocus => {
                            state.focus = None;
                            state.drag_click = None;
                        }
                        Binding::Copy => {
                            if let Some(selection) = content.selection() {
                                clipboard.write(clipboard::Kind::Standard, selection);
                            }
                        }
                        Binding::Cut => {
                            if let Some(selection) = content.selection() {
                                clipboard.write(clipboard::Kind::Standard, selection);
                                publish(editor::Action::Edit(editor::Edit::Delete));
                            }
                        }
                        Binding::Paste => {
                            if let Some(contents) = clipboard.read(clipboard::Kind::Standard) {
                                publish(editor::Action::Edit(editor::Edit::Paste(Arc::new(
                                    contents,
                                ))));
                            }
                        }
                        Binding::Move(motion) => {
                            publish(editor::Action::Move(motion));
                        }
                        Binding::Select(motion) => {
                            publish(editor::Action::Select(motion));
                        }
                        Binding::SelectWord => {
                            publish(editor::Action::SelectWord);
                        }
                        Binding::SelectLine => {
                            publish(editor::Action::SelectLine);
                        }
                        Binding::SelectAll => {
                            publish(editor::Action::SelectAll);
                        }
                        Binding::Insert(c) => {
                            publish(editor::Action::Edit(editor::Edit::Insert(c)));
                        }
                        Binding::Enter => {
                            publish(editor::Action::Edit(editor::Edit::Enter));
                        }
                        Binding::Backspace => {
                            publish(editor::Action::Edit(editor::Edit::Backspace));
                        }
                        Binding::Delete => {
                            publish(editor::Action::Edit(editor::Edit::Delete));
                        }
                        Binding::Sequence(sequence) => {
                            for binding in sequence {
                                apply_binding(binding, content, state, on_edit, clipboard, shell);
                            }
                        }
                        Binding::Custom(message) => {
                            shell.publish(message);
                        }
                    }
                }

                apply_binding(binding, self.content, state, on_edit, clipboard, shell);

                if let Some(focus) = &mut state.focus {
                    focus.updated_at = Instant::now();
                }
            }
        }

        event::Status::Captured
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _defaults: &renderer::Style,
        layout: layout::Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        let mut internal = self.content.0.borrow_mut();
        let state = tree.state.downcast_ref::<State<Highlighter>>();

        let font = self.font.unwrap_or_else(|| renderer.default_font());
        let text_size = self.text_size.unwrap_or_else(|| renderer.default_size());

        internal.editor.highlight(
            font,
            state.highlighter.borrow_mut().deref_mut(),
            |highlight| (self.highlighter_format)(highlight, theme),
        );

        let is_disabled = self.on_edit.is_none();
        let is_mouse_over = cursor.is_over(bounds);

        let status = if is_disabled {
            Status::Disabled
        } else if state.focus.is_some() {
            Status::Focused
        } else if is_mouse_over {
            Status::Hovered
        } else {
            Status::Active
        };

        let style = theme.style(&self.class, status);

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: style.border,
                ..renderer::Quad::default()
            },
            style.background,
        );

        let mut children = layout.children();
        let line_number_bounds = children.next().unwrap().bounds().shrink(self.padding);
        let editor_bounds = children.next().unwrap().bounds().shrink(self.padding);

        let line_count = internal.editor.line_count();
        let digit_count = (line_count as f32).log10().ceil() as usize;
        let line_height = self.line_height.to_absolute(text_size).0;

        for i in 0..line_count {
            let y = line_number_bounds.y + (i as f32 - state.accumulate_scroll) * line_height;

            if y + line_height >= line_number_bounds.y
                && y <= line_number_bounds.y + line_number_bounds.height
            {
                renderer.fill_text(
                    text::Text {
                        content: format!("{:>width$}", i + 1, width = digit_count),
                        bounds: Size::new(line_number_bounds.width, line_height),
                        size: text_size,
                        line_height: self.line_height,
                        font,
                        horizontal_alignment: alignment::Horizontal::Left,
                        vertical_alignment: alignment::Vertical::Top,
                        shaping: text::Shaping::Advanced,
                        wrapping: text::Wrapping::None,
                    },
                    Point::new(line_number_bounds.x, y),
                    style.line_number,
                    line_number_bounds,
                );
            }
        }

        if internal.editor.is_empty() {
            if let Some(placeholder) = self.placeholder.clone() {
                renderer.fill_text(
                    text::Text {
                        content: placeholder.into_owned(),
                        bounds: editor_bounds.size(),
                        size: text_size,
                        line_height: self.line_height,
                        font,
                        horizontal_alignment: alignment::Horizontal::Left,
                        vertical_alignment: alignment::Vertical::Top,
                        shaping: text::Shaping::Advanced,
                        wrapping: self.wrapping,
                    },
                    editor_bounds.position(),
                    style.placeholder,
                    editor_bounds,
                );
            }
        } else {
            renderer.fill_editor(
                &internal.editor,
                editor_bounds.position(),
                style.value,
                editor_bounds,
            );
        }

        let translation = editor_bounds.position() - Point::ORIGIN;

        if let Some(focus) = state.focus.as_ref() {
            match internal.editor.cursor() {
                editor::Cursor::Caret(position) if focus.is_cursor_visible() => {
                    let cursor = Rectangle::new(
                        position + translation,
                        Size::new(1.0, self.line_height.to_absolute(text_size).into()),
                    );

                    if let Some(clipped_cursor) = editor_bounds.intersection(&cursor) {
                        renderer.fill_quad(
                            renderer::Quad {
                                bounds: clipped_cursor,
                                ..renderer::Quad::default()
                            },
                            style.value,
                        );
                    }
                }
                editor::Cursor::Selection(ranges) => {
                    for range in ranges
                        .into_iter()
                        .filter_map(|range| editor_bounds.intersection(&(range + translation)))
                    {
                        renderer.fill_quad(
                            renderer::Quad {
                                bounds: range,
                                ..renderer::Quad::default()
                            },
                            style.selection,
                        );
                    }
                }
                editor::Cursor::Caret(_) => {}
            }
        }
    }

    fn mouse_interaction(
        &self,
        _state: &widget::Tree,
        layout: layout::Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let is_disabled = self.on_edit.is_none();

        if cursor.is_over(layout.bounds()) {
            if is_disabled {
                mouse::Interaction::NotAllowed
            } else {
                mouse::Interaction::Text
            }
        } else {
            mouse::Interaction::default()
        }
    }

    fn operate(
        &self,
        tree: &mut widget::Tree,
        _layout: layout::Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        let state = tree.state.downcast_mut::<State<Highlighter>>();

        operation.focusable(state, None);
    }
}

impl<'a, Highlighter, Message, Theme, Renderer>
    From<TextEditor<'a, Highlighter, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Highlighter: text::Highlighter,
    Message: 'a,
    Theme: Catalog + 'a,
    Renderer: text::Renderer,
{
    fn from(text_editor: TextEditor<'a, Highlighter, Message, Theme, Renderer>) -> Self {
        Self::new(text_editor)
    }
}
