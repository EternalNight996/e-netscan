#![allow(dead_code)]
use iced::{button, container, text_input, Background, Color, Vector};

const TEXT_PLACEHOLDER_COLOR: Color = Color::from_rgb(
    0x90 as f32 / 255.0,
    0x90 as f32 / 255.0,
    0x90 as f32 / 255.0,
);
const TEXT_VALUE_COLOR: Color = Color::from_rgb(
    0xe0 as f32 / 255.0,
    0x00 as f32 / 255.0,
    0x00 as f32 / 255.0,
);
const TEXT_SELECT_COLOR: Color = Color::from_rgb(
    0xc0 as f32 / 255.0,
    0xc0 as f32 / 255.0,
    0xc0 as f32 / 255.0,
);
const PANE_ID_COLOR_UNFOCUSED: Color = Color::from_rgb(
    0xFF as f32 / 255.0,
    0xC7 as f32 / 255.0,
    0xC7 as f32 / 255.0,
);
const PANE_ID_COLOR_FOCUSED: Color = Color::from_rgb(
    0xFF as f32 / 255.0,
    0x47 as f32 / 255.0,
    0x47 as f32 / 255.0,
);

const SURFACE: Color = Color::from_rgb(
    0xF2 as f32 / 255.0,
    0xF3 as f32 / 255.0,
    0xF5 as f32 / 255.0,
);

const ACTIVE: Color = Color::from_rgb(
    0x72 as f32 / 255.0,
    0x89 as f32 / 255.0,
    0xDA as f32 / 255.0,
);

const HOVERED: Color = Color::from_rgb(
    0x67 as f32 / 255.0,
    0x7B as f32 / 255.0,
    0xC4 as f32 / 255.0,
);

const FINISHED: Color = Color::from_rgb(
    0x20 as f32 / 255.0,
    0xbf as f32 / 255.0,
    0x20 as f32 / 255.0,
);

pub(crate) enum TitleBar {
    Active,
    Focused,
}

impl container::StyleSheet for TitleBar {
    fn style(&self) -> container::Style {
        let pane = match self {
            Self::Active => Pane::Active,
            Self::Focused => Pane::Focused,
        }
        .style();

        container::Style {
            text_color: Some(Color::WHITE),
            background: Some(pane.border_color.into()),
            ..Default::default()
        }
    }
}

pub(crate) enum Pane {
    Active,
    Focused,
}

impl container::StyleSheet for Pane {
    fn style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color(SURFACE)),
            border_width: 2.0,
            border_color: match self {
                Self::Active => Color::from_rgb(0.7, 0.7, 0.7),
                Self::Focused => Color::BLACK,
            },
            ..Default::default()
        }
    }
}

pub(crate) enum TextInput {
    Primary,
}
impl text_input::StyleSheet for TextInput {
    fn active(&self) -> text_input::Style {
        let (bg, color) = match self {
            TextInput::Primary => (Color::TRANSPARENT, Color::BLACK),
        };
        text_input::Style {
            background: Background::Color(bg),
            border_color: color,
            border_radius: 5.0,
            border_width: 1.0
        }
    }
    fn focused(&self) -> text_input::Style {
        let (bg, color) = match self {
            TextInput::Primary => (Color::TRANSPARENT, Color::BLACK),
        };
        text_input::Style {
            background: Background::Color(bg),
            border_color: color,
            border_radius: 5.0,
            border_width: 1.0
        }
    }

    fn placeholder_color(&self) -> Color {
        TEXT_PLACEHOLDER_COLOR
    }

    fn value_color(&self) -> Color {
        TEXT_VALUE_COLOR
    }

    fn selection_color(&self) -> Color {
        TEXT_SELECT_COLOR
    }
}
pub(crate) enum Button {
    Primary,
    Destructive,
    Control,
    Pin,
    Finished,
}

impl button::StyleSheet for Button {
    fn active(&self) -> button::Style {
        let (background, text_color) = match self {
            Button::Primary => (Some(ACTIVE), Color::WHITE),
            Button::Destructive => (None, Color::from_rgb8(0xFF, 0x47, 0x47)),
            Button::Control => (Some(PANE_ID_COLOR_FOCUSED), Color::WHITE),
            Button::Pin => (Some(ACTIVE), Color::WHITE),
            Button::Finished => (Some(FINISHED), Color::WHITE),
        };

        button::Style {
            text_color,
            background: background.map(Background::Color),
            border_radius: 5.0,
            shadow_offset: Vector::new(0.0, 0.0),
            ..button::Style::default()
        }
    }

    fn hovered(&self) -> button::Style {
        let active = self.active();

        let background = match self {
            Button::Primary => Some(HOVERED),
            Button::Destructive => Some(Color {
                a: 0.2,
                ..active.text_color
            }),
            Button::Control => Some(PANE_ID_COLOR_FOCUSED),
            Button::Pin => Some(HOVERED),
            Button::Finished => Some(FINISHED),
        };

        button::Style {
            background: background.map(Background::Color),
            ..active
        }
    }
}
