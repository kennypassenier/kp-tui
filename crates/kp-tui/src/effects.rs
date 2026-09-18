//! Effects: the motion that is not a reveal — an attention burst, a
//! breathing plate, a texture sweep, a marquee, a spinner.
//!
//! Every one is a pure function of (elapsed time, element id), the rule
//! `fx.rs` already follows for the reveal: nothing is kept between
//! frames, so replaying, testing and switching theme mid-effect are all
//! free.
//!
//! Where they come from: homelab's `client/src/tui/fx.rs` hand-rolls six
//! of these (glitch, decrypt, pulse, scanline, ticker, flicker, spinner)
//! against eighteen colour literals and a fixed 30 fps tick. The
//! mechanics are carried over; the colours, the periods and the glyphs
//! now come from the theme, and time is milliseconds so a demo at 15 fps
//! and a terminal at 60 look the same.

use kp_tui_palette::Rgb;

use crate::fx::{GLYPHS, Motion};

/// SplitMix64 on a pair — the same hash `fx.rs` reveals with, so an
/// effect and a reveal on the same element do not correlate.
pub(crate) fn hash(a: u64, b: u64) -> u64 {
    let mut z = a.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ b.wrapping_add(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Linear blend in sRGB, `t` clamped to 0..=1. Good enough for two
/// colours out of one theme, which are never far apart in hue.
pub fn mix(a: Rgb, b: Rgb, t: f32) -> Rgb {
    let t = t.clamp(0.0, 1.0);
    let c = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    Rgb(c(a.0, b.0), c(a.1, b.1), c(a.2, b.2))
}

/// A breath: `a` at the start, `b` at half a period, `a` again at the
/// end. Reduced motion holds the colour it starts from.
pub fn pulse(a: Rgb, b: Rgb, elapsed_ms: u32, period_ms: u32, motion: Motion) -> Rgb {
    if motion == Motion::Reduced || period_ms == 0 {
        return a;
    }
    let phase = (elapsed_ms % period_ms) as f32 / period_ms as f32;
    // A sine rather than a triangle: no corner at the turn.
    let t = 0.5 - 0.5 * (phase * std::f32::consts::TAU).cos();
    mix(a, b, t)
}

/// A lit row crossing a panel of height `h`: `sweep_ms` to cross, once
/// every `period_ms`. `id` staggers the panels so they do not cross in
/// lockstep.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sweep {
    pub period_ms: u32,
    pub sweep_ms: u32,
}

impl Sweep {
    pub fn row(self, h: u16, elapsed_ms: u32, id: u64, motion: Motion) -> Option<u16> {
        if motion == Motion::Reduced || h == 0 || self.period_ms == 0 || self.sweep_ms == 0 {
            return None;
        }
        let offset = (hash(id, 7) % self.period_ms as u64) as u32;
        let phase = (elapsed_ms + offset) % self.period_ms;
        (phase < self.sweep_ms)
            .then(|| ((phase as f32 / self.sweep_ms as f32) * h as f32) as u16)
            .filter(|row| *row < h)
    }
}

/// A burst of wrong characters: `share` of the line, held for `hold_ms`,
/// returning every `every_ms`. `every_ms: 0` is one burst and then
/// nothing, which is what the registers that refuse a loop declare
/// (DI5: "flicker at .15s infinite was measured and refused").
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Glitch {
    pub every_ms: u32,
    pub hold_ms: u32,
    pub share: f32,
}

impl Glitch {
    /// Which burst is running, if one is: its number, so the substituted
    /// glyphs hold still for the length of the burst rather than
    /// churning every frame.
    fn burst(self, elapsed_ms: u32) -> Option<u64> {
        if self.hold_ms == 0 {
            return None;
        }
        match self.every_ms {
            0 => (elapsed_ms < self.hold_ms).then_some(0),
            every => {
                let n = elapsed_ms / every;
                (elapsed_ms % every < self.hold_ms).then_some(n as u64 + 1)
            }
        }
    }

    /// The line as it is drawn this frame. Whitespace is never replaced:
    /// the word shapes have to survive, or it reads as noise rather than
    /// as the same line under interference.
    pub fn text(self, text: &str, elapsed_ms: u32, id: u64, motion: Motion) -> String {
        let Some(burst) = self.burst(elapsed_ms).filter(|_| motion == Motion::Full) else {
            return text.to_string();
        };
        text.chars()
            .enumerate()
            .map(|(i, c)| {
                if c.is_whitespace() {
                    return c;
                }
                let r = hash(id ^ burst, i as u64);
                if (r % 1000) as f32 / 1000.0 >= self.share {
                    return c;
                }
                GLYPHS[(hash(r, 0xD1) % GLYPHS.len() as u64) as usize]
            })
            .collect()
    }
}

/// A line of segments sliding left at `cps` characters a second, joined
/// by the theme's own divider. Reduced motion shows the head of the line
/// and leaves it there.
pub fn marquee(
    segments: &[String],
    width: u16,
    elapsed_ms: u32,
    cps: f32,
    divider: &str,
    motion: Motion,
) -> String {
    let joined = segments.join(divider);
    if joined.is_empty() || width == 0 {
        return String::new();
    }
    let line: Vec<char> = format!("{joined}{divider}").chars().collect();
    let offset = if motion == Motion::Reduced || cps <= 0.0 {
        0
    } else {
        ((elapsed_ms as f32 * cps / 1000.0) as usize) % line.len()
    };
    (0..width as usize)
        .map(|i| line[(offset + i) % line.len()])
        .collect()
}

/// A panel striking like a failing tube, once, on arrival. The stops are
/// `@keyframes kp-alarm-cyberpunk-flicker` in `css/cyberpunk-register.css`
/// [scope-100]: catch at 60 %, sag to 52 % with a sideways kick, snap to
/// full — deliberately shallow, so DI5's reading stays at one flash a
/// second.
///
/// A terminal has no opacity, so the first number is how far the ink has
/// come from the ground towards its own colour, which is what a caller
/// mixes with. GUESS: the kick is ±0.6 % of a panel in CSS, which in a
/// cell grid is either nothing or one whole cell; one cell is the nearest
/// a character grid has, and a panel that does not move at all loses the
/// strike.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Strike {
    pub ms: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StrikeFrame {
    /// 0.0 = the ground's colour, 1.0 = the ink's own.
    pub ink: f32,
    /// Columns the panel is kicked sideways, -1, 0 or 1.
    pub kick: i16,
}

impl Strike {
    pub fn at(self, elapsed_ms: u32, motion: Motion) -> Option<StrikeFrame> {
        if motion == Motion::Reduced || self.ms == 0 || elapsed_ms >= self.ms {
            return None;
        }
        let pct = elapsed_ms as f32 / self.ms as f32;
        Some(match pct {
            p if p < 0.18 => StrikeFrame { ink: 0.0, kick: 0 },
            p if p < 0.30 => StrikeFrame { ink: 0.6, kick: -1 },
            p if p < 0.42 => StrikeFrame { ink: 0.52, kick: 1 },
            _ => StrikeFrame { ink: 1.0, kick: 0 },
        })
    }
}

/// The frames a theme turns while it waits. `.kp-spinner` is a ring in
/// every register; what differs is whether the corners are cut
/// (`border-radius: 0`) and how loud the theme is, which in a cell grid
/// is the difference between a braille ring, a block quadrant and a
/// bar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Spinner {
    /// A round ring: ten braille frames.
    Braille,
    /// A square ring: the quadrant blocks, four frames.
    Quadrant,
    /// A square ring, heavier: the half blocks.
    Half,
    /// No ring at all: a bar that fills and empties.
    Bar,
    /// Plain ASCII, for a theme that would not draw a ring either.
    Ascii,
}

