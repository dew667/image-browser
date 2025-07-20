use iced::{border, widget::button::{Status, Style}, Background, Border, Color, Shadow, Theme, Vector};

pub fn default(theme: &Theme, status: Status) -> Style {
    let (background, text_color) = match status {
        Status::Active => (
            Some(Background::Color(Color::from_rgb8(240, 242, 245))),
            Color::from_rgb8(60, 60, 60),
        ),
        Status::Hovered => (
            Some(Background::Color(Color::from_rgb8(220, 222, 225))),
            Color::from_rgb8(60, 60, 60),
        ),
        Status::Pressed => (
            Some(Background::Color(Color::from_rgb8(210, 212, 215))),
            Color::from_rgb8(60, 60, 60),
        ),
        Status::Disabled => (
            Some(Background::Color(theme.palette().primary)), 
            theme.palette().text
        ),
    };

    Style {
        background,
        text_color,
        border: Border::default(),
        shadow: Shadow::default(),
    }
}

pub fn primary(_theme: &Theme, status: Status) -> Style {
    let (background, text_color) = match status {
        Status::Active => (
            Some(Background::Color(Color::from_rgb8(0, 122, 255))),
            Color::WHITE,
        ),
        Status::Hovered => (
            Some(Background::Color(Color::from_rgb8(0, 102, 225))),
            Color::WHITE,
        ),
        Status::Pressed => (
            Some(Background::Color(Color::from_rgb8(0, 82, 205))),
            Color::WHITE,
        ),
        Status::Disabled => (
            Some(Background::Color(Color::from_rgb8(180, 180, 180))),
            Color::from_rgb8(240, 240, 240),
        ),
    };

    Style {
        background,
        shadow: Shadow {
            offset: Vector::new(0.0, 0.0),
            blur_radius: 4.0.into(),
            color: Color::TRANSPARENT,
        },
        border: Border {
            radius: 4.0.into(),
            width: 0.0,
            color: Color::TRANSPARENT,
        },
        text_color,
    }
}