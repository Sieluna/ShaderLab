use iced::advanced::layout::{Limits, Node};
use iced::advanced::{renderer, widget};
use iced::{Alignment, Element, Length, Padding, Pixels, Point, Size};

use super::Axis;

pub fn resolve<'a, E, T, Message, Theme, Renderer>(
    axis: Axis,
    renderer: &Renderer,
    limits: &Limits,
    width: Length,
    height: Length,
    padding: Padding,
    spacing: Pixels,
    align_items: Alignment,
    items: &[E],
    trees: &mut [T],
) -> Node
where
    E: std::borrow::Borrow<Element<'a, Message, Theme, Renderer>>,
    T: std::borrow::BorrowMut<widget::Tree>,
    Renderer: renderer::Renderer,
{
    let limits = limits.width(width).height(height).shrink(padding);
    let total_spacing = spacing * items.len().saturating_sub(1) as f32;
    let max_cross = axis.cross(limits.max());

    let mut fill_main_sum = 0;
    let mut cross = match axis {
        Axis::Horizontal => match height {
            Length::Shrink => 0.0,
            _ => max_cross,
        },
        Axis::Vertical => match width {
            Length::Shrink => 0.0,
            _ => max_cross,
        },
    };

    let mut available = axis.main(limits.max()) - total_spacing.0;

    let mut nodes: Vec<Node> = Vec::with_capacity(items.len());
    nodes.resize(items.len(), Node::default());

    for (i, (child, tree)) in items.iter().zip(trees.iter_mut()).enumerate() {
        let (fill_main_factor, fill_cross_factor) = {
            let size = child.borrow().as_widget().size();

            axis.pack(size.width.fill_factor(), size.height.fill_factor())
        };

        if fill_main_factor == 0 {
            if fill_cross_factor == 0 {
                let (max_width, max_height) = axis.pack(available, max_cross);

                let child_limits = Limits::new(Size::ZERO, Size::new(max_width, max_height));

                let layout =
                    child
                        .borrow()
                        .as_widget()
                        .layout(tree.borrow_mut(), renderer, &child_limits);
                let size = layout.size();

                available -= axis.main(size);
                cross = cross.max(axis.cross(size));

                nodes[i] = layout;
            }
        } else {
            fill_main_sum += fill_main_factor;
        }
    }

    for (i, (child, tree)) in items.iter().zip(trees.iter_mut()).enumerate() {
        let (fill_main_factor, fill_cross_factor) = {
            let size = child.borrow().as_widget().size();

            axis.pack(size.width.fill_factor(), size.height.fill_factor())
        };

        if fill_main_factor == 0 && fill_cross_factor != 0 {
            let (max_width, max_height) = axis.pack(available, cross);

            let child_limits = Limits::new(Size::ZERO, Size::new(max_width, max_height));

            let layout =
                child
                    .borrow()
                    .as_widget()
                    .layout(tree.borrow_mut(), renderer, &child_limits);
            let size = layout.size();

            available -= axis.main(size);
            cross = cross.max(axis.cross(layout.size()));

            nodes[i] = layout;
        }
    }

    let remaining = match axis {
        Axis::Horizontal => match width {
            Length::Shrink => 0.0,
            _ => available.max(0.0),
        },
        Axis::Vertical => match height {
            Length::Shrink => 0.0,
            _ => available.max(0.0),
        },
    };

    for (i, (child, tree)) in items.iter().zip(trees).enumerate() {
        let (fill_main_factor, fill_cross_factor) = {
            let size = child.borrow().as_widget().size();

            axis.pack(size.width.fill_factor(), size.height.fill_factor())
        };

        if fill_main_factor != 0 {
            let max_main = remaining * fill_main_factor as f32 / fill_main_sum as f32;

            let min_main = if max_main.is_infinite() {
                0.0
            } else {
                max_main
            };

            let max_cross = if fill_cross_factor == 0 {
                max_cross
            } else {
                cross
            };

            let (min_width, min_height) = axis.pack(min_main, 0.0);
            let (max_width, max_height) = axis.pack(max_main, max_cross);

            let child_limits = Limits::new(
                Size::new(min_width, min_height),
                Size::new(max_width, max_height),
            );

            let layout =
                child
                    .borrow()
                    .as_widget()
                    .layout(tree.borrow_mut(), renderer, &child_limits);
            cross = cross.max(axis.cross(layout.size()));

            nodes[i] = layout;
        }
    }

    let pad = axis.pack(padding.left, padding.top);
    let mut main = pad.0;

    for (i, node) in nodes.iter_mut().enumerate() {
        if i > 0 {
            main += spacing.0;
        }

        let (x, y) = axis.pack(main, pad.1);

        node.move_to_mut(Point::new(x, y));

        match axis {
            Axis::Horizontal => {
                node.align_mut(Alignment::Start, align_items, Size::new(0.0, cross));
            }
            Axis::Vertical => {
                node.align_mut(align_items, Alignment::Start, Size::new(cross, 0.0));
            }
        }

        let size = node.size();

        main += axis.main(size);
    }

    let (intrinsic_width, intrinsic_height) = axis.pack(main - pad.0, cross);
    let size = limits.resolve(width, height, Size::new(intrinsic_width, intrinsic_height));

    Node::with_children(size.expand(padding), nodes)
}
