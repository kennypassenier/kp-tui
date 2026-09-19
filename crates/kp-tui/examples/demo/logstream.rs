//! homelab's log stream tab, rebuilt on this crate — the fourth proof.
//!
//! The original is `client/src/tui/view/logs.rs`, 139 lines: the multiplexed
//! feed from the host, a source selector along the top, arrow scrolling with
//! anchored scrollback, a level column, a source column in the stack's own
//! hue, and a lit row crossing the pane every ten seconds.
//!
//! The first pass said it found nothing missing. It had: the selector
//! painted itself and filtered nothing, and scrolling back down to the
//! last line never let go of the pause. Both are behaviours homelab has
//! and the rebuild had lost, and both are in the crate now [fix-65].
//!
//! Kenny picked **Density band** out of five directions, so the pane now
//! carries how much arrived when over the lines themselves, with the
//! buckets that hold an error in the theme's danger ink. Nothing the
//! screen could do was given up for it: the source selector, the pause,
//! the anchored scrollback, the jump to the tail, the level filter, the
//! line count and the scrollbar are all still there, and the footer names
//! the key for each.

use kp_tui::{
    KeyHints, LogPane, Surface, Theme,
    fx::Motion,
    logs::{LogBuffer, LogLine, Severity},
};
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::Style,
    widgets::{Paragraph, Widget},
};

use crate::fleet::FLEET;

/// The feed as it stands at the moment of the shot: one line per event,
/// newest last, the way `journalctl --output=json` hands them over.
pub fn feed() -> LogBuffer {
    let mut buffer = LogBuffer::new(400);
    // Five minutes of a working evening: the stacks talking over each
    // other, a backup that fails twice, and a quiet spell in the middle.
    // A fixture — the pane says "synthetic, not real" in its own title.
    const LINES: [(&str, Severity, &str); 12] = [
        ("media", Severity::Info, "jellyfin: transcode worker ready"),
        ("web", Severity::Info, "caddy: 200 GET / 2.1 kB in 3 ms"),
        (
            "backup",
            Severity::Warning,
            "restic: repository locked by another process",
        ),
        (
            "monitoring",
            Severity::Info,
            "prometheus: scrape of node-exporter took 412 ms",
        ),
        ("dns", Severity::Info, "blocky: 1284 queries, 19 % blocked"),
        (
            "backup",
            Severity::Error,
            "restic: snapshot failed, lock is 41 minutes old",
        ),
        (
            "media",
            Severity::Debug,
            "jellyfin: cache hit for /Items/1a2b",
        ),
        (
            "web",
            Severity::Info,
            "caddy: certificate renewed, 89 days left",
        ),
        (
            "monitoring",
            Severity::Warning,
            "alertmanager: 1 alert firing, backup_failed",
        ),
        (
            "dns",
            Severity::Info,
            "blocky: blocklist refreshed, 148 291 entries",
        ),
        (
            "media",
            Severity::Info,
            "jellyfin: library scan finished in 2 m 14 s",
        ),
        ("web", Severity::Debug, "caddy: reusing upstream connection"),
    ];
    // The gap before each line, in milliseconds: a busy opening, a quiet
    // middle, and a burst around the backup that fails.
    let gaps = |i: u32| -> u32 {
        match i {
            0..=17 => 240,
            18..=35 => 1_900,
            36..=53 => 160,
            _ => 700,
        }
    };
    let mut ms = 41_000u32;
    for i in 0..72u32 {
        let (unit, severity, message) = LINES[i as usize % LINES.len()];
        ms += gaps(i);
        let time = format!(
            "09:{:02}:{:02}.{:03}",
            41 + ms / 60_000,
            ms / 1000 % 60,
            ms % 1000
        );
        buffer.push(LogLine::new(&time, "pve-01", unit, severity, message));
    }
    buffer
}

/// Every key the screen answers to, named where the reader can see them.
const KEYS: [(&str, &str); 6] = [
    ("←→", "source"),
    ("↑↓", "scroll — and pause"),
    ("space", "follow"),
    ("l", "level"),
    ("G", "tail"),
    ("q", "quit"),
];

/// The sources the selector offers: everything, then one per stack.
pub fn sources() -> Vec<&'static str> {
    let mut names = vec!["all"];
    names.extend(FLEET.iter().map(|s| s.name));
    names
}

pub fn draw(
    frame: &mut Frame,
    th: &Theme,
    buffer: &LogBuffer,
    selected: usize,
    reveal_ms: u32,
    motion: Motion,
) {
    let screen = frame.area();
    let [body, footer] =
        Layout::vertical([Constraint::Min(4), Constraint::Length(1)]).areas(screen);

    // The ground first: the register's own texture, and the row that
    // crosses the pane where a register declares a sweep. homelab writes
    // that scanline by hand in all three of its log views.
    Surface::new(th, th.id.palette().card)
        .at(reveal_ms, motion)
        .paint(body, frame.buffer_mut());

    let names = sources();
    LogPane::new(th, "Log stream", buffer)
        .sources(&names, selected)
        .density(3)
        .live(false)
        .render(body, frame.buffer_mut());

    Paragraph::new(KeyHints::new(th, &KEYS).footer())
        .style(Style::new().bg(th.c.background))
        .render(footer, frame.buffer_mut());
}
