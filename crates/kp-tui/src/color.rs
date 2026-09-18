//! Colour depth: what the terminal can show, and how a token's RGB falls
//! back to it.

use ratatui::style::Color;

// `Rgb` and `Role` come from the generated palette: one definition of what
// a colour is and what it is for, shared by the crate that writes them and
// the crate that paints them.
pub use kp_tui_palette::{Rgb, Role};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorDepth {
    TrueColor,
    Ansi256,
    Ansi16,
}

impl ColorDepth {
    /// `--colors <truecolor|256|16>` or `KP_COLORS` wins; then `COLORTERM`
    /// (`truecolor`, `24bit`); then `TERM` containing `256color`; else 16.
    /// Takes the lookup as a function so tests do not touch the process
    /// environment.
    pub fn detect(flag: Option<&str>, env: impl Fn(&str) -> Option<String>) -> Self {
        if let Some(d) = flag
            .map(str::to_string)
            .or_else(|| env("KP_COLORS"))
            .and_then(|v| Self::parse(&v))
        {
            return d;
        }
        if let Some(ct) = env("COLORTERM")
            && matches!(ct.to_ascii_lowercase().as_str(), "truecolor" | "24bit")
        {
            return Self::TrueColor;
        }
        match env("TERM") {
            Some(t) if t.contains("256color") => Self::Ansi256,
            _ => Self::Ansi16,
        }
    }

    pub fn parse(v: &str) -> Option<Self> {
        match v {
            "truecolor" | "24bit" | "rgb" => Some(Self::TrueColor),
            "256" => Some(Self::Ansi256),
            "16" => Some(Self::Ansi16),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::TrueColor => "truecolor",
            Self::Ansi256 => "256",
            Self::Ansi16 => "16",
        }
    }

    pub fn resolve(self, role: Role, rgb: Rgb) -> Color {
        match self {
            Self::TrueColor => Color::Rgb(rgb.0, rgb.1, rgb.2),
            Self::Ansi256 => Color::Indexed(nearest_256(rgb)),
            Self::Ansi16 => ansi16(role, rgb),
        }
    }
}

/// Squared "redmean" distance: cheap and much closer to perception than
/// plain Euclidean RGB.
fn dist(a: Rgb, b: Rgb) -> i64 {
    let rm = (a.0 as i64 + b.0 as i64) / 2;
    let (dr, dg, db) = (
        a.0 as i64 - b.0 as i64,
        a.1 as i64 - b.1 as i64,
        a.2 as i64 - b.2 as i64,
    );
    (((512 + rm) * dr * dr) >> 8) + 4 * dg * dg + (((767 - rm) * db * db) >> 8)
}

/// The xterm 256 palette from index 16: a 6x6x6 cube and 24 greys. The
/// first 16 are left out because terminals retheme them.
pub fn nearest_256(c: Rgb) -> u8 {
    const LEVELS: [u8; 6] = [0, 95, 135, 175, 215, 255];
    let mut best = (i64::MAX, 16u8);
    for i in 16u16..=255 {
        let cand = if i < 232 {
            let n = i - 16;
            Rgb(
                LEVELS[(n / 36) as usize],
                LEVELS[(n / 6 % 6) as usize],
                LEVELS[(n % 6) as usize],
            )
        } else {
            let v = (8 + (i - 232) * 10) as u8;
            Rgb(v, v, v)
        };
        let d = dist(c, cand);
        if d < best.0 {
            best = (d, i as u8);
        }
    }
    best.1
}

fn hsl(c: Rgb) -> (f32, f32, f32) {
    let (r, g, b) = (c.0 as f32 / 255.0, c.1 as f32 / 255.0, c.2 as f32 / 255.0);
    let (max, min) = (r.max(g).max(b), r.min(g).min(b));
    let l = (max + min) / 2.0;
    let d = max - min;
    if d == 0.0 {
        return (0.0, 0.0, l);
    }
    let s = d / (1.0 - (2.0 * l - 1.0).abs());
    let h = if max == r {
        60.0 * ((g - b) / d).rem_euclid(6.0)
    } else if max == g {
        60.0 * ((b - r) / d + 2.0)
    } else {
        60.0 * ((r - g) / d + 4.0)
    };
    (h, s, l)
}

/// 16 colours carry a role, not a value: the terminal's user picked those
/// sixteen RGBs, so grounds and body text stay the terminal's own (`Reset`)
/// and only chromatic roles pick a hue bucket.
fn ansi16(role: Role, c: Rgb) -> Color {
    let (h, s, l) = hsl(c);
    let chromatic = s > 0.25 && l > 0.12 && l < 0.92;
    match role {
        Role::Surface => Color::Reset,
        Role::Ink => Color::Reset,
        Role::MutedInk => Color::DarkGray,
        Role::OnFill => {
            if l < 0.5 {
                Color::Black
            } else {
                Color::White
            }
        }
        Role::Line if !chromatic || l < 0.3 => Color::DarkGray,
        _ if !chromatic => {
            if l < 0.5 {
                Color::Black
            } else {
                Color::White
            }
        }
        _ => {
            let bright = l > 0.55;
            let pick = |dim: Color, lit: Color| if bright { lit } else { dim };
            match h {
                h if !(20.0..345.0).contains(&h) => pick(Color::Red, Color::LightRed),
                h if h < 75.0 => pick(Color::Yellow, Color::LightYellow),
                h if h < 165.0 => pick(Color::Green, Color::LightGreen),
                h if h < 200.0 => pick(Color::Cyan, Color::LightCyan),
                h if h < 260.0 => pick(Color::Blue, Color::LightBlue),
                _ => pick(Color::Magenta, Color::LightMagenta),
            }
        }
    }
}

/// Relative luminance, WCAG 2.1.
fn luminance(c: Rgb) -> f32 {
    let ch = |v: u8| {
        let v = v as f32 / 255.0;
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * ch(c.0) + 0.7152 * ch(c.1) + 0.0722 * ch(c.2)
}

/// The contrast ratio between two painted colours, 1.0 to 21.0.
///
/// The same reading `gates/colour.mjs` takes in kp-themes: the channels are
/// already whole bytes here, which is what that gate rounds to before it
/// measures, so the two agree.
pub fn contrast(a: Rgb, b: Rgb) -> f32 {
    let (x, y) = (luminance(a), luminance(b));
    let (hi, lo) = if x > y { (x, y) } else { (y, x) };
    (hi + 0.05) / (lo + 0.05)
}
