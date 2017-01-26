extern crate rustbox;

use self::rustbox::{Style, RustBox, Color};
use ui::types::TuiContext;

pub fn normal_line(ctx: &TuiContext<RustBox>, x: usize, y: usize, str: &str) {
    ctx.ui.print(x,
                 y,
                 rustbox::RB_NORMAL,
                 ctx.user_cursor.fg(x, y),
                 ctx.user_cursor.bg(x, y),
                 str);
}

pub fn heat_line(ctx: &TuiContext<RustBox>, x: usize, y: usize, temp: &Temperature, str: &str) {
    ctx.ui.print(x,
                 y,
                 temp.to_style(),
                 ctx.user_cursor.fg_or(temp.to_colour(), x, y),
                 ctx.user_cursor.bg(x, y),
                 str);
}

pub fn styled_line(ctx: &TuiContext<RustBox>, x: usize, y: usize, temp: &Temperature, str: &str) {
    ctx.ui.print(x,
                 y,
                 temp.to_style(),
                 ctx.user_cursor.fg(x, y),
                 ctx.user_cursor.bg(x, y),
                 str);
}

pub enum Temperature {
    Hot,
    Warm,
    Cold,
}

impl Temperature {
    pub fn from(temp_c: f32) -> Temperature {
        match temp_c {
            0.0...10.0 => Temperature::Cold,
            10.1...50.0 => Temperature::Warm,
            _ => Temperature::Hot,
        }
    }

    // Monoidal append.
    pub fn append(t1: &Temperature, t2: &Temperature) -> Temperature {
        match (t1, t2) {
            (&Temperature::Hot, _) => Temperature::Hot,
            (_, &Temperature::Hot) => Temperature::Hot,
            (&Temperature::Warm, _) => Temperature::Warm,
            (_, &Temperature::Warm) => Temperature::Warm,
            _ => Temperature::Cold,
        }
    }

    fn to_style(&self) -> Style {
        match *self {
            Temperature::Cold => rustbox::RB_NORMAL,
            Temperature::Warm => rustbox::RB_NORMAL,
            Temperature::Hot => rustbox::RB_BOLD,
        }
    }

    fn to_colour(&self) -> Color {
        match *self {
            Temperature::Cold => Color::White,
            Temperature::Warm => Color::Yellow,
            Temperature::Hot => Color::Red,
        }
    }
}