impl Spinner {
    pub const fn frames(self) -> &'static [&'static str] {
        match self {
            Self::Braille => &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            Self::Quadrant => &["▘", "▝", "▗", "▖"],
            Self::Half => &["▀", "▐", "▄", "▌"],
            Self::Bar => &["▁", "▃", "▄", "▆", "▇", "▆", "▄", "▃"],
            Self::Ascii => &["|", "/", "-", "\\"],
        }
    }

    /// One turn every `period_ms`; reduced motion holds the first frame.
    pub fn frame(self, elapsed_ms: u32, period_ms: u32, motion: Motion) -> &'static str {
        let frames = self.frames();
        if motion == Motion::Reduced || period_ms == 0 {
            return frames[0];
        }
        let per = (period_ms as usize / frames.len()).max(1);
        frames[(elapsed_ms as usize / per) % frames.len()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_pulse_returns_to_where_it_started() {
        let (a, b) = (Rgb(0, 0, 0), Rgb(100, 200, 40));
        assert_eq!(pulse(a, b, 0, 2600, Motion::Full), a);
        assert_eq!(pulse(a, b, 1300, 2600, Motion::Full), b);
        assert_eq!(pulse(a, b, 2600, 2600, Motion::Full), a);
        assert_eq!(pulse(a, b, 1300, 2600, Motion::Reduced), a);
    }

    #[test]
    fn a_sweep_crosses_the_panel_once_a_period() {
        let s = Sweep {
            period_ms: 10_000,
            sweep_ms: 1_200,
        };
        let lit: Vec<u16> = (0..12_000)
            .step_by(50)
            .filter_map(|ms| s.row(10, ms, 3, Motion::Full))
            .collect();
        assert!(lit.contains(&0) && lit.contains(&9), "{lit:?}");
        assert!(lit.iter().all(|r| *r < 10));
        // Two panels do not sweep together.
        let rows = |id| -> Vec<Option<u16>> {
            (0..10_000)
                .step_by(100)
                .map(|ms| s.row(10, ms, id, Motion::Full))
                .collect()
        };
        assert_ne!(rows(1), rows(2));
        assert!(s.row(10, 500, 3, Motion::Reduced).is_none());
    }

    #[test]
    fn a_glitch_holds_its_glyphs_for_the_burst_and_keeps_the_spaces() {
        let g = Glitch {
            every_ms: 4_000,
            hold_ms: 120,
            share: 0.35,
        };
        let text = "power draw nominal";
        let plain = g.text(text, 2_000, 9, Motion::Full);
        assert_eq!(plain, text, "outside a burst the line is itself");
        let a = g.text(text, 4_020, 9, Motion::Full);
        let b = g.text(text, 4_090, 9, Motion::Full);
        assert_ne!(a, text);
        assert_eq!(a, b, "the same burst draws the same glyphs");
        assert_eq!(
            a.chars().filter(|c| *c == ' ').count(),
            text.chars().filter(|c| *c == ' ').count()
        );
        assert_eq!(a.chars().count(), text.chars().count());
        assert_eq!(g.text(text, 4_020, 9, Motion::Reduced), text);
    }

    #[test]
    fn a_single_burst_never_comes_back() {
        let g = Glitch {
            every_ms: 0,
            hold_ms: 200,
            share: 0.5,
        };
        assert_ne!(g.text("alarm", 10, 1, Motion::Full), "alarm");
        for ms in (200..20_000).step_by(37) {
            assert_eq!(g.text("alarm", ms, 1, Motion::Full), "alarm");
        }
    }

    #[test]
    fn a_marquee_fills_the_width_and_wraps() {
        let segs = vec!["cpu 12%".to_string(), "mem 41%".to_string()];
        let line = marquee(&segs, 20, 0, 8.0, " │ ", Motion::Full);
        assert_eq!(line.chars().count(), 20);
        let later = marquee(&segs, 20, 1_000, 8.0, " │ ", Motion::Full);
        assert_ne!(line, later);
        // One full turn of the joined line comes back to the start.
        let len = "cpu 12% │ mem 41% │ ".chars().count() as f32;
        let round = (len / 8.0 * 1000.0) as u32;
        assert_eq!(marquee(&segs, 20, round, 8.0, " │ ", Motion::Full), line);
        assert_eq!(marquee(&segs, 20, 1_000, 8.0, " │ ", Motion::Reduced), line);
    }

    #[test]
    fn a_spinner_turns_once_a_period() {
        let s = Spinner::Braille;
        assert_eq!(s.frame(0, 900, Motion::Full), "⠋");
        assert_eq!(s.frame(900, 900, Motion::Full), "⠋");
        assert_ne!(s.frame(450, 900, Motion::Full), "⠋");
        assert_eq!(s.frame(450, 900, Motion::Reduced), "⠋");
        for s in [
            Spinner::Braille,
            Spinner::Quadrant,
            Spinner::Half,
            Spinner::Bar,
            Spinner::Ascii,
        ] {
            assert!(s.frames().iter().all(|f| f.chars().count() == 1));
        }
    }

    #[test]
    fn a_strike_is_over_when_the_keyframe_is() {
        let s = Strike { ms: 600 };
        assert_eq!(s.at(0, Motion::Full).unwrap().ink, 0.0);
        assert_eq!(s.at(120, Motion::Full).unwrap().kick, -1);
        assert_eq!(s.at(190, Motion::Full).unwrap().kick, 1);
        // Both visible jumps go the same way: 0 -> 0.6 -> 0.52 -> 1 has one
        // dip, and it is 0.08 deep, under DI5's ten per cent step.
        let dip = s.at(120, Motion::Full).unwrap().ink - s.at(190, Motion::Full).unwrap().ink;
        assert!((dip - 0.08).abs() < 1e-6, "{dip}");
        assert_eq!(s.at(300, Motion::Full).unwrap().ink, 1.0);
        assert!(s.at(600, Motion::Full).is_none(), "once, then never again");
        assert!(s.at(100, Motion::Reduced).is_none());
    }
}

