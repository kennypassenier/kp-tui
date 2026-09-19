//! homelab's stacks screen, rebuilt on this crate — the proof.
//!
//! The original is `client/src/tui/view/stacks.rs`, 186 lines: a registry
//! list on the left, a manifest and an app grid on the right, all of it in
//! one cyan-and-green look. This is the same screen with the same data
//! shape, drawn with the crate's widgets and no colour of its own.
//!
//! It was written first against what the crate had, and the three things
//! it had to hand-roll became `Badge`, `Facts` and `DataTable`
//! [docs/HOMELAB_PROOF.md]. The second pass put those three in place.
//!
//! This is the third: Kenny picked **Card grid** out of five directions,
//! so the three panels are gone and every stack is a card of its own —
//! its state, its apps, its two meters and its last quarter hour. The
//! meters share one label column, because a bar that starts where the
//! word before it happens to end is the fault he found in the picture
//! [fix-64].
//!
//! The registry list is gone — the cards are the list. The manifest and
//! the app grid stayed: a direction may change the shape of a screen, it
//! may not quietly drop what the screen could do [fix-65].

use kp_tui::{
    Badge, Column, DataTable, Facts, Meter, Rail, Spark, Stage, Theme, Tone,
    fx::{self, Motion},
    source_colour,
    widgets::Panel,
};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Paragraph, Widget},
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
    /// The two readings the card carries, 0.0 to 1.0.
    pub cpu: f32,
    pub memory: f32,
    /// A quarter of an hour of load, one reading a minute, 0..100.
    pub load: [f64; 30],
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

        cpu: 0.22,
        memory: 0.64,
        load: [
            35.9, 38.3, 48.4, 45.1, 53.6, 53.5, 50.7, 55.8, 48.8, 51.2, 43.6, 40.0, 39.6, 40.0,
            27.2, 24.4, 25.9, 27.3, 21.3, 18.8, 26.5, 17.1, 29.6, 66.3, 68.7, 72.8, 79.6, 89.9,
            86.0, 93.8,
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

        cpu: 0.31,
        memory: 0.48,
        load: [
            38.8, 37.6, 40.5, 34.5, 33.1, 32.5, 34.9, 28.0, 22.3, 21.1, 15.1, 9.3, 11.9, 8.3, 1.3,
            4.9, 5.0, 11.0, 12.1, 10.3, 22.7, 16.8, 24.9, 33.2, 29.6, 36.7, 33.4, 41.9, 43.0, 39.5,
        ],
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

        cpu: 0.04,
        memory: 0.11,
        load: [
            26.9, 17.8, 19.1, 14.0, 9.5, 3.5, 3.8, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
            0.0, 7.4, 4.1, 13.6, 13.8, 16.2, 17.6, 27.1, 19.4, 19.6, 19.1, 21.7, 8.4, 8.5,
        ],
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

        cpu: 0.18,
        memory: 0.72,
        load: [
            33.1, 32.7, 27.5, 24.1, 13.7, 12.9, 10.7, 16.6, 18.2, 10.4, 13.4, 17.6, 21.8, 29.2,
            34.9, 35.3, 35.9, 43.8, 45.3, 48.7, 53.2, 48.9, 44.6, 42.7, 39.5, 27.8, 33.4, 27.6,
            24.7, 20.3,
        ],
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

        cpu: 0.09,
        memory: 0.26,
        load: [
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.8, 7.1, 8.0, 11.9, 17.9, 21.0, 27.2, 25.2, 36.4,
            33.2, 26.4, 25.4, 23.4, 19.8, 12.6, 16.8, 14.2, 3.8, 0.6, 0.0, 0.0, 0.0, 0.0,
        ],
    },
];

/// The two readings every card carries, in the order they are drawn. The
/// column they start their bars in is measured once, from the longest of
/// them, and every card uses it — so the bars line up down the screen and
/// not only inside one card [fix-64].
const READINGS: [&str; 2] = ["cpu", "memory"];

pub fn draw(frame: &mut Frame, th: &Theme, selected: usize, reveal_ms: u32, motion: Motion) {
    let screen = frame.area();
    let stage = Stage::new(reveal_ms, motion);
    let [rail, body, detail] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(16),
        Constraint::Min(8),
    ])
    .areas(screen);
    frame.render_widget(Rail::new(th).grown(stage.ground()), rail);

    // Three to a row, as wide as the screen allows; a card is seven rows
    // high, which is the two borders and the five it has to say.
    let cols = Layout::horizontal([Constraint::Ratio(1, 3); 3]).split(body);
    let label_w = kp_tui::label_column(th, &READINGS);
    for (i, stack) in FLEET.iter().enumerate() {
        let col = cols[i % 3];
        let card = Rect {
            y: col.y + (i / 3) as u16 * 8,
            height: 8.min(col.height),
            ..col
        };
        if card.y + card.height > body.y + body.height || card.height < 4 {
            break;
        }
        draw_card(
            frame,
            th,
            card,
            stack,
            i == selected,
            label_w,
            reveal_ms,
            motion,
        );
    }

    // What the card has no room for, for the stack in hand: the manifest
    // homelab puts on the right, and the apps it runs.
    draw_detail(
        frame,
        th,
        detail,
        &FLEET[selected.min(FLEET.len() - 1)],
        reveal_ms,
        motion,
    );
}

