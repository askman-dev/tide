use floem::peniko::Color;

use alacritty_terminal::vte::ansi::Rgb as AnsiRgb;

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

fn theme_color_to_ansi_rgb(color: Color) -> AnsiRgb {
    let rgba = color.to_rgba8();
    AnsiRgb {
        r: rgba.r,
        g: rgba.g,
        b: rgba.b,
    }
}

fn ansi_rgb(r: u8, g: u8, b: u8) -> AnsiRgb {
    AnsiRgb { r, g, b }
}

#[derive(Clone, Copy)]
pub struct TerminalPalette {
    pub primary_background: AnsiRgb,
    pub primary_foreground: AnsiRgb,
    pub normal: [AnsiRgb; 8],
    pub bright: [AnsiRgb; 8],
    pub dim: [AnsiRgb; 8],
}

impl TerminalPalette {
    /// Build a terminal color palette derived from the current UI theme.
    ///
    /// The primary foreground/background are taken directly from the theme,
    /// while the normal/bright/dim ANSI colors start from Alacritty's
    /// defaults for its dark theme.
    pub fn for_theme(theme: UiTheme) -> Self {
        let primary_background = theme_color_to_ansi_rgb(theme.panel_bg);
        let primary_foreground = theme_color_to_ansi_rgb(theme.text);

        let normal = [
            ansi_rgb(0x18, 0x18, 0x18),
            ansi_rgb(0xac, 0x42, 0x42),
            ansi_rgb(0x90, 0xa9, 0x59),
            ansi_rgb(0xf4, 0xbf, 0x75),
            ansi_rgb(0x6a, 0x9f, 0xb5),
            ansi_rgb(0xaa, 0x75, 0x9f),
            ansi_rgb(0x75, 0xb5, 0xaa),
            ansi_rgb(0xd8, 0xd8, 0xd8),
        ];

        let bright = [
            ansi_rgb(0x6b, 0x6b, 0x6b),
            ansi_rgb(0xc5, 0x55, 0x55),
            ansi_rgb(0xaa, 0xc4, 0x74),
            ansi_rgb(0xfe, 0xca, 0x88),
            ansi_rgb(0x82, 0xb8, 0xc8),
            ansi_rgb(0xc2, 0x8c, 0xb8),
            ansi_rgb(0x93, 0xd3, 0xc3),
            ansi_rgb(0xf8, 0xf8, 0xf8),
        ];

        let dim = [
            ansi_rgb(0x0f, 0x0f, 0x0f),
            ansi_rgb(0x71, 0x2b, 0x2b),
            ansi_rgb(0x5f, 0x6f, 0x3a),
            ansi_rgb(0xa1, 0x7e, 0x4d),
            ansi_rgb(0x45, 0x68, 0x77),
            ansi_rgb(0x70, 0x4d, 0x68),
            ansi_rgb(0x4d, 0x77, 0x70),
            ansi_rgb(0x8e, 0x8e, 0x8e),
        ];

        TerminalPalette {
            primary_background,
            primary_foreground,
            normal,
            bright,
            dim,
        }
    }
}
