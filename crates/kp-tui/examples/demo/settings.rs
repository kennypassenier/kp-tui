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
//!
//! Kenny asked whether two of the five directions could go together —
//! **Grouped cards** and **Diff** — so this is both: the settings live in
//! three cards, and under them stands what the host runs today beside what
//! S would make of it. Every label is padded to one column, so no value
//! sits against the word in front of it [fix-64].

use kp_tui::{
    Badge, Choice, Field, KeyHints, Stage, Theme, Tone, fx::Motion, label_column, widgets::Panel,
};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
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

/// What the host is running right now, against which the edits are read.
/// A change shows in the diff; a row that matches shows as unchanged.
const ON_HOST: [(&str, &str); 5] = [
    ("nightly run", "04:00"),
    ("keep daily", "every 1d for 7 days"),
    ("keep fortnightly", "every 14d for 90 days"),
    ("keep monthly", "every 60d forever"),
    ("webhook", "off"),
];

const KEYS: [(&str, &str); 6] = [
    ("↑↓", "field"),
    ("←→", "value"),
    ("a", "add tier"),
    ("enter", "edit webhook"),
    ("S", "save"),
    ("q", "quit"),
];

/// Every label on the screen, so one column serves all three cards: a
/// value may not start where its own label happens to end [fix-64].
const LABELS: [&str; 6] = [
    "nightly run",
    "window",
    "daily",
    "fortnightly",
    "monthly",
    "on failure",
];

pub fn draw(frame: &mut Frame, th: &Theme, selected: usize, reveal_ms: u32, motion: Motion) {
    let screen = frame.area();
    let stage = Stage::new(reveal_ms, motion);
    let [cards, diff, footer] = Layout::vertical([
        Constraint::Length(8),
        Constraint::Min(6),
        Constraint::Length(1),
    ])
    .areas(screen);

    let column = label_column(th, &LABELS) + 2;
    let cols = Layout::horizontal([Constraint::Ratio(1, 3); 3]).split(cards);
    draw_schedule(
        frame, th, cols[0], selected, column, stage, reveal_ms, motion,
    );
    draw_retention(
        frame, th, cols[1], selected, column, stage, reveal_ms, motion,
    );
    draw_notify(
        frame, th, cols[2], selected, column, stage, reveal_ms, motion,
    );
    draw_diff(frame, th, diff, stage, reveal_ms, motion);

    Paragraph::new(KeyHints::new(th, &KEYS).footer())
        .style(Style::new().bg(th.c.background))
        .render(footer, frame.buffer_mut());
}

/// The rows of one card, drawn on the card's own plate.
fn put(frame: &mut Frame, th: &Theme, inner: Rect, lines: Vec<Line<'static>>) {
    let areas = Layout::vertical(vec![Constraint::Length(1); lines.len().max(1)]).split(inner);
    for (n, line) in lines.into_iter().enumerate() {
        Paragraph::new(line)
            .style(Style::new().fg(th.c.card_foreground).bg(th.c.card))
            .render(areas[n], frame.buffer_mut());
    }
}

fn help(th: &Theme, text: &str) -> Line<'static> {
    Line::from(Span::styled(
        text.to_string(),
        Style::new().fg(th.c.muted_foreground),
    ))
}

#[allow(clippy::too_many_arguments)]
fn draw_schedule(
    frame: &mut Frame,
    th: &Theme,
    area: Rect,
    selected: usize,
    column: usize,
    stage: Stage,
    reveal_ms: u32,
    motion: Motion,
) {
    let panel = Panel::new(th, "Schedule")
        .focused(selected == 0)
        .stage(stage)
        .reveal(reveal_ms, motion);
    let inner = panel.block().inner(area);
    frame.render_widget(panel, area);

    let hour = match BACKUP_HOUR {
        Some(h) => format!("{h:02}:00"),
        None => "off".into(),
    };
    let nightly = Choice::new(th, "nightly run", &hour).focused(selected == 0);
    let window = Choice::new(th, "window", "2 hours");
    put(
        frame,
        th,
        inner,
        vec![
            Line::from([nightly.label_spans(column), nightly.value_spans()].concat()),
            Line::from([window.label_spans(column), window.value_spans()].concat()),
            Line::default(),
            help(th, "backup and auto-updates"),
            help(th, "for every managed stack"),
            help(th, "at this hour"),
        ],
    );
}

#[allow(clippy::too_many_arguments)]
fn draw_retention(
    frame: &mut Frame,
    th: &Theme,
    area: Rect,
    selected: usize,
    column: usize,
    stage: Stage,
    reveal_ms: u32,
    motion: Motion,
) {
    let focused = (1..=TIERS.len() * 2).contains(&selected);
    let panel = Panel::new(th, "Retention")
        .focused(focused)
        .stage(stage)
        .reveal(reveal_ms, motion);
    let inner = panel.block().inner(area);
    frame.render_widget(panel, area);

    let names = ["daily", "fortnightly", "monthly"];
    let mut lines = Vec::new();
    for (i, tier) in TIERS.iter().enumerate() {
        // Short in the card, spelled out in the diff under it: a card
        // this narrow cannot carry both words and both steppers.
        //
        // Both values are padded to the width of the longest of their own
        // kind, so the second stepper starts in the same column on every
        // tier — a 14 may not push it one cell further than a 1 [fix-64].
        let every = format!("{:>3}", format!("{}d", tier.every_days));
        let span = format!(
            "{:<7}",
            match tier.span_days {
                Some(d) => format!("for {d}d"),
                None => "always".into(),
            }
        );
        let left = Choice::new(th, names[i], &every).focused(selected == 1 + i * 2);
        let right = Choice::new(th, "", &span).focused(selected == 2 + i * 2);
        lines.push(Line::from(
            [
                left.label_spans(column),
                left.value_spans(),
                vec![Span::raw("  ")],
                right.value_spans(),
            ]
            .concat(),
        ));
    }
    lines.push(Line::default());
    lines.push(help(th, "one snapshot per interval,"));
    lines.push(help(th, "kept for the span beside it"));
    put(frame, th, inner, lines);
}

