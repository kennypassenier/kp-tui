//! homelab's log stream tab, rebuilt on this crate — the fourth proof.
//!
//! The original is `client/src/tui/view/logs.rs`, 139 lines: the multiplexed
//! feed from the host, a source selector along the top, arrow scrolling with
//! anchored scrollback, a level column, a source column in the stack's own
//! hue, and a lit row crossing the pane every ten seconds.
//!
//! It found nothing missing: `LogPane` already draws the panel, the
//! source bar, the rows in each source's own hue and the scrollbar, and
//! `Surface` carries the register's texture and its sweep.

use kp_tui::{
    LogPane, Surface, Theme,
    fx::Motion,
    logs::{LogBuffer, LogLine, Severity},
};
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    widgets::Widget,
};

use crate::fleet::FLEET;

/// The feed as it stands at the moment of the shot: one line per event,
/// newest last, the way `journalctl --output=json` hands them over.
pub fn feed() -> LogBuffer {
    let mut buffer = LogBuffer::new(400);
    for (time, unit, severity, message) in [
        (
            "09:41:02.118",
            "media",
            Severity::Info,
            "jellyfin: transcode worker ready",
        ),
        (
            "09:41:02.664",
            "web",
            Severity::Info,
            "caddy: certificate renewed, 89 days left",
        ),
        (
            "09:41:03.201",
            "backup",
            Severity::Warning,
            "restic: repository locked by another process",
        ),
        (
            "09:41:03.998",
            "monitoring",
            Severity::Info,
            "prometheus: scrape of node-exporter took 412 ms",
        ),
        (
            "09:41:04.512",
            "dns",
            Severity::Info,
            "blocky: 1284 queries, 19 % blocked",
        ),
        (
            "09:41:05.077",
            "backup",
            Severity::Error,
            "restic: snapshot failed, lock is 41 minutes old",
        ),
        (
            "09:41:05.630",
            "media",
            Severity::Debug,
            "jellyfin: cache hit for /Items/1a2b",
        ),
        (
            "09:41:06.144",
            "web",
            Severity::Info,
            "caddy: 200 GET / 2.1 kB in 3 ms",
        ),
        (
            "09:41:06.702",
            "monitoring",
            Severity::Warning,
            "alertmanager: 1 alert firing, backup_failed",
        ),
        (
            "09:41:07.255",
            "dns",
            Severity::Info,
            "blocky: blocklist refreshed, 148 291 entries",
        ),
        (
            "09:41:07.810",
            "media",
            Severity::Info,
            "jellyfin: library scan finished in 2 m 14 s",
        ),
        (
            "09:41:08.366",
            "web",
            Severity::Debug,
            "caddy: reusing upstream connection",
        ),
    ] {
        buffer.push(LogLine::new(time, "pve-01", unit, severity, message));
    }
    buffer
}

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
    let [body] = Layout::vertical([Constraint::Min(4)]).areas(screen);

    // The ground first: the register's own texture, and the row that
    // crosses the pane where a register declares a sweep. homelab writes
    // that scanline by hand in all three of its log views.
    Surface::new(th, th.id.palette().card)
        .at(reveal_ms, motion)
        .paint(body, frame.buffer_mut());

    let names = sources();
    LogPane::new(th, "Log stream", buffer)
        .sources(&names, selected)
        .live(false)
        .render(body, frame.buffer_mut());
}
