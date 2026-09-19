//! homelab's dashboard, rebuilt on this crate — the second proof.
//!
//! The original is `client/src/tui/view/dashboard.rs`, 342 lines: the host
//! mesh, the capacity panel, the LXC table and the live transfers. It is
//! the screen homelab opens on after its splash, and it is the one that
//! leans hardest on widgets — two hand-drawn bars, a table with a
//! selection and a scanline, a spinner, and a stream of marching dots.
//!
//! It found one defect and two gaps on the way [fix-2, docs/HOMELAB_PROOF.md];
//! all three were closed in the crate, so nothing on this screen is marked
//! `GAP` any more and no colour on it is chosen here.
//!
//! Kenny picked **Sparks behind** out of five directions: every reading
//! carries its own last ten minutes under it, so a number that is high
//! says whether it has been high all along. The host's own facts, the
//! stack table and the transfers all stayed where they were — the
//! direction changes how the numbers read, not what the screen can do
//! [fix-65]. The four bars share one label column [fix-64].

use kp_tui::{
    Badge, Column, DataTable, Facts, Meter, Spark, Stage, Stream, Theme, Tone, fx::Motion,
    source_colour, spinner, widgets::Panel,
};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use crate::fleet::FLEET;

pub struct Host {
    pub name: &'static str,
    pub disk_pct: f32,
    pub tls: &'static str,
    pub ram_used_mb: u32,
    pub ram_total_mb: u32,
    pub ram_committed_mb: u32,
    pub load1: f32,
    pub cores: u32,
}

pub const HOST: Host = Host {
    name: "pve-01",
    disk_pct: 0.61,
    tls: "9F:2C:A1:0E:77:B4",
    ram_used_mb: 41_920,
    ram_total_mb: 65_536,
    ram_committed_mb: 88_064,
    load1: 3.4,
    cores: 8,
};

pub struct Transfer {
    pub label: &'static str,
    pub done: u64,
    pub total: Option<u64>,
}

pub const TRANSFERS: [Transfer; 2] = [
    Transfer {
        label: "media/jellyfin.tar",
        done: 412_000_000,
        total: Some(980_000_000),
    },
    Transfer {
        label: "backup/restic-index",
        done: 88_400_000,
        total: None,
    },
];

/// The readings the dashboard leads with, each with the ten minutes
/// behind it. `at` is the reading now; `max` is what a full bar means.
struct Reading {
    label: &'static str,
    value: f32,
    max: f32,
    /// What the number is called when it is written out in full.
    says: String,
    warn: f32,
    danger: f32,
}

/// Ten minutes of history, one sample every three seconds — two to a
/// cell column, so the strip is full from edge to edge on a wide screen.
/// A fixture, like every other number here: the live sampler feeds the
/// dashboard screen, not this one.
fn behind(seed: f64, level: f64) -> Vec<f64> {
    (0..200)
        .map(|i| {
            let t = i as f64 + seed;
            (level + (t / 21.0).sin() * level * 0.35 + (t / 7.0).cos() * level * 0.12).max(0.0)
        })
        .collect()
}

pub fn draw(frame: &mut Frame, th: &Theme, selected: usize, reveal_ms: u32, motion: Motion) {
    let screen = frame.area();
    let stage = Stage::new(reveal_ms, motion);
    let [left, right] =
        Layout::horizontal([Constraint::Min(50), Constraint::Length(38)]).areas(screen);
    let [readings, fleet] =
        Layout::vertical([Constraint::Length(14), Constraint::Min(6)]).areas(left);
    let [host, transfers] =
        Layout::vertical([Constraint::Length(7), Constraint::Min(4)]).areas(right);
    draw_readings(frame, th, readings, stage, reveal_ms, motion);
    draw_fleet(frame, th, fleet, selected, stage, reveal_ms, motion);
    draw_host(frame, th, host, stage, reveal_ms, motion);
    draw_transfers(frame, th, transfers, stage, reveal_ms, motion);
}

