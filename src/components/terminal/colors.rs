//! Terminal color palette and color resolution.

#[cfg(target_os = "macos")]
use crate::theme::{TerminalPalette, UiTheme};

#[cfg(target_os = "macos")]
use alacritty_terminal::{
    term::{cell::Flags, color::Colors as TermColors},
    vte::ansi::{Color as AnsiColor, NamedColor},
};

#[cfg(target_os = "macos")]
use floem::peniko::{Brush, Color};

#[cfg(target_os = "macos")]
use std::ops::{Index, IndexMut};

#[cfg(target_os = "macos")]
pub fn background_brush(theme: UiTheme) -> Brush {
    Brush::from(theme.panel_bg)
}

#[cfg(target_os = "macos")]
pub fn cursor_brush(theme: UiTheme) -> Brush {
    Brush::from(theme.accent.with_alpha(0.7))
}

#[cfg(target_os = "macos")]
pub fn ansi_rgb_to_color(rgb: alacritty_terminal::vte::ansi::Rgb) -> Color {
    Color::from_rgb8(rgb.r, rgb.g, rgb.b)
}

#[cfg(target_os = "macos")]
pub struct TerminalColorList([alacritty_terminal::vte::ansi::Rgb; alacritty_terminal::term::color::COUNT]);

#[cfg(target_os = "macos")]
impl TerminalColorList {
    pub fn from_palette(palette: &TerminalPalette) -> Self {
        use alacritty_terminal::term::color::COUNT;

        let mut list = TerminalColorList([alacritty_terminal::vte::ansi::Rgb::default(); COUNT]);

        list.fill_named(palette);
        list.fill_cube();
        list.fill_gray_ramp();

        list
    }

    fn fill_named(&mut self, palette: &TerminalPalette) {
        // Normal ANSI colors.
        self[NamedColor::Black] = palette.normal[0];
        self[NamedColor::Red] = palette.normal[1];
        self[NamedColor::Green] = palette.normal[2];
        self[NamedColor::Yellow] = palette.normal[3];
        self[NamedColor::Blue] = palette.normal[4];
        self[NamedColor::Magenta] = palette.normal[5];
        self[NamedColor::Cyan] = palette.normal[6];
        self[NamedColor::White] = palette.normal[7];

        // Bright ANSI colors.
        self[NamedColor::BrightBlack] = palette.bright[0];
        self[NamedColor::BrightRed] = palette.bright[1];
        self[NamedColor::BrightGreen] = palette.bright[2];
        self[NamedColor::BrightYellow] = palette.bright[3];
        self[NamedColor::BrightBlue] = palette.bright[4];
        self[NamedColor::BrightMagenta] = palette.bright[5];
        self[NamedColor::BrightCyan] = palette.bright[6];
        self[NamedColor::BrightWhite] = palette.bright[7];

        // Foreground and background.
        self[NamedColor::Foreground] = palette.primary_foreground;
        self[NamedColor::Background] = palette.primary_background;
        self[NamedColor::BrightForeground] = palette.bright[7];

        // Dimmed foreground and ANSI colors.
        let dim_fg = {
            let fg = palette.primary_foreground;
            let r = (fg.r as f32 * 0.66) as u8;
            let g = (fg.g as f32 * 0.66) as u8;
            let b = (fg.b as f32 * 0.66) as u8;
            alacritty_terminal::vte::ansi::Rgb { r, g, b }
        };

        self[NamedColor::DimForeground] = dim_fg;

        self[NamedColor::DimBlack] = palette.dim[0];
        self[NamedColor::DimRed] = palette.dim[1];
        self[NamedColor::DimGreen] = palette.dim[2];
        self[NamedColor::DimYellow] = palette.dim[3];
        self[NamedColor::DimBlue] = palette.dim[4];
        self[NamedColor::DimMagenta] = palette.dim[5];
        self[NamedColor::DimCyan] = palette.dim[6];
        self[NamedColor::DimWhite] = palette.dim[7];
    }

    fn fill_cube(&mut self) {
        let mut index: usize = 16;

        for r in 0..6 {
            for g in 0..6 {
                for b in 0..6 {
                    let red = if r == 0 { 0 } else { r * 40 + 55 };
                    let green = if g == 0 { 0 } else { g * 40 + 55 };
                    let blue = if b == 0 { 0 } else { b * 40 + 55 };

                    self.0[index] = alacritty_terminal::vte::ansi::Rgb {
                        r: red,
                        g: green,
                        b: blue,
                    };

                    index += 1;
                }
            }
        }

        debug_assert!(index == 232);
    }