/// One stack, as a card: what it is, what it runs, what it costs and what
/// it has been doing.
#[allow(clippy::too_many_arguments)]
fn draw_card(
    frame: &mut Frame,
    th: &Theme,
    area: Rect,
    s: &Stack,
    focused: bool,
    label_w: usize,
    reveal_ms: u32,
    motion: Motion,
) {
    let stage = Stage::new(reveal_ms, motion);
    // The hostname arrives in the theme's own routine, where homelab
    // always decrypts it.
    let title = fx::frame(s.hostname, th.a.reveal, reveal_ms, motion).text;
    let panel = Panel::new(th, &title)
        .focused(focused)
        .stage(stage)
        .reveal(reveal_ms, motion);
    let inner = panel.block().inner(area);
    frame.render_widget(panel, area);
    if inner.height < 3 {
        return;
    }

    let rows = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(1),
    ])
    .split(inner);

    let mut head = vec![
        Span::styled("▎", Style::new().fg(source_colour(th, s.name))),
        Badge::dot(th, s.online),
        Span::styled(
            if s.online { " up  " } else { " down" }.to_string(),
            Style::new().fg(th.c.muted_foreground),
        ),
        Span::raw("  "),
    ];
    if s.drift {
        head.extend(Badge::new(th, "upd", Tone::Warning).spans());
    }
    if !s.enabled {
        head.extend(Badge::new(th, "off", Tone::MutedInk).spans());
    }
    let running = s.apps.iter().filter(|a| a.1).count();
    let body = vec![
        Line::from(head),
        Line::from(vec![
            Span::styled(
                format!("{running}/{} apps", s.apps.len()),
                Style::new().fg(th.c.card_foreground),
            ),
            Span::styled(
                format!("   vmid {}", s.vmid),
                Style::new().fg(th.c.muted_foreground),
            ),
        ]),
    ];
    for (n, line) in body.into_iter().enumerate() {
        Paragraph::new(line)
            .style(Style::new().fg(th.c.card_foreground).bg(th.c.card))
            .render(rows[n], frame.buffer_mut());
    }

    for (n, (label, value)) in READINGS.iter().zip([s.cpu, s.memory]).enumerate() {
        Meter::new(th, label, value)
            .label_width(label_w)
            .render(rows[2 + n], frame.buffer_mut());
    }

    // The last quarter hour under it all, four levels to the row.
    Spark::new(th, &s.load)
        .max(100.0)
        .warn_above(0.8)
        .render(rows[4], frame.buffer_mut());
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
    let panel = Panel::new(th, &title)
        .stage(Stage::new(reveal_ms, motion))
        .reveal(reveal_ms, motion);
    let inner = panel.block().inner(manifest);
    frame.render_widget(panel, manifest);

    let facts = [
        ("vmid", s.vmid.to_string(), Tone::Ink),
        (
            "env",
            if s.env_sealed {
                "sealed".into()
            } else {
                "missing — deploy fails closed".to_string()
            },
            if s.env_sealed {
                Tone::Success
            } else {
                Tone::Danger
            },
        ),
        (
            "drift",
            if s.drift {
                "intent differs from applied".into()
            } else {
                "none — intent is runtime".to_string()
            },
            if s.drift {
                Tone::Warning
            } else {
                Tone::Success
            },
        ),
        (
            "nightly",
            if s.enabled {
                "enabled".into()
            } else {
                "parked".to_string()
            },
            if s.enabled {
                Tone::Success
            } else {
                Tone::Warning
            },
        ),
        (
            "safety",
            "whitelist · hostname-guard · fail-closed".to_string(),
            Tone::Success,
        ),
    ];
    let [facts_area, spark_area] =
        Layout::horizontal([Constraint::Min(30), Constraint::Length(18)]).areas(inner);
    Facts::new(th, &facts)
        .columns(2)
        .render(facts_area, frame.buffer_mut());
    // The load of the last quarter hour, at four levels to the row.
    Spark::new(th, &s.load)
        .max(100.0)
        .warn_above(0.8)
        .render(spark_area, frame.buffer_mut());

    let title = format!("App grid · {} units", s.apps.len());
    let panel = Panel::new(th, &title).stage(Stage::new(reveal_ms, motion));
    let inner = panel.block().inner(grid);
    frame.render_widget(panel, grid);

    let columns = [
        Column::new("app", 16),
        Column::new("state", 9),
        Column::new("restarts", 10).right(),
    ];
    let rows: Vec<Vec<Line<'static>>> = s
        .apps
        .iter()
        .map(|(name, running, restarts)| {
            vec![
                Line::from(Span::styled(
                    (*name).to_string(),
                    Style::new().fg(th.c.card_foreground),
                )),
                Line::from(if *running {
                    Badge::new(th, "run", Tone::Success).spans()
                } else {
                    Badge::new(th, "down", Tone::Danger).plated(true).spans()
                }),
                Line::from(Span::styled(
                    restarts.to_string(),
                    Style::new().fg(if *restarts > 0 {
                        th.ink(Tone::Warning, th.id.palette().card)
                    } else {
                        th.c.muted_foreground
                    }),
                )),
            ]
        })
        .collect();
    DataTable::new(th, &columns, &rows).render(inner, frame.buffer_mut());

    // The sweep the register declares crosses the grid, where homelab's
    // crosses it always and in one hard-coded colour.
    kp_tui::Surface::new(th, th.id.palette().card)
        .at(reveal_ms, motion)
        .id(0xA9)
        .paint(inner, frame.buffer_mut());
}
