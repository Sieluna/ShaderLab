use iced::{Background, Border, Color, Padding, Shadow, Theme, Vector};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Active,
    Focused,
}

#[derive(Debug, Clone, Copy)]
pub struct Style {
    pub bar_background: Background,
    pub bar_border: Border,
    pub bar_shadow: Shadow,
    pub bar_background_expand: Padding,

    pub menu_background: Background,
    pub menu_border: Border,
    pub menu_shadow: Shadow,
    pub menu_background_expand: Padding,

    pub path: Background,
    pub path_border: Border,
}

pub trait Catalog {
    type Class<'a>;

    fn default<'a>() -> Self::Class<'a>;

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style;
}

pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme, Status) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        class(self, status)
    }
}

pub fn default(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();

    let active = Style {
        bar_background: palette.background.base.color.into(),
        bar_border: Border {
            radius: 8.0.into(),
            ..Default::default()
        },
        bar_shadow: Shadow::default(),
        bar_background_expand: 5.into(),

        menu_background: palette.background.base.color.into(),
        menu_border: Border {
            radius: 8.0.into(),
            ..Default::default()
        },
        menu_shadow: Shadow {
            color: Color::from([0.0, 0.0, 0.0, 0.5]),
            offset: Vector::ZERO,
            blur_radius: 10.0,
        },
        menu_background_expand: 5.into(),
        path: palette.primary.weak.color.into(),
        path_border: Border {
            radius: 6.0.into(),
            ..Default::default()
        },
    };

    match status {
        Status::Active => active,
        Status::Focused => Style {
            path: palette.primary.strong.color.into(),
            ..active
        },
    }
}
