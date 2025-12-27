use floem::peniko::Color;

pub const LIST_FONT_SIZE: f32 = 13.0;
pub const LIST_HEIGHT: f32 = 22.0;
pub const HEADER_FONT_SIZE: f32 = 11.0;
pub const HEADER_HEIGHT: f32 = 28.0;
pub const TREE_INDENT: f32 = 12.0;

#[derive(Clone, Copy)]
pub struct UiTheme {
    pub surface: Color,
    pub panel_bg: Color,
    pub element_bg: Color,
    pub border_subtle: Color,
    pub accent: Color,
    pub text: Color,
    pub text_muted: Color,
    pub text_soft: Color,
}

impl UiTheme {
    pub fn new() -> Self {
        Self {
            surface: rgb(18, 19, 24),
            panel_bg: rgb(23, 24, 30),
            element_bg: rgb(34, 36, 44),
            border_subtle: rgb(30, 32, 38),
            accent: rgb(94, 160, 255),
            text: rgb(230, 232, 240),
            text_muted: rgb(154, 160, 175),
            text_soft: rgb(118, 124, 138),
        }
    }
}

pub fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgb8(r, g, b)
}
