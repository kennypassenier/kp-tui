//! homelab's settings tab, rebuilt on this crate — the third proof.
//!
//! The original is `client/src/tui/view/settings.rs`, 116 lines: the host's
//! runtime configuration, edited over the TLS line. It was chosen because it
//! leans on nothing the first two screens leaned on — no table, no meter, no
//! panel grid — but on fields, a row you step through, help text under every
//! setting, an inline text edit and a dirty marker.
//!
//! It found two gaps on the way [docs/HOMELAB_PROOF.md]: a value that is
//! stepped through rather than typed, and a state dot that only knew up
//! from down. Both are in the crate now, so nothing here is marked `GAP`
//! and no colour on this screen is chosen by hand.

use kp_tui::{Badge, Choice, Field, KeyHints, Stage, Theme, Tone, fx::Motion, widgets::Panel};
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::Style,
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

pub struct Tier {
    pub every_days: u32,
    pub span_days: Option<u32>,
}

pub const TIERS: [Tier; 3] = [
    Tier {
        every_days: 1,
        span_days: Some(7),
    },
    Tier {
        every_days: 14,
        span_days: Some(60),
    },
    Tier {
        every_days: 60,
        span_days: None,
    },
];

pub const BACKUP_HOUR: Option<u32> = Some(3);
pub const WEBHOOK: Option<&str> = None;
pub const DIRTY: bool = true;

/// The rows the arrow keys walk: the hour, then two per tier, then the hook.
pub const ROWS: usize = 1 + TIERS.len() * 2 + 1;

const KEYS: [(&str, &str); 6] = [
    ("↑↓", "field"),
    ("←→", "value"),
    ("a", "add tier"),
    ("enter", "edit webhook"),
    ("S", "save"),
    ("q", "quit"),
];

pub fn draw(frame: &mut Frame, th: &Theme, selected: usize, reveal_ms: u32, motion: Motion) {
    let screen = frame.area();
    let stage = Stage::new(reveal_ms, motion);
    let [body, footer] =
        Layout::vertical([Constraint::Min(10), Constraint::Length(1)]).areas(screen);

    let panel = Panel::new(th, "Host settings")
        .focused(true)
        .stage(stage)
        .reveal(reveal_ms, motion);
    let inner = panel.block().inner(body);
    frame.render_widget(panel, body);

    // The rows in the order the screen reads, so the row the arrow keys
    // point at and the row drawn are the same index by construction.
    let hook_row = 1 + TIERS.len() * 2;
    let mut rows: Vec<Line<'static>> = Vec::new();
    let help = |text: &str| {
        Line::from(Span::styled(
            text.to_string(),
            Style::new().fg(th.c.muted_foreground),
        ))
    };

    rows.push(help(
        "Changes apply live on the host after S — written to host.toml.",
    ));
    rows.push(Line::default());

    // Row 0 · the nightly run: a label, a value that steps, its explanation.
    let hour = match BACKUP_HOUR {
        Some(h) => format!("{h:02}:00"),
        None => "off".into(),
    };
    let nightly = Choice::new(th, "nightly run", &hour).focused(selected == 0);
    rows.push(Line::from(
        [
            nightly.label_spans(14),
            nightly.value_spans(),
            vec![Span::styled(
                "  backup and auto-updates for every managed stack at this hour".to_string(),
                Style::new().fg(th.c.muted_foreground),
            )],
        ]
        .concat(),
    ));
    rows.push(Line::default());

    // The retention tiers, two stepped values to the row.
    rows.push(help(
        "Retention — newest first; within each window one snapshot per interval is kept:",
    ));
    for (i, tier) in TIERS.iter().enumerate() {
        let every = format!("every {}d", tier.every_days);
        let span = match tier.span_days {
            Some(d) => format!("for {d} days"),
            None => "forever".into(),
        };
        rows.push(Line::from(
            [
                vec![Span::styled(
                    format!("  tier {}   ", i + 1),
                    Style::new().fg(th.c.muted_foreground),
                )],
                Choice::new(th, "", &every)
                    .focused(selected == 1 + i * 2)
                    .value_spans(),
                vec![Span::raw("  ")],
                Choice::new(th, "", &span)
                    .focused(selected == 2 + i * 2)
                    .value_spans(),
            ]
            .concat(),
        ));
    }
    rows.push(help(
        "  1d/7d then 14d/60d then 60d/forever is a daily week, a fortnightly two months, then bimonthly",
    ));
    rows.push(Line::default());

    // The webhook, the one row that is typed into rather than stepped
    // through: the theme's own caret, blinking at the rate its register
    // states.
    rows.push(Line::from(
        Field::new(th, "webhook", WEBHOOK.unwrap_or("off — enter to set"))
            .focused(selected == hook_row)
            .blink(reveal_ms / 16)
            .spans(),
    ));
    rows.push(help(
        "  one POST per finished operation, {op, ok, error} — point it at a Home Assistant webhook",
    ));
    rows.push(Line::default());

    // The dirty marker: a dot in the tone the state deserves, and the word
    // beside it in an ink that reads on this surface.
    let (tone, says) = if DIRTY {
        (Tone::Warning, "unsaved changes — S to apply")
    } else {
        (Tone::Success, "in sync with host")
    };
    rows.push(Line::from(vec![
        Badge::state(th, tone),
        Span::raw(" "),
        Span::styled(
            says.to_string(),
            Style::new().fg(th.ink(tone, th.id.palette().card)),
        ),
    ]));

    let areas = Layout::vertical(vec![Constraint::Length(1); rows.len()]).split(inner);
    for (n, line) in rows.into_iter().enumerate() {
        Paragraph::new(line)
            .style(Style::new().bg(th.c.card))
            .render(areas[n], frame.buffer_mut());
    }

    Paragraph::new(KeyHints::new(th, &KEYS).footer())
        .style(Style::new().bg(th.c.background))
        .render(footer, frame.buffer_mut());
}
