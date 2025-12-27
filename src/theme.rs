use floem::kurbo::Point;
use floem::peniko::{Color, Gradient};

#[derive(Clone, Copy)]
pub struct UiColors {
    pub bg_top: Color,
    pub bg_bottom: Color,
    pub chrome_top: Color,
    pub chrome_bottom: Color,
    pub sidebar_top: Color,
    pub sidebar_bottom: Color,
    pub panel_top: Color,
    pub panel_bottom: Color,
    pub panel_header: Color,
    pub surface_top: Color,
    pub surface_bottom: Color,
    pub terminal_top: Color,
    pub terminal_bottom: Color,
    pub text: Color,
    pub text_muted: Color,
    pub text_soft: Color,
    pub accent: Color,
    pub border: Color,
    pub border_soft: Color,
}

impl UiColors {
    pub fn new() -> Self {
        Self {
            bg_top: rgb(24, 27, 36),
            bg_bottom: rgb(12, 13, 18),
            chrome_top: rgb(34, 38, 50),
            chrome_bottom: rgb(24, 26, 34),
            sidebar_top: rgb(27, 30, 40),
            sidebar_bottom: rgb(20, 22, 30),
            panel_top: rgb(28, 31, 40),
            panel_bottom: rgb(23, 25, 33),
            panel_header: rgb(32, 36, 46),
            surface_top: rgb(30, 33, 44),
            surface_bottom: rgb(22, 24, 32),
            terminal_top: rgb(13, 15, 20),
            terminal_bottom: rgb(9, 10, 14),
            text: rgb(232, 235, 242),
            text_muted: rgb(168, 174, 188),
            text_soft: rgb(122, 128, 142),
            accent: rgb(96, 170, 255),
            border: rgb(46, 51, 64),
            border_soft: rgb(36, 40, 52),
        }
    }
}

pub fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgb8(r, g, b)
}

pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
    Color::from_rgba8(r, g, b, a)
}

pub fn vertical_gradient(height: f64, start: Color, end: Color) -> Gradient {
    Gradient::new_linear(Point::new(0.0, 0.0), Point::new(0.0, height))
        .with_stops([(0.0, start), (1.0, end)])
}
