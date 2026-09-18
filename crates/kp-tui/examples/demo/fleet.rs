//! homelab's stacks screen, rebuilt on this crate — the proof.
//!
//! The original is `client/src/tui/view/stacks.rs`, 186 lines: a registry
//! list on the left, a manifest and an app grid on the right, all of it in
//! one cyan-and-green look. This is the same screen with the same data
//! shape, drawn with the crate's widgets and no colour of its own.
//!
//! What the crate could not supply is written inline here with a
//! `GAP:` comment, so the shortfall is code a reader can count rather
//! than a claim in a document. `docs/HOMELAB_PROOF.md` totals it up.

use kp_tui::{
    SelectList, Theme,
    fx::{self, Motion},
    source_colour,
    widgets::Panel,
};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Cell, Paragraph, Row, Table, Widget},
};

pub struct Stack {
    pub name: &'static str,
    pub hostname: &'static str,
    pub vmid: u32,
    pub online: bool,
    pub drift: bool,
    pub enabled: bool,
    pub env_sealed: bool,
    pub apps: &'static [(&'static str, bool, u32)],
}

pub const FLEET: [Stack; 5] = [
    Stack {
        name: "media",
        hostname: "ct-media",
        vmid: 112,
        online: true,
        drift: true,
        enabled: true,
        env_sealed: true,
        apps: &[
            ("jellyfin", true, 0),
            ("sonarr", true, 0),
            ("radarr", true, 2),
            ("qbittorrent", false, 7),
        ],
    },
    Stack {
        name: "web",
        hostname: "ct-web",
        vmid: 104,
        online: true,
        drift: false,
        enabled: true,
        env_sealed: true,
        apps: &[("caddy", true, 0), ("ghost", true, 1), ("umami", true, 0)],
    },
    Stack {
        name: "backup",
        hostname: "ct-backup",
        vmid: 131,
        online: false,
        drift: false,
        enabled: false,
        env_sealed: false,
        apps: &[("restic", false, 0)],
    },
    Stack {
        name: "monitoring",
        hostname: "ct-mon",
        vmid: 120,
        online: true,
        drift: false,
        enabled: true,
        env_sealed: true,
        apps: &[("grafana", true, 0), ("prometheus", true, 0)],
    },
    Stack {
        name: "dns",
        hostname: "ct-dns",
        vmid: 101,
        online: true,
        drift: false,
        enabled: true,
        env_sealed: true,
        apps: &[("blocky", true, 0)],
    },
];

pub fn draw(frame: &mut Frame, th: &Theme, selected: usize, reveal_ms: u32, motion: Motion) {
    let screen = frame.area();
    let [left, right] =
        Layout::horizontal([Constraint::Length(32), Constraint::Min(40)]).areas(screen);
    draw_registry(frame, th, left, selected, reveal_ms, motion);
    draw_detail(
        frame,
        th,
        right,
        &FLEET[selected.min(FLEET.len() - 1)],
        reveal_ms,
        motion,
    );
}

/// GAP 1 — a state dot and a tag. homelab writes `●`/`○` and `[UPD]`,
/// `[OFF]` by hand; the package has `.kp-badge` on the web and this crate
/// has nothing, so the glyphs and their colours are chosen here.
fn dot(th: &Theme, online: bool) -> Span<'static> {
    if online {
        Span::styled("● ", Style::new().fg(th.c.success))
    } else {
        Span::styled("○ ", Style::new().fg(th.c.muted_foreground))
    }
}

