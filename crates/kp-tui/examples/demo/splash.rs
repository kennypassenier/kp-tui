//! homelab's boot splash, rebuilt on this crate — the seventh proof, and
//! the last of its nine screens.
//!
//! The original is `client/src/tui/view/splash.rs`, 98 lines: a block
//! logo that materialises through the decrypt reveal, and a POST-style
//! boot log whose lines arrive one at a time, driven by the real link
//! state.
//!
//! It is the one screen in homelab's whole client where a colour is
//! computed rather than named: the logo walks a cyan-to-magenta gradient
//! written in raw RGB. That is the gap this screen found — a theme should
//! answer for its own ramp — and `Theme::ramp` is it [gap-16], so all
//! twenty-two registers colour this logo differently and none of them is
//! a literal.

use kp_tui::{
    Badge, Theme, Tone,
    fx::{self, Motion},
};
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

const LOGO: [&str; 6] = [
    r"██╗  ██╗██████╗   ████████╗██╗   ██╗██╗",
    r"██║ ██╔╝██╔══██╗     ██╔══╝██║   ██║██║",
    r"█████╔╝ ██████╔╝     ██║   ██║   ██║██║",
    r"██╔═██╗ ██╔═══╝      ██║   ██║   ██║██║",
    r"██║  ██╗██║          ██║   ╚██████╔╝██║",
    r"╚═╝  ╚═╝╚═╝          ╚═╝    ╚═════╝ ╚═╝",
];

/// The boot log, in the order it arrives. The tone is what the line
/// means, not what it is called.
pub const BOOT: [(&str, &str, Tone); 4] = [
    ("sys core", "control deck online", Tone::Info),
    ("host mesh", "link established · TLS pinned", Tone::Success),
    ("safety", "whitelist armed · no-touch enforced", Tone::Info),
    ("ready", "press any key", Tone::Success),
];

/// One boot line every 420 ms, after the logo has finished arriving.
const EVERY_MS: u32 = 420;
const LOGO_MS: u32 = 900;

pub fn draw(frame: &mut Frame, th: &Theme, reveal_ms: u32, motion: Motion) {
    let screen = frame.area();
    // The ground first, so the register's own texture lies under the
    // logo rather than over it.
    kp_tui::Surface::new(th, th.id.palette().background)
        .at(reveal_ms, motion)
        .paint(screen, frame.buffer_mut());

    let tall = LOGO.len() as u16 + 2 + BOOT.len() as u16;
    let top = screen.y + screen.height.saturating_sub(tall) / 2;

    let row = |y: u16| Rect {
        x: screen.x,
        y,
        width: screen.width,
        height: 1,
    };

    for (i, art) in LOGO.iter().enumerate() {
        // The logo arrives in the register's own routine; where a
        // register has none, `fx::frame` hands the text straight back and
        // the logo is simply there.
        let f = fx::frame(art, th.a.reveal, reveal_ms, motion);
        // Down the ramp, one step per row: the theme's own five chart
        // colours read between, where homelab computes cyan to magenta.
        let t = i as f32 / (LOGO.len() - 1) as f32;
        let colour = if reveal_ms >= LOGO_MS {
            th.ramp(t)
        } else {
            th.c.primary
        };
        Paragraph::new(Line::from(Span::styled(
            f.text,
            Style::new().fg(colour).add_modifier(Modifier::BOLD),
        )))
        .alignment(Alignment::Center)
        .render(row(top + i as u16), frame.buffer_mut());
    }

    let arrived = reveal_ms.saturating_sub(LOGO_MS) / EVERY_MS;
    let column = kp_tui::label_column(th, &BOOT.map(|(tag, _, _)| tag)) + 2;
    // The block is centred, not each line inside it: four lines centred
    // one by one would give four different left edges, which is the fault
    // Kenny found on the pictures [fix-64].
    let widest = BOOT
        .iter()
        .map(|(_, says, _)| column + 2 + says.chars().count())
        .max()
        .unwrap_or(0) as u16;
    let left = screen.x + screen.width.saturating_sub(widest) / 2;
    for (i, (tag, says, tone)) in BOOT.iter().enumerate() {
        if i as u32 > arrived {
            break;
        }
        let ink = th.ink(*tone, th.id.palette().background);
        let line = Line::from(vec![
            Badge::state(th, *tone),
            Span::raw(" "),
            Span::styled(
                format!("{tag:<column$}"),
                Style::new().fg(ink).add_modifier(Modifier::BOLD),
            ),
            Span::styled((*says).to_string(), Style::new().fg(th.c.muted_foreground)),
        ]);
        let at = Rect {
            x: left,
            width: widest.min(screen.width),
            ..row(top + LOGO.len() as u16 + 2 + i as u16)
        };
        Paragraph::new(line).render(at, frame.buffer_mut());
    }
}
