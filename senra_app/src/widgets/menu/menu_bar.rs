use iced::advanced::layout::{Limits, Node};
use iced::advanced::widget::{Operation, Tree, tree};
use iced::advanced::{Clipboard, Layout, Shell, Widget};
use iced::advanced::{mouse, overlay, renderer};
use iced::{Element, Event, Length, Padding, Pixels, Rectangle, Size};
use iced::{alignment, event};

use super::flex;
use super::menu_bar_overlay::MenuBarOverlay;
use super::style::{Status, StyleFn};
use super::*;

#[derive(Debug, Default, Clone, Copy)]
pub struct MenuBarState {
    pub active_root: Index,
    pub open: bool,
    pub is_pressed: bool,
}

pub struct MenuBar<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    pub roots: Vec<Item<'a, Message, Theme, Renderer>>,
    pub spacing: Pixels,
    pub padding: Padding,
    pub width: Length,
    pub height: Length,
    pub check_bounds_width: f32,
    pub draw_path: DrawPath,
    pub scroll_speed: ScrollSpeed,
    pub class: Theme::Class<'a>,
}

impl<'a, Message, Theme, Renderer> MenuBar<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    pub fn new(mut roots: Vec<Item<'a, Message, Theme, Renderer>>) -> Self {
        for i in &mut roots {
            if let Some(m) = i.menu.as_mut() {
                m.axis = Axis::Vertical;
            }
        }

        Self {
            roots,
            spacing: Pixels::ZERO,
            padding: Padding::ZERO,
            width: Length::Shrink,
            height: Length::Shrink,
            check_bounds_width: 50.0,
            draw_path: DrawPath::FakeHovering,
            scroll_speed: ScrollSpeed {
                line: 60.0,
                pixel: 1.0,
            },
            class: Theme::default(),
        }
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    pub fn spacing(mut self, spacing: impl Into<Pixels>) -> Self {
        self.spacing = spacing.into();
        self
    }

    pub fn check_bounds_width(mut self, check_bounds_width: f32) -> Self {
        self.check_bounds_width = check_bounds_width;
        self
    }

    pub fn draw_path(mut self, draw_path: DrawPath) -> Self {
        self.draw_path = draw_path;
        self
    }

    pub fn scroll_speed(mut self, scroll_speed: ScrollSpeed) -> Self {
        self.scroll_speed = scroll_speed;
        self
    }

    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }

    pub fn style(mut self, style: impl Fn(&Theme, Status) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    pub fn class(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }
}
impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for MenuBar<'_, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }

    fn layout(&self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        flex::resolve(
            Axis::Horizontal,
            renderer,
            limits,
            self.width,
            self.height,
            self.padding,
            self.spacing,
            alignment::Alignment::Center,
            &self.roots.iter().map(|item| &item.item).collect::<Vec<_>>(),
            &mut tree
                .children
                .iter_mut()
                .map(|tree| &mut tree.children[0])
                .collect::<Vec<_>>(),
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        mut cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let styling = theme.style(&self.class, Status::Active);
        renderer.fill_quad(
            renderer::Quad {
                bounds: pad_rectangle(layout.bounds(), styling.bar_background_expand),
                border: styling.bar_border,
                shadow: styling.bar_shadow,
            },
            styling.bar_background,
        );

        let state = tree.state.downcast_ref::<MenuBarState>();
        if state.open {
            if let Some(active) = state.active_root {
                let Some(active_bounds) = layout.children().nth(active).map(|l| l.bounds()) else {
                    return;
                };

                match self.draw_path {
                    DrawPath::Backdrop => {
                        renderer.fill_quad(
                            renderer::Quad {
                                bounds: active_bounds,
                                border: styling.path_border,
                                ..Default::default()
                            },
                            styling.path,
                        );
                    }
                    DrawPath::FakeHovering => {
                        if !cursor.is_over(active_bounds) {
                            cursor = mouse::Cursor::Available(active_bounds.center());
                        }
                    }
                }
            }
        }

        self.roots
            .iter()
            .zip(tree.children.iter())
            .zip(layout.children())
            .for_each(|((item, tree), layout)| {
                item.draw(tree, renderer, theme, style, layout, cursor, viewport);
            });
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<MenuBarState>()
    }

    fn state(&self) -> tree::State {
        tree::State::Some(Box::<MenuBarState>::default())
    }

    fn children(&self) -> Vec<Tree> {
        self.roots.iter().map(Item::tree).collect::<Vec<_>>()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children_custom(&self.roots, |tree, item| item.diff(tree), Item::tree);
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation<()>,
    ) {
        operation.container(None, layout.bounds(), &mut |operation| {
            self.roots
                .iter()
                .zip(tree.children.iter_mut())
                .zip(layout.children())
                .for_each(|((child, state), layout)| {
                    child.operate(state, layout, renderer, operation);
                });
        });
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        use event::Status::*;

        let status = self
            .roots
            .iter_mut()
            .zip(tree.children.iter_mut())
            .zip(layout.children())
            .map(|((item, tree), layout)| {
                item.on_event(
                    tree,
                    event.clone(),
                    layout,
                    cursor,
                    renderer,
                    clipboard,
                    shell,
                    viewport,
                )
            })
            .fold(Ignored, event::Status::merge);

        let bar = tree.state.downcast_mut::<MenuBarState>();
        let bar_bounds = layout.bounds();

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if cursor.is_over(bar_bounds) {
                    bar.is_pressed = true;
                    Captured
                } else {
                    Ignored
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if cursor.is_over(bar_bounds) && bar.is_pressed {
                    bar.open = !bar.open;
                    bar.is_pressed = false;
                    for (i, l) in layout.children().enumerate() {
                        if cursor.is_over(l.bounds()) {
                            bar.active_root = Some(i);
                            break;
                        }
                    }
                    Captured
                } else {
                    Ignored
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if bar.open {
                    if cursor.is_over(bar_bounds) {
                        for (i, l) in layout.children().enumerate() {
                            if cursor.is_over(l.bounds()) {
                                bar.active_root = Some(i);
                                break;
                            }
                        }
                    } else {
                        bar.open = false;
                    }
                    Captured
                } else {
                    Ignored
                }
            }
            _ => Ignored,
        }
        .merge(status)
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.roots
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((item, tree), layout)| {
                item.mouse_interaction(tree, layout, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default()
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        translation: iced::Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let state = tree.state.downcast_mut::<MenuBarState>();

        let init_bar_bounds = layout.bounds();
        let init_root_bounds = layout.children().map(|l| l.bounds()).collect();

        if state.open {
            Some(
                MenuBarOverlay {
                    translation,
                    tree,
                    roots: &mut self.roots,
                    init_bar_bounds,
                    init_root_bounds,
                    check_bounds_width: self.check_bounds_width,
                    draw_path: &self.draw_path,
                    scroll_speed: self.scroll_speed,
                    class: &self.class,
                }
                .overlay_element(),
            )
        } else {
            None
        }
    }
}
impl<'a, Message, Theme, Renderer> From<MenuBar<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a + Catalog,
    Renderer: 'a + renderer::Renderer,
{
    fn from(value: MenuBar<'a, Message, Theme, Renderer>) -> Self {
        Self::new(value)
    }
}