fn tag(th: &Theme, text: &'static str, warn: bool) -> Span<'static> {
    Span::styled(
        text,
        Style::new().fg(if warn {
            th.c.warning
        } else {
            th.c.muted_foreground
        }),
    )
}

fn draw_registry(
    frame: &mut Frame,
    th: &Theme,
    area: Rect,
    selected: usize,
    reveal_ms: u32,
    motion: Motion,
) {
    let panel = Panel::new(th, "Stack registry")
        .focused(true)
        .reveal(reveal_ms, motion);
    let inner = panel.block().inner(area);
    frame.render_widget(panel, area);

    let items: Vec<Line<'static>> = FLEET
        .iter()
        .map(|s| {
            // The hostname arrives in the theme's own routine, where
            // homelab always decrypts it.
            let f = fx::frame(s.hostname, th.a.reveal, reveal_ms, motion);
            let mut spans = vec![
                dot(th, s.online),
                Span::styled(
                    f.text,
                    Style::new()
                        .fg(source_colour(th, s.name))
                        .add_modifier(Modifier::BOLD),
                ),
            ];
            if s.drift {
                spans.push(tag(th, " [UPD]", true));
            }
            if !s.enabled {
                spans.push(tag(th, " [OFF]", false));
            }
            Line::from(spans)
        })
        .collect();
    frame.render_widget(SelectList::new(th, &items, selected), inner);
}

fn draw_detail(
    frame: &mut Frame,
    th: &Theme,
    area: Rect,
    s: &Stack,
    reveal_ms: u32,
    motion: Motion,
) {
    let [manifest, grid] =
        Layout::vertical([Constraint::Length(5), Constraint::Min(5)]).areas(area);

    let title = format!("Manifest · {}", s.hostname);
    let panel = Panel::new(th, &title).reveal(reveal_ms, motion);
    let inner = panel.block().inner(manifest);
    frame.render_widget(panel, manifest);

    // GAP 2 — a facts list. Three rows of `label value   label value`,
    // aligned by hand. homelab does the same, five times over its screens.
    let key = Style::new().fg(th.c.muted_foreground);
    let ok = Style::new().fg(th.c.success);
    let warn = Style::new().fg(th.c.warning);
    let bad = Style::new().fg(th.c.destructive);
    let lines = vec![
        Line::from(vec![
            Span::styled("vmid ", key),
            Span::styled(s.vmid.to_string(), Style::new().fg(th.c.card_foreground)),
            Span::styled("   env ", key),
            if s.env_sealed {
                Span::styled("sealed", ok)
            } else {
                Span::styled("missing — deploy fails closed", bad)
            },
        ]),
        Line::from(vec![
            Span::styled("drift ", key),
            if s.drift {
                Span::styled("intent differs from applied", warn)
            } else {
                Span::styled("none — intent is runtime", ok)
            },
            Span::styled("   nightly ", key),
            if s.enabled {
                Span::styled("enabled", ok)
            } else {
                Span::styled("parked", warn)
            },
        ]),
        Line::from(vec![
            Span::styled("safety ", key),
            Span::styled("whitelist · hostname-guard · fail-closed", ok),
        ]),
    ];
    Paragraph::new(lines)
        .style(Style::new().bg(th.c.card))
        .render(inner, frame.buffer_mut());

    let title = format!("App grid · {} units", s.apps.len());
    let panel = Panel::new(th, &title);
    let inner = panel.block().inner(grid);
    frame.render_widget(panel, grid);

    // GAP 3 — a themed table. ratatui's own Table takes the styles one at
    // a time; the crate has no widget that gives a header, a rule and a
    // row rhythm the register's own way, so every style here is chosen in
    // the demo rather than asked of the theme.
    let header = Row::new(vec!["app", "state", "restarts"]).style(
        Style::new()
            .fg(th.c.muted_foreground)
            .add_modifier(th.a.title_modifier),
    );
    let rows: Vec<Row> = s
        .apps
        .iter()
        .map(|(name, running, restarts)| {
            let (label, style) = if *running {
                ("running", Style::new().fg(th.c.success))
            } else {
                ("down", Style::new().fg(th.c.destructive))
            };
            Row::new(vec![
                Cell::from(Span::styled(
                    (*name).to_string(),
                    Style::new().fg(th.c.card_foreground),
                )),
                Cell::from(Span::styled(label, style)),
                Cell::from(Span::styled(
                    restarts.to_string(),
                    if *restarts > 0 {
                        Style::new().fg(th.c.warning)
                    } else {
                        Style::new().fg(th.c.muted_foreground)
                    },
                )),
            ])
        })
        .collect();
    Table::new(
        rows,
        [
            Constraint::Length(16),
            Constraint::Length(9),
            Constraint::Min(8),
        ],
    )
    .header(header)
    .style(Style::new().bg(th.c.card))
    .render(inner, frame.buffer_mut());

    // The sweep the register declares crosses the grid, where homelab's
    // crosses it always and in one hard-coded colour.
    kp_tui::Surface::new(th, th.id.palette().card)
        .at(reveal_ms, motion)
        .id(0xA9)
        .paint(inner, frame.buffer_mut());
}