    fn fill_gray_ramp(&mut self) {
        let mut index: usize = 232;

        for i in 0..24 {
            let value = i * 10 + 8;
            self.0[index] = alacritty_terminal::vte::ansi::Rgb {
                r: value,
                g: value,
                b: value,
            };

            index += 1;
        }

        debug_assert!(index == 256);
    }

    pub fn color_for_index(&self, index: usize, overrides: &TermColors) -> alacritty_terminal::vte::ansi::Rgb {
        let clamped = index.min(self.0.len().saturating_sub(1));
        overrides[clamped].unwrap_or(self.0[clamped])
    }
}

#[cfg(target_os = "macos")]
impl Index<usize> for TerminalColorList {
    type Output = alacritty_terminal::vte::ansi::Rgb;

    fn index(&self, idx: usize) -> &Self::Output {
        &self.0[idx]
    }
}

#[cfg(target_os = "macos")]
impl IndexMut<usize> for TerminalColorList {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        &mut self.0[idx]
    }
}

#[cfg(target_os = "macos")]
impl Index<NamedColor> for TerminalColorList {
    type Output = alacritty_terminal::vte::ansi::Rgb;

    fn index(&self, idx: NamedColor) -> &Self::Output {
        &self.0[idx as usize]
    }
}

#[cfg(target_os = "macos")]
impl IndexMut<NamedColor> for TerminalColorList {
    fn index_mut(&mut self, idx: NamedColor) -> &mut Self::Output {
        &mut self.0[idx as usize]
    }
}

#[cfg(target_os = "macos")]
pub fn resolve_fg_color(
    overrides: &TermColors,
    palette: &TerminalColorList,
    color: &AnsiColor,
    flags: Flags,
) -> Color {
    const DRAW_BOLD_TEXT_WITH_BRIGHT_COLORS: bool = true;
    const DIM_FACTOR: f32 = 0.66;

    let rgb = match *color {
        AnsiColor::Spec(spec) => {
            if (flags & Flags::DIM) == Flags::DIM {
                let r = (spec.r as f32 * DIM_FACTOR) as u8;
                let g = (spec.g as f32 * DIM_FACTOR) as u8;
                let b = (spec.b as f32 * DIM_FACTOR) as u8;
                alacritty_terminal::vte::ansi::Rgb { r, g, b }
            } else {
                spec
            }
        }
        AnsiColor::Named(ansi) => {
            let dim_bold = flags & Flags::DIM_BOLD;

            if dim_bold == Flags::DIM_BOLD && ansi == NamedColor::Foreground {
                palette.color_for_index(NamedColor::DimForeground as usize, overrides)
            } else if DRAW_BOLD_TEXT_WITH_BRIGHT_COLORS && dim_bold == Flags::BOLD {
                palette.color_for_index(ansi.to_bright() as usize, overrides)
            } else if dim_bold == Flags::DIM
                || (!DRAW_BOLD_TEXT_WITH_BRIGHT_COLORS && dim_bold == Flags::DIM_BOLD)
            {
                palette.color_for_index(ansi.to_dim() as usize, overrides)
            } else {
                palette.color_for_index(ansi as usize, overrides)
            }
        }
        AnsiColor::Indexed(idx) => {
            let dim_bold = flags & Flags::DIM_BOLD;
            let mut palette_index = idx as usize;

            match (DRAW_BOLD_TEXT_WITH_BRIGHT_COLORS, dim_bold, idx) {
                (true, Flags::BOLD, 0..=7) => {
                    palette_index = idx as usize + 8;
                }
                (false, Flags::DIM, 8..=15) => {
                    palette_index = idx as usize - 8;
                }
                (false, Flags::DIM, 0..=7) => {
                    palette_index = NamedColor::DimBlack as usize + idx as usize;
                }
                _ => {}
            }

            palette.color_for_index(palette_index, overrides)
        }
    };

    ansi_rgb_to_color(rgb)
}

#[cfg(target_os = "macos")]
pub fn resolve_bg_color(
    overrides: &TermColors,
    palette: &TerminalColorList,
    color: &AnsiColor,
) -> Color {
    let rgb = match *color {
        AnsiColor::Spec(spec) => spec,
        AnsiColor::Named(ansi) => palette.color_for_index(ansi as usize, overrides),
        AnsiColor::Indexed(idx) => palette.color_for_index(idx as usize, overrides),
    };

    ansi_rgb_to_color(rgb)
}