/// The four readings, each three rows: the bar with its label and its
/// number, then the ten minutes behind it.
fn draw_readings(
    frame: &mut Frame,
    th: &Theme,
    area: Rect,
    stage: Stage,
    reveal_ms: u32,
    motion: Motion,
) {
    let panel = Panel::new(th, "Host — now, and the last ten minutes")
        .focused(true)
        .stage(stage)
        .reveal(reveal_ms, motion);
    let inner = panel.block().inner(area);
    frame.render_widget(panel, area);

    let used = HOST.ram_used_mb as f32 / HOST.ram_total_mb as f32;
    let readings = [
        Reading {
            label: "load",
            value: HOST.load1 / HOST.cores as f32,
            max: HOST.cores as f32,
            says: format!("{:.2} of {} cores", HOST.load1, HOST.cores),
            warn: 0.75,
            danger: 1.0,
        },
        Reading {
            label: "memory",
            value: used,
            max: 1.0,
            says: format!("{} of {} MB", HOST.ram_used_mb, HOST.ram_total_mb),
            warn: 0.75,
            danger: 0.9,
        },
        Reading {
            label: "ssd",
            value: HOST.disk_pct,
            max: 1.0,
            says: "root pool".to_string(),
            warn: 0.8,
            danger: 0.92,
        },
        Reading {
            label: "network",
            // A gigabit link, so 125 MB/s is the bar's own full.
            value: 41.0 / 125.0,
            max: 125.0,
            says: "41 MB/s of 125".to_string(),
            warn: 0.7,
            danger: 0.9,
        },
    ];
    let labels: Vec<&str> = readings.iter().map(|r| r.label).collect();
    let column = kp_tui::label_column(th, &labels);

    let rows = Layout::vertical([Constraint::Length(3); 4]).split(inner);
    for (i, r) in readings.iter().enumerate() {
        let Some(slot) = rows.get(i) else { break };
        let [bar, note, strip] = Layout::vertical([Constraint::Length(1); 3]).areas(*slot);
        Meter::new(th, r.label, r.value)
            .label_width(column)
            .thresholds(r.warn, r.danger)
            .render(bar, frame.buffer_mut());
        Paragraph::new(Line::from(Span::styled(
            format!("{:>width$}  {}", "", r.says, width = column),
            Style::new().fg(th.c.muted_foreground),
        )))
        .style(Style::new().bg(th.c.card))
        .render(note, frame.buffer_mut());
        Spark::new(th, &behind(i as f64 * 7.0, (r.value * r.max) as f64))
            .max(r.max as f64)
            .warn_above(r.warn as f64)
            .render(strip, frame.buffer_mut());
    }
}

/// The host itself: what it is called, the certificate homelab prints,
/// and the three numbers the capacity panel used to carry.
fn draw_host(
    frame: &mut Frame,
    th: &Theme,
    area: Rect,
    stage: Stage,
    reveal_ms: u32,
    motion: Motion,
) {
    let panel = Panel::new(th, "Host mesh")
        .stage(stage)
        .reveal(reveal_ms, motion);
    let inner = panel.block().inner(area);
    frame.render_widget(panel, area);
    let [name, tls, facts] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(1),
    ])
    .areas(inner);

    let mut spans = vec![
        Badge::dot(th, true),
        Span::raw(" "),
        Span::styled(
            HOST.name.to_string(),
            Style::new()
                .fg(th.c.card_foreground)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
    ];
    spans.extend(Badge::new(th, "online", Tone::Success).spans());
    Paragraph::new(Line::from(spans))
        .style(Style::new().bg(th.c.card))
        .render(name, frame.buffer_mut());

    Paragraph::new(Line::from(vec![
        Span::styled("tls ", Style::new().fg(th.c.muted_foreground)),
        Span::styled(
            HOST.tls.to_string(),
            Style::new().fg(th.ink(Tone::Success, th.id.palette().card)),
        ),
        Span::styled("  ", Style::new()),
        Span::styled(
            spinner(th, reveal_ms, motion),
            Style::new().fg(th.c.primary),
        ),
    ]))
    .style(Style::new().bg(th.c.card))
    .render(tls, frame.buffer_mut());

    let used = HOST.ram_used_mb as f32 / HOST.ram_total_mb as f32;
    let free = HOST.ram_total_mb - HOST.ram_used_mb;
    let over = HOST.ram_committed_mb as f32 / HOST.ram_total_mb as f32;
    let rows = [
        (
            "free",
            format!("{free} MB"),
            if used > 0.9 {
                Tone::Danger
            } else {
                Tone::Success
            },
        ),
        ("alloc", format!("{:.0}%", over * 100.0), Tone::Warning),
        ("cores", HOST.cores.to_string(), Tone::Ink),
        ("uptime", "19 d".to_string(), Tone::Ink),
    ];
    Facts::new(th, &rows)
        .columns(2)
        .render(facts, frame.buffer_mut());
}

