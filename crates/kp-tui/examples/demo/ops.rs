//! homelab's dashboard, rebuilt on this crate — the second proof.
//!
//! The original is `client/src/tui/view/dashboard.rs`, 342 lines: the host
//! mesh, the capacity panel, the LXC table and the live transfers. It is
//! the screen homelab opens on after its splash, and it is the one that
//! leans hardest on widgets — two hand-drawn bars, a table with a
//! selection and a scanline, a spinner, and a stream of marching dots.
//!
//! What the crate could not supply is marked `GAP` inline, the way
//! `fleet.rs` did before its three gaps were closed.

use kp_tui::{
    Badge, Column, DataTable, Facts, Meter, Spark, Stage, Theme, Tone, fx::Motion, source_colour,
    spinner, widgets::Panel,
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

pub fn draw(frame: &mut Frame, th: &Theme, selected: usize, reveal_ms: u32, motion: Motion) {
    let screen = frame.area();
    let stage = Stage::new(reveal_ms, motion);
    let [left, right] =
        Layout::horizontal([Constraint::Min(50), Constraint::Length(38)]).areas(screen);
    let [host, fleet] = Layout::vertical([Constraint::Length(5), Constraint::Min(6)]).areas(left);
    let [capacity, transfers] =
        Layout::vertical([Constraint::Length(7), Constraint::Min(4)]).areas(right);
    draw_host(frame, th, host, stage, reveal_ms, motion);
    draw_fleet(frame, th, fleet, selected, stage, reveal_ms, motion);
    draw_capacity(frame, th, capacity, stage, reveal_ms, motion);
    draw_transfers(frame, th, transfers, stage, reveal_ms, motion);
}

fn draw_host(
    frame: &mut Frame,
    th: &Theme,
    area: Rect,
    stage: Stage,
    reveal_ms: u32,
    motion: Motion,
) {
    let panel = Panel::new(th, "Host mesh")
        .focused(true)
        .stage(stage)
        .reveal(reveal_ms, motion);
    let inner = panel.block().inner(area);
    frame.render_widget(panel, area);
    let [name, disk, tls] = Layout::vertical([Constraint::Length(1); 3]).areas(inner);

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

    // homelab draws this bar by hand out of block characters; the Meter is
    // the same bar with the theme's own thresholds, and it fills in
    // eighths of a cell.
    frame.render_widget(Meter::new(th, "ssd", HOST.disk_pct), disk);

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
}

fn draw_capacity(
    frame: &mut Frame,
    th: &Theme,
    area: Rect,
    stage: Stage,
    reveal_ms: u32,
    motion: Motion,
) {
    let panel = Panel::new(th, "Capacity")
        .stage(stage)
        .reveal(reveal_ms, motion);
    let inner = panel.block().inner(area);
    frame.render_widget(panel, area);
    let [ram, facts, load] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Min(1),
    ])
    .areas(inner);

    let used = HOST.ram_used_mb as f32 / HOST.ram_total_mb as f32;
    frame.render_widget(Meter::new(th, "ram", used).thresholds(0.75, 0.9), ram);

    let free = HOST.ram_total_mb - HOST.ram_used_mb;
    let over = HOST.ram_committed_mb as f32 / HOST.ram_total_mb as f32;
    let rows = [
        (
            "free",
            format!("{} MB", free),
            if used > 0.9 {
                Tone::Danger
            } else {
                Tone::Success
            },
        ),
        ("alloc", format!("{:.0}%", over * 100.0), Tone::Warning),
        (
            "load",
            format!("{:.2} of {}", HOST.load1, HOST.cores),
            if HOST.load1 > HOST.cores as f32 {
                Tone::Danger
            } else {
                Tone::Success
            },
        ),
        ("cores", HOST.cores.to_string(), Tone::Ink),
    ];
    Facts::new(th, &rows)
        .columns(2)
        .render(facts, frame.buffer_mut());

    // The load of the last quarter hour, four levels to the row.
    let history: Vec<f64> = (0..36)
        .map(|i| 2.6 + ((i as f64) / 5.0).sin() * 1.4 + (i as f64) * 0.02)
        .collect();
    Spark::new(th, &history)
        .max(HOST.cores as f64)
        .warn_above(0.75)
        .render(load, frame.buffer_mut());
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
        Column::new("node", 18),
        Column::new("status", 10),
        // GAP: DataTable puts nothing between two columns, so a
        // right-aligned count ends flush against the flags beside it.
        // Left-aligned, the column carries its own gutter.
        Column::new("apps", 8),
        Column::new("flags", 16),
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
            // GAP — an indeterminate stream. homelab marches `▸` along the
            // row once a tick; the crate has no widget that says "this is
            // running and nobody knows for how long", so the march is
            // written here, in the demo, with its colour chosen by hand.
            None => {
                let width = flow.width as usize;
                let head = (reveal_ms / 90) as usize % width.max(1);
                let march: String = (0..width)
                    .map(|i| {
                        if (i + head).is_multiple_of(4) {
                            '▸'
                        } else {
                            '·'
                        }
                    })
                    .collect();
                Paragraph::new(Line::from(Span::styled(
                    march,
                    Style::new().fg(th.c.accent),
                )))
                .style(Style::new().bg(th.c.card))
                .render(flow, frame.buffer_mut());
            }
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