#[allow(clippy::too_many_arguments)]
fn draw_notify(
    frame: &mut Frame,
    th: &Theme,
    area: Rect,
    selected: usize,
    column: usize,
    stage: Stage,
    reveal_ms: u32,
    motion: Motion,
) {
    let hook_row = 1 + TIERS.len() * 2;
    let panel = Panel::new(th, "Notify")
        .focused(selected == hook_row)
        .stage(stage)
        .reveal(reveal_ms, motion);
    let inner = panel.block().inner(area);
    frame.render_widget(panel, area);

    // The one row that is typed into rather than stepped through: the
    // theme's own caret, blinking at the rate its register states.
    let field = Field::new(th, "webhook", WEBHOOK.unwrap_or("off — enter to set"))
        .focused(selected == hook_row)
        .label_width(column)
        .blink(reveal_ms / 16);
    let on_failure = Choice::new(th, "on failure", "always");
    put(
        frame,
        th,
        inner,
        vec![
            Line::from(field.spans()),
            Line::from([on_failure.label_spans(column), on_failure.value_spans()].concat()),
            Line::default(),
            help(th, "one POST per finished"),
            help(th, "operation, {op, ok, error}"),
        ],
    );
}

/// What the host runs today beside what saving would make of it: the
/// second half of Kenny's question, and the reason the dirty dot can say
/// how many rows it stands for.
fn draw_diff(
    frame: &mut Frame,
    th: &Theme,
    area: Rect,
    stage: Stage,
    reveal_ms: u32,
    motion: Motion,
) {
    let wanted = [
        (
            "nightly run",
            match BACKUP_HOUR {
                Some(h) => format!("{h:02}:00"),
                None => "off".into(),
            },
        ),
        ("keep daily", tier_words(0)),
        ("keep fortnightly", tier_words(1)),
        ("keep monthly", tier_words(2)),
        ("webhook", WEBHOOK.unwrap_or("off").to_string()),
    ];
    let changed = wanted
        .iter()
        .zip(ON_HOST)
        .filter(|((_, w), (_, h))| w != h)
        .count();
    let title = match changed {
        0 => "In sync with the host".to_string(),
        1 => "1 unsaved change".to_string(),
        n => format!("{n} unsaved changes"),
    };
    let panel = Panel::new(th, &title)
        .stage(stage)
        .reveal(reveal_ms, motion);
    let inner = panel.block().inner(area);
    frame.render_widget(panel, area);

    // Three columns at fixed starts: the setting, the host, the edit.
    let names: Vec<&str> = ON_HOST.iter().map(|(n, _)| *n).collect();
    let name_w = label_column(th, &names) + 2;
    let host_w = ON_HOST
        .iter()
        .map(|(_, v)| v.chars().count())
        .max()
        .unwrap_or(0)
        + 2;
    let mut lines = vec![Line::from(vec![
        Span::styled(
            format!("{:<name_w$}", ""),
            Style::new().fg(th.c.muted_foreground),
        ),
        Span::styled(
            format!("{:<host_w$}", "on the host"),
            Style::new().fg(th.c.muted_foreground),
        ),
        Span::styled(
            "after S".to_string(),
            Style::new().fg(th.c.muted_foreground),
        ),
    ])];
    for ((name, host), (_, want)) in ON_HOST.iter().zip(wanted.iter()) {
        let differs = host != want;
        lines.push(Line::from(vec![
            Span::styled(
                format!("{name:<name_w$}"),
                Style::new().fg(th.c.card_foreground),
            ),
            Span::styled(
                format!("{host:<host_w$}"),
                Style::new().fg(th.c.muted_foreground),
            ),
            Span::styled(
                if differs {
                    want.clone()
                } else {
                    "—".to_string()
                },
                if differs {
                    Style::new()
                        .fg(th.ink(Tone::Warning, th.id.palette().card))
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::new().fg(th.c.muted_foreground)
                },
            ),
        ]));
    }
    lines.push(Line::default());

    // The dirty marker: a dot in the tone the state deserves, and the word
    // beside it in an ink that reads on this surface.
    let (tone, says) = if DIRTY && changed > 0 {
        (Tone::Warning, "S applies them; R reloads the host's values")
    } else {
        (Tone::Success, "nothing to apply")
    };
    lines.push(Line::from(vec![
        Badge::state(th, tone),
        Span::raw(" "),
        Span::styled(
            says.to_string(),
            Style::new().fg(th.ink(tone, th.id.palette().card)),
        ),
    ]));
    put(frame, th, inner, lines);
}

fn tier_words(i: usize) -> String {
    let t = &TIERS[i];
    match t.span_days {
        Some(d) => format!("every {}d for {d} days", t.every_days),
        None => format!("every {}d forever", t.every_days),
    }
}