fn draw_fleet(
    frame: &mut Frame,
    th: &Theme,
    area: Rect,
    selected: usize,
    stage: Stage,
    reveal_ms: u32,
    motion: Motion,
) {
    let title = format!("Lxc mesh · {} nodes", FLEET.len());
    let panel = Panel::new(th, &title)
        .stage(stage)
        .reveal(reveal_ms, motion);
    let inner = panel.block().inner(area);
    frame.render_widget(panel, area);

    let columns = [
        Column::new("node", 17),
        Column::new("status", 8),
        Column::new("apps", 5).right(),
        Column::new("flags", 15),
    ];
    let rows: Vec<Vec<Line<'static>>> = FLEET
        .iter()
        .map(|s| {
            let running = s.apps.iter().filter(|a| a.1).count();
            let mut flags = Vec::new();
            // A register that plates its badges gives them no margin of
            // their own, so two badges in a row need the space between
            // them from whoever puts them there.
            let mut badge = |text: &'static str, tone| {
                if !flags.is_empty() {
                    flags.push(Span::raw(" "));
                }
                flags.extend(Badge::new(th, text, tone).spans());
            };
            if !s.enabled {
                badge("off", Tone::MutedInk);
            }
            if s.drift {
                badge("upd", Tone::Warning);
            }
            if !s.env_sealed {
                badge("noenv", Tone::Danger);
            }
            vec![
                Line::from(vec![
                    // The stack's own hue as a bar down the leading edge,
                    // the way homelab marks a node.
                    Span::styled("▎", Style::new().fg(source_colour(th, s.name))),
                    Span::styled(s.name.to_string(), Style::new().fg(th.c.card_foreground)),
                ]),
                Line::from(vec![
                    Badge::dot(th, s.online),
                    Span::styled(
                        if s.online { " up" } else { " down" },
                        Style::new().fg(th.c.muted_foreground),
                    ),
                ]),
                Line::from(Span::styled(
                    format!("{running}/{}", s.apps.len()),
                    Style::new().fg(th.c.card_foreground),
                )),
                Line::from(flags),
            ]
        })
        .collect();
    DataTable::new(th, &columns, &rows)
        .selected(selected.min(FLEET.len() - 1))
        .render(inner, frame.buffer_mut());
}

fn draw_transfers(
    frame: &mut Frame,
    th: &Theme,
    area: Rect,
    stage: Stage,
    reveal_ms: u32,
    motion: Motion,
) {
    let panel = Panel::new(th, "Data transfers")
        .stage(stage)
        .reveal(reveal_ms, motion);
    let inner = panel.block().inner(area);
    frame.render_widget(panel, area);

    let rows = Layout::vertical([Constraint::Length(2); 2]).split(inner);
    for (t, rect) in TRANSFERS.iter().zip(rows.iter()) {
        let [label, flow] = Layout::vertical([Constraint::Length(1); 2]).areas(*rect);
        let name = t.label.rsplit('/').next().unwrap_or(t.label);
        Paragraph::new(Line::from(vec![
            Span::styled(name.to_string(), Style::new().fg(th.c.card_foreground)),
            Span::styled(
                match t.total {
                    Some(total) => format!("  {} / {}", bytes(t.done), bytes(total)),
                    None => format!("  {}", bytes(t.done)),
                },
                Style::new().fg(th.c.muted_foreground),
            ),
        ]))
        .style(Style::new().bg(th.c.card))
        .render(label, frame.buffer_mut());

        match t.total {
            // A transfer that knows its size is a Meter, thresholds off:
            // a bar that fills is not a warning.
            Some(total) => frame.render_widget(
                Meter::new(th, "", t.done as f32 / total as f32).thresholds(2.0, 2.0),
                flow,
            ),
            // A transfer whose size nobody knows is the package's own
            // indeterminate bar, in cells: the register's diagonal
            // drifting over the muted track [gap-11].
            None => frame.render_widget(Stream::new(th, reveal_ms, motion), flow),
        }
    }
}

fn bytes(n: u64) -> String {
    const UNITS: [&str; 4] = ["B", "kB", "MB", "GB"];
    let mut value = n as f64;
    let mut unit = 0;
    while value >= 1000.0 && unit < UNITS.len() - 1 {
        value /= 1000.0;
        unit += 1;
    }
    format!("{value:.0} {}", UNITS[unit])
}