/// A screen arriving, in the order the package declares.
///
/// `css/components.css` gives four beats — the ground in over 240 ms, the
/// panels settling over 520 ms, the titles arriving over 480 ms, the detail
/// over 300 ms — and the web has used them since the alarm was built. A
/// terminal had none of it: panels were simply there. This is the same
/// sequence as four clocks, so a caller asks "how far in is the panel
/// beat" instead of keeping state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stage {
    pub elapsed_ms: u32,
    pub motion: Motion,
}

impl Stage {
    pub const GROUND_MS: u32 = 240;
    pub const PANEL_MS: u32 = 520;
    pub const TITLE_MS: u32 = 480;
    pub const DETAIL_MS: u32 = 300;

    pub fn new(elapsed_ms: u32, motion: Motion) -> Self {
        Stage { elapsed_ms, motion }
    }

    fn beat(&self, start: u32, length: u32) -> f32 {
        if self.motion == Motion::Reduced {
            return 1.0;
        }
        let t = (self.elapsed_ms.saturating_sub(start)) as f32 / length as f32;
        // The same cubic ease-out `--fx-ease` approximates for a reveal.
        1.0 - (1.0 - t.clamp(0.0, 1.0)).powi(3)
    }

    pub fn ground(&self) -> f32 {
        self.beat(0, Self::GROUND_MS)
    }

