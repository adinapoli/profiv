extern crate rustbox;

use self::rustbox::{RustBox, Color, Key};
use parser::GHCProf;

pub struct UI {
    ui: RustBox,
}

#[derive(Debug)]
pub enum UIError {
    UiInitialisationFailed,
}

impl UI {
    pub fn new() -> Result<UI, UIError> {
        match RustBox::init(Default::default()) {
            Result::Ok(v) => Ok(UI { ui: v }),
            Result::Err(_) => Err(UIError::UiInitialisationFailed),
        }
    }

    pub fn render_loop<'a>(&self, prof: GHCProf<'a>) {
        let ref rustbox = self.ui;
        rustbox.print(1,
                      1,
                      rustbox::RB_BOLD,
                      Color::White,
                      Color::Black,
                      "Hello, world!");
        rustbox.print(1,
                      3,
                      rustbox::RB_BOLD,
                      Color::White,
                      Color::Black,
                      "Press 'q' to quit.");
        loop {
            rustbox.present();
            match rustbox.poll_event(false) {
                Ok(rustbox::Event::KeyEvent(key)) => {
                    match key {
                        Key::Char('q') => {
                            break;
                        }
                        _ => {}
                    }
                }
                Err(e) => panic!("{}", e),
                _ => {}
            }
        }
    }
}
