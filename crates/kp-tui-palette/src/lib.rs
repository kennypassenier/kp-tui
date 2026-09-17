//! The kp-themes palettes, exactly as the package generates them.
//!
//! `palette.rs` is vendored, not written here: kp-themes emits it from
//! `themes/*/tokens.json` and attaches it to every release as
//! `kp-tui-palette.rs`. Upgrading is a download and a copy, the way
//! chassis-rs vendors the stylesheets, so a binary built from this crate
//! needs no node_modules and no network.
//!
//! Nothing in this crate decides anything. What a terminal can carry of a
//! theme BEYOND colour — border glyphs, case, prefixes, the cursor, the
//! reveal — is judgement, and lives in `kp-tui`.

mod palette;

pub use palette::{KP_THEMES_VERSION, Palette, Rgb, Role, THEMES};

/// A theme by its name, as the package spells it (`shade-light`).
pub fn by_name(name: &str) -> Option<&'static palette::Theme> {
    THEMES.iter().find(|t| t.name == name)
}

/// Every theme's name, in the package's own order.
pub fn names() -> impl Iterator<Item = &'static str> {
    THEMES.iter().map(|t| t.name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_package_ships_twenty_two_themes_in_its_own_order() {
        assert_eq!(THEMES.len(), 22);
        assert_eq!(THEMES[0].name, "formal");
        assert_eq!(THEMES[21].name, "titanium");
        assert!(by_name("cyberpunk").is_some());
        assert!(by_name("no-such-theme").is_none());
    }

    #[test]
    fn every_theme_carries_a_full_palette_and_says_where_it_came_from() {
        assert!(!KP_THEMES_VERSION.is_empty());
        for theme in THEMES {
            // A palette with a black hole in it would read as a theme that
            // simply paints nothing there; the generator refuses a missing
            // token, and this is the copy's own check of that.
            let p = theme.palette;
            assert_ne!(
                p.background, p.foreground,
                "{}: ink on its own ground",
                theme.name
            );
            assert_ne!(
                p.primary, p.primary_foreground,
                "{}: a button nobody can read",
                theme.name
            );
            assert!(
                theme.fx_duration_ms > 0,
                "{}: no motion duration",
                theme.name
            );
        }
    }

    #[test]
    fn map_carries_the_shape_into_another_colour_type() {
        let p = THEMES[0].palette.map(|role, rgb| (role, rgb.0));
        assert_eq!(p.background.0, Role::Surface);
        assert_eq!(p.foreground.0, Role::Ink);
    }
}
