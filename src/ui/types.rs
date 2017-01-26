
extern crate rustbox;
use self::rustbox::Color;

#[derive(Debug)]
pub enum UIError {
    UiInitialisationFailed,
}

pub struct Cursor {
    pub x: usize,
    pub y: usize,
}

impl Cursor {
    pub fn new(x: usize, y: usize) -> Cursor {
        Cursor { x: x, y: y }
    }
}

pub struct UserCursor {
    pub x: usize,
    pub y: usize,
}

impl UserCursor {
    pub fn new(x: usize, y: usize) -> UserCursor {
        UserCursor { x: x, y: y }
    }

    pub fn bg(&self, x: usize, y: usize) -> Color {
        self.bg_or(Color::Default, x, y)
    }

    pub fn bg_or(&self, default: Color, _: usize, y: usize) -> Color {
        let UserCursor { x: _, y: uy } = *self;
        if y == uy {
            return Color::White;
        } else {
            return default;
        }
    }

    pub fn fg_or(&self, default: Color, _: usize, y: usize) -> Color {
        let UserCursor { x: _, y: uy } = *self;
        if y == uy {
            return Color::Black;
        } else {
            return default;
        }
    }

    pub fn fg(&self, x: usize, y: usize) -> Color {
        self.fg_or(Color::White, x, y)
    }
}

pub struct TuiContext<'a, A: 'a> {
    pub ui: &'a A,
    pub user_cursor: &'a mut UserCursor,
    pub draw_cursor: &'a mut Cursor,
}
