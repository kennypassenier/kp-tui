//! A resolved theme: the generated palette in the terminal's colour depth,
//! plus the hand-written anatomy.

use ratatui::style::{Color, Style};

use crate::ThemeId;
use crate::anatomy::Anatomy;
use crate::color::ColorDepth;
use kp_tui_palette::Palette;

/// What every widget takes. Cheap to build (a few dozen matches), so a
/// theme switch simply builds a new one before the next frame.
#[derive(Clone, Copy, Debug)]
pub struct Theme {
    pub id: ThemeId,
    pub depth: ColorDepth,
    pub c: Palette<Color>,
    pub a: &'static Anatomy,
}

impl Theme {
    pub fn new(id: ThemeId, depth: ColorDepth) -> Self {
        Theme {
            id,
            depth,
            c: id.palette().map(|role, rgb| depth.resolve(role, rgb)),
            a: id.anatomy(),
        }
    }

    pub fn base(&self) -> Style {
        Style::new().bg(self.c.background).fg(self.c.foreground)
    }

    pub fn label(&self, text: &str) -> String {
        let body = if self.a.uppercase_labels {
            text.to_uppercase()
        } else {
            text.to_string()
        };
        format!("{}{}", self.a.label_prefix, body)
    }
}
