//! A resolved theme: the generated palette in the terminal's colour depth,
//! plus the hand-written anatomy.

use ratatui::style::{Color, Style};

use crate::ThemeId;
use crate::anatomy::{Anatomy, Tone};
use crate::color::{ColorDepth, Role, contrast};
use crate::effects::mix;
use kp_tui_palette::{Palette, Rgb};

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

impl Theme {
    /// A semantic colour that can be READ on `surface`.
    ///
    /// The package keeps two tokens per state — `--success` and
    /// `--success-foreground` — and which of the two is the ink differs by
    /// register: in eighteen themes `--success-foreground` is the ink for a
    /// page, in the other four it is the ink for the plate. Neither is
    /// guaranteed, so this measures both against the surface, takes the
    /// better, and lifts it towards the theme's own `--foreground` until it
    /// reads at 4.5:1.
    ///
    /// Measured 2026-09-18, and this is why it exists: `--success` on
    /// `--card` reads 1.19:1 in synthwave, 1.16:1 in titanium, 1.21:1 in
    /// light — 43 of the 66 state/card pairs in the set are under the bar.
    /// A plate colour is not an ink.
    pub fn ink(&self, tone: Tone, surface: Rgb) -> Color {
        self.depth.resolve(Role::Ink, self.ink_rgb(tone, surface))
    }

    /// The same choice, before the depth resolves it — for a caller that
    /// wants to mix it further, and for the test that measures it.
    pub fn ink_rgb(&self, tone: Tone, surface: Rgb) -> Rgb {
        let p = self.id.palette();
        let (plate, pair) = match tone {
            Tone::Success => (p.success, p.success_foreground),
            Tone::Warning => (p.warning, p.warning_foreground),
            Tone::Danger => (p.destructive, p.destructive_foreground),
            Tone::Info => (p.info, p.info_foreground),
            // Everything else is already an ink or a ground of its own.
            other => {
                return match self.tone_rgb(other) {
                    Some(c) => c,
                    None => p.foreground,
                };
            }
        };
        let best = if contrast(pair, surface) >= contrast(plate, surface) {
            pair
        } else {
            plate
        };
        if contrast(best, surface) >= MIN_CONTRAST {
            return best;
        }
        // Towards the theme's own ink, which the register already keeps
        // readable on this surface, one twentieth at a time — the hue
        // survives as long as it can.
        for step in 1..=20 {
            let lifted = mix(best, p.foreground, step as f32 / 20.0);
            if contrast(lifted, surface) >= MIN_CONTRAST {
                return lifted;
            }
        }
        p.foreground
    }

    /// A point on the theme's own chart ramp, `0.0` to `1.0`: the five
    /// chart colours laid end to end and read between.
    ///
    /// homelab's splash walks a cyan-to-magenta gradient it writes in raw
    /// RGB — the one place in its whole client where a colour is computed
    /// rather than named. This is the same idea asked of the theme, so
    /// twenty-two registers each have their own ramp and none of them is
    /// a literal [gap-16].
    pub fn ramp(&self, t: f32) -> Color {
        self.depth.resolve(Role::Ink, self.ramp_rgb(t))
    }

    pub fn ramp_rgb(&self, t: f32) -> Rgb {
        let p = self.id.palette();
        let stops = [p.chart_1, p.chart_2, p.chart_3, p.chart_4, p.chart_5];
        let t = t.clamp(0.0, 1.0) * (stops.len() - 1) as f32;
        let i = (t.floor() as usize).min(stops.len() - 2);
        let f = t - i as f32;
        let mix = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * f).round() as u8;
        let (a, b) = (stops[i], stops[i + 1]);
        Rgb(mix(a.0, b.0), mix(a.1, b.1), mix(a.2, b.2))
    }

    /// A tone as the palette writes it, with no reading taken.
    pub fn tone_rgb(&self, tone: Tone) -> Option<Rgb> {
        let p = self.id.palette();
        Some(match tone {
            Tone::None => return None,
            Tone::Background => p.background,
            Tone::Card => p.card,
            Tone::Muted => p.muted,
            Tone::Line => p.border_strong,
            Tone::Ink => p.foreground,
            Tone::MutedInk => p.muted_foreground,
            Tone::Primary => p.primary,
            Tone::PrimaryInk => p.primary_foreground,
            Tone::Secondary => p.secondary,
            Tone::SecondaryInk => p.secondary_foreground,
            Tone::Accent => p.accent,
            Tone::Success => p.success,
            Tone::Warning => p.warning,
            Tone::Danger => p.destructive,
            Tone::Info => p.info,
        })
    }
}

impl Theme {
    /// The ink that can be read ON a plate: whichever of the theme's own
    /// grounds and inks reads furthest from it. A state plate is a
    /// different colour in every register, so the pairing cannot be fixed
    /// in advance.
    pub fn on_plate(&self, plate: Rgb) -> Color {
        self.depth.resolve(Role::OnFill, self.on_plate_rgb(plate))
    }

    /// The same choice, unresolved.
    pub fn on_plate_rgb(&self, plate: Rgb) -> Rgb {
        let p = self.id.palette();
        // Black and white are in the running last: a terminal can paint
        // both, and a plate nobody can read on is worse than one that
        // steps outside the palette for its ink. shade-dark's warning
        // plate is the case — its own colours top out at 4.17:1.
        [
            p.background,
            p.foreground,
            p.card,
            p.card_foreground,
            p.primary_foreground,
            Rgb(0, 0, 0),
            Rgb(255, 255, 255),
        ]
        .into_iter()
        .max_by(|a, b| {
            contrast(*a, plate)
                .partial_cmp(&contrast(*b, plate))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap_or(p.background)
    }
}

/// The floor every ink here is held to: WCAG AA for body text, which is the
/// bar `gates/check-site.mjs` holds the package's own site to.
pub const MIN_CONTRAST: f32 = 4.5;
