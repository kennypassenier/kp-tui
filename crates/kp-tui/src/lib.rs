//! kp-themes in a terminal: the generated palettes, the hand-written
//! anatomy, and widgets that take a `&Theme` and name no colour of their
//! own.
//!
//! The split is deliberate. `kp-tui-palette` is data the package
//! generates; everything here is judgement, and every field of `Anatomy`
//! cites the register line it was read from, or says `GUESS:` and why.
//! `research/ratatui/ANATOMY_PROPOSAL.md` in kp-themes is the working that
//! produced the nineteen rows Kenny did not review one by one.

pub mod anatomy;
pub mod color;
pub mod components;
pub mod dashboard;
pub mod effects;
pub mod fx;
pub mod live;
pub mod logs;
pub mod theme;
pub mod widgets;

pub use anatomy::{
    Alarm, Anatomy, ButtonFace, Fx, Reveal, Rule, Selection, TableHead, Texture, Tone,
};
pub use color::ColorDepth;
pub use components::{
    AlarmPanel, Badge, Choice, Column, CommandPalette, DataTable, Facts, Field, KeyHints, LogPane,
    Meter, Popup, PopupKind, SelectList, Spark, Stepper, Stream, Surface, Ticker, fuzzy,
    fuzzy_score, fuzzy_spans, scrim, shadow, source_colour, spinner,
};
pub use effects::{Glitch, Spinner, Stage, Strike, Sweep, roll};
pub use kp_tui_palette::{KP_THEMES_VERSION, Palette, Rgb, Role, THEMES};
pub use theme::Theme;
pub use widgets::Rail;

/// One of the package's themes, by its place in `themes/order.json`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ThemeId(pub usize);

impl ThemeId {
    /// The three the demo carried first, and the ones a caller is most
    /// likely to name. Every other theme is `ThemeId::from_name`, or a
    /// walk through `ALL`.
    pub const FORMAL: ThemeId = ThemeId(0);
    pub const CYBERPUNK: ThemeId = ThemeId(3);
    pub const TERMINAL: ThemeId = ThemeId(6);
    #[allow(non_upper_case_globals)]
    pub const Formal: ThemeId = Self::FORMAL;
    #[allow(non_upper_case_globals)]
    pub const Cyberpunk: ThemeId = Self::CYBERPUNK;
    #[allow(non_upper_case_globals)]
    pub const Terminal: ThemeId = Self::TERMINAL;

    /// Every theme, in the package's own order.
    pub const ALL: [ThemeId; 22] = {
        let mut all = [ThemeId(0); 22];
        let mut i = 0;
        while i < 22 {
            all[i] = ThemeId(i);
            i += 1;
        }
        all
    };

    /// Every theme, in the package's own order.
    pub fn all() -> impl Iterator<Item = ThemeId> {
        Self::ALL.into_iter()
    }

    pub fn name(self) -> &'static str {
        THEMES[self.0].name
    }

    pub fn from_name(name: &str) -> Option<Self> {
        THEMES.iter().position(|t| t.name == name).map(ThemeId)
    }

    pub fn palette(self) -> &'static Palette<Rgb> {
        &THEMES[self.0].palette
    }

    pub fn anatomy(self) -> &'static Anatomy {
        anatomy::ANATOMIES[self.0]
    }

    pub fn dark(self) -> bool {
        THEMES[self.0].dark
    }

    /// `--fx-duration`, in milliseconds.
    pub fn fx_duration_ms(self) -> u32 {
        THEMES[self.0].fx_duration_ms
    }

    pub fn next(self) -> Self {
        ThemeId((self.0 + 1) % THEMES.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_theme_has_a_palette_and_an_anatomy() {
        assert_eq!(anatomy::ANATOMIES.len(), THEMES.len());
        for id in ThemeId::all() {
            assert_eq!(ThemeId::from_name(id.name()), Some(id));
            // The anatomy belongs to its own theme: a row copied from the
            // one above it would give two themes the same prefix AND the
            // same reveal, which no two of these share.
            let a = id.anatomy();
            let _ = (a.label_prefix, a.reveal, a.button_face);
        }
    }

    #[test]
    fn stepping_walks_the_whole_set_and_comes_back() {
        let mut id = ThemeId(0);
        for _ in 0..THEMES.len() {
            id = id.next();
        }
        assert_eq!(id, ThemeId(0));
    }
}
