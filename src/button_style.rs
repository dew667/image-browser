use iced::{
    Background, Border, Color, Shadow, Theme, Vector, border, color,
    widget::button::{Status, Style},
};

pub fn default(theme: &Theme, status: Status) -> Style {
    let (background, text_color) = match status {
        Status::Active => (
            Some(Background::Color(Color::from_rgb8(248, 249, 250))),
            Color::from_rgb8(33, 37, 41),
        ),
        Status::Hovered => (
            Some(Background::Color(Color::from_rgb8(233, 236, 239))),
            Color::from_rgb8(33, 37, 41),
        ),
        Status::Pressed => (
            Some(Background::Color(Color::from_rgb8(222, 226, 230))),
            Color::from_rgb8(33, 37, 41),
        ),
        Status::Disabled => (
            Some(Background::Color(Color::from_rgb8(248, 249, 250))),
            Color::from_rgb8(108, 117, 125),
        ),
    };

    Style {
        background,
        text_color,
        border: Border {
            radius: 6.0.into(),
            width: 1.0,
            color: Color::from_rgb8(222, 226, 230),
        },
        shadow: Shadow {
            offset: Vector::new(0.0, 1.0),
            blur_radius: 2.0.into(),
            color: Color::from_rgba8(0, 0, 0, 0.1),
        },
    }
}

pub fn primary(_theme: &Theme, status: Status) -> Style {
    let (background, text_color) = match status {
        Status::Active => (
            Some(Background::Color(Color::from_rgb8(13, 110, 253))),
            Color::WHITE,
        ),
        Status::Hovered => (
            Some(Background::Color(Color::from_rgb8(0, 86, 179))),
            Color::WHITE,
        ),
        Status::Pressed => (
            Some(Background::Color(Color::from_rgb8(10, 88, 202))),
            Color::WHITE,
        ),
        Status::Disabled => (
            Some(Background::Color(Color::from_rgb8(108, 117, 125))),
            Color::from_rgb8(248, 249, 250),
        ),
    };

    Style {
        background,
        text_color,
        border: Border {
            radius: 8.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        shadow: Shadow {
            offset: Vector::new(0.0, 2.0),
            blur_radius: 4.0.into(),
            color: Color::from_rgba8(13, 110, 253, 0.25),
        },
    }
}

pub fn transparent(_theme: &Theme, status: Status) -> Style {
    let (background, text_color) = match status {
        Status::Active => (
            Some(Background::Color(Color::TRANSPARENT)),
            Color::from_rgb8(33, 37, 41),
        ),
        Status::Hovered => (
            Some(Background::Color(Color::from_rgba8(0, 0, 0, 0.05))),
            Color::from_rgb8(33, 37, 41),
        ),
        Status::Pressed => (
            Some(Background::Color(Color::from_rgba8(0, 0, 0, 0.1))),
            Color::from_rgb8(33, 37, 41),
        ),
        Status::Disabled => (
            Some(Background::Color(Color::TRANSPARENT)),
            Color::from_rgb8(108, 117, 125),
        ),
    };

    Style {
        background,
        text_color,
        border: Border {
            radius: 4.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        shadow: Shadow::default(),
    }
}

pub fn highlighted(_theme: &Theme, status: Status) -> Style {
    let (background, border_color) = match status {
        Status::Active => (
            Some(Background::Color(Color::from_rgba8(13, 110, 253, 0.1))),
            Color::from_rgb8(13, 110, 253),
        ),
        Status::Hovered => (
            Some(Background::Color(Color::from_rgba8(13, 110, 253, 0.15))),
            Color::from_rgb8(13, 110, 253),
        ),
        Status::Pressed => (
            Some(Background::Color(Color::from_rgba8(13, 110, 253, 0.2))),
            Color::from_rgb8(10, 88, 202),
        ),
        Status::Disabled => (
            Some(Background::Color(Color::from_rgba8(108, 117, 125, 0.1))),
            Color::from_rgb8(108, 117, 125),
        ),
    };

    Style {
        background,
        text_color: Color::from_rgb8(33, 37, 41),
        border: Border {
            radius: 6.0.into(),
            width: 2.0,
            color: border_color,
        },
        shadow: Shadow {
            offset: Vector::new(0.0, 1.0),
            blur_radius: 3.0.into(),
            color: Color::from_rgba8(13, 110, 253, 0.2),
        },
    }
}

pub fn sidebar_item(_theme: &Theme, status: Status) -> Style {
    let (background, text_color) = match status {
        Status::Active => (
            Some(Background::Color(Color::TRANSPARENT)),
            Color::from_rgb8(52, 58, 64),
        ),
        Status::Hovered => (
            Some(Background::Color(Color::from_rgba8(13, 110, 253, 0.08))),
            Color::from_rgb8(13, 110, 253),
        ),
        Status::Pressed => (
            Some(Background::Color(Color::from_rgba8(13, 110, 253, 0.12))),
            Color::from_rgb8(10, 88, 202),
        ),
        Status::Disabled => (
            Some(Background::Color(Color::TRANSPARENT)),
            Color::from_rgb8(173, 181, 189),
        ),
    };

    Style {
        background,
        text_color,
        border: Border {
            radius: 4.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        shadow: Shadow::default(),
    }
}

pub fn thumbnail(_theme: &Theme, status: Status) -> Style {
    let (background, border_color, shadow) = match status {
        Status::Active => (
            Some(Background::Color(Color::WHITE)),
            Color::from_rgb8(222, 226, 230),
            Shadow {
                offset: Vector::new(0.0, 1.0),
                blur_radius: 3.0.into(),
                color: Color::from_rgba8(0, 0, 0, 0.1),
            },
        ),
        Status::Hovered => (
            Some(Background::Color(Color::WHITE)),
            Color::from_rgb8(13, 110, 253),
            Shadow {
                offset: Vector::new(0.0, 2.0),
                blur_radius: 6.0.into(),
                color: Color::from_rgba8(13, 110, 253, 0.25),
            },
        ),
        Status::Pressed => (
            Some(Background::Color(Color::from_rgb8(248, 249, 250))),
            Color::from_rgb8(10, 88, 202),
            Shadow {
                offset: Vector::new(0.0, 1.0),
                blur_radius: 2.0.into(),
                color: Color::from_rgba8(0, 0, 0, 0.15),
            },
        ),
        Status::Disabled => (
            Some(Background::Color(Color::from_rgb8(248, 249, 250))),
            Color::from_rgb8(222, 226, 230),
            Shadow::default(),
        ),
    };

    Style {
        background,
        text_color: Color::from_rgb8(33, 37, 41),
        border: Border {
            radius: 8.0.into(),
            width: 2.0,
            color: border_color,
        },
        shadow,
    }
}

pub fn thumbnail_selected(_theme: &Theme, status: Status) -> Style {
    let (background, shadow) = match status {
        Status::Active | Status::Hovered | Status::Pressed => (
            Some(Background::Color(Color::WHITE)),
            Shadow {
                offset: Vector::new(0.0, 4.0),
                blur_radius: 12.0.into(),
                color: Color::from_rgba8(13, 110, 253, 0.4),
            },
        ),
        Status::Disabled => (
            Some(Background::Color(Color::from_rgb8(248, 249, 250))),
            Shadow::default(),
        ),
    };

    Style {
        background,
        text_color: Color::from_rgb8(33, 37, 41),
        border: Border {
            radius: 8.0.into(),
            width: 3.0,
            color: Color::from_rgb8(13, 110, 253),
        },
        shadow,
    }
}