    pub fn panel(&self) -> f32 {
        self.beat(Self::GROUND_MS, Self::PANEL_MS)
    }

    pub fn title(&self) -> f32 {
        self.beat(Self::GROUND_MS + Self::PANEL_MS, Self::TITLE_MS)
    }

    pub fn detail(&self) -> f32 {
        self.beat(
            Self::GROUND_MS + Self::PANEL_MS + Self::TITLE_MS,
            Self::DETAIL_MS,
        )
    }

    /// When the last beat has finished, a caller can stop asking.
    pub fn done(&self) -> bool {
        self.motion == Motion::Reduced
            || self.elapsed_ms
                >= Self::GROUND_MS + Self::PANEL_MS + Self::TITLE_MS + Self::DETAIL_MS
    }
}

/// A number on its way to a new value: eased over `ms`, never jumping.
///
/// A figure that snaps reads as a different figure; one that travels reads
/// as the same figure changing. No loop, so the flash reading DI5 takes
/// stays at zero.
pub fn roll(from: f64, to: f64, elapsed_ms: u32, ms: u32, motion: Motion) -> f64 {
    if motion == Motion::Reduced || ms == 0 || elapsed_ms >= ms {
        return to;
    }
    let t = elapsed_ms as f32 / ms as f32;
    let eased = 1.0 - (1.0 - t).powi(3);
    from + (to - from) * eased as f64
}
