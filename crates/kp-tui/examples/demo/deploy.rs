//! homelab's deploy focus window, rebuilt on this crate — the fifth proof.
//!
//! The original is `client/src/tui/view/focus.rs`, 242 lines: a near
//! fullscreen takeover over whatever tab you were on, with this deploy's
//! own transcript, a question drawn over that transcript when a step needs
//! a decision, the transfer visuals, a gauge and a footer.
//!
//! It is the only homelab screen that is an overlay, which is why it was
//! picked: the four before it all owned their whole area.
//!
//! It found one gap on the way: a bar with a sentence written across it
//! rather than a percentage beside it. That is `Meter::across` now, so
//! nothing here is marked `GAP` and no colour on it is chosen by hand.
//!
//! Kenny picked **Two panes** out of five directions: the steps on the
//! left, the transcript on the right, both moving at once. The transcript
//! keeps every line it had — the direction adds the answer to "where is
//! it now", it does not take the output away [fix-65]. The steps' own
//! columns are fixed, so a longer name cannot push the timing sideways
//! [fix-64].

use kp_tui::{
    Badge, Glitch, KeyHints, Meter, Popup, PopupKind, Stream, Theme, Tone, fx::Motion, spinner,
};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

pub struct Step {
    pub text: &'static str,
    pub tone: Tone,
}

pub const TRANSCRIPT: [Step; 9] = [
    Step {
        text: "[sync][run ] rsync -a --delete ./stacks/media/ pve-01:/srv/media/",
        tone: Tone::Info,
    },
    Step {
        text: "[sync][exit] 0 in 4.2 s, 118 files, 41.9 MB",
        tone: Tone::Success,
    },
    Step {
        text: "[gate] compose config — valid",
        tone: Tone::Success,
    },
    Step {
        text: "[gate] image digests pinned — 6 of 6",
        tone: Tone::Success,
    },
    Step {
        text: "[run ] docker compose up -d",
        tone: Tone::Info,
    },
    Step {
        text: "jellyfin recreated",
        tone: Tone::MutedInk,
    },
    Step {
        text: "jellyfin-exporter recreated",
        tone: Tone::MutedInk,
    },
    Step {
        text: "[warn] jellyfin: health check not ready after 30 s, retrying",
        tone: Tone::Warning,
    },
    Step {
        text: "[run ] waiting for health…",
        tone: Tone::Info,
    },
];

/// The question a step raises, drawn over the transcript rather than beside
/// it: the transcript is exactly what the operator is reading, and a
/// question elsewhere is a question missed.
pub struct Ask {
    pub op: &'static str,
    pub step: &'static str,
    pub what: &'static str,
    pub if_allowed: &'static str,
    pub if_stopped: &'static str,
}

pub const ASK: Ask = Ask {
    op: "deploy media",
    step: "prune",
    what: "Remove 3 images no stack refers to any more?",
    if_allowed: "frees 2.1 GB; a rollback re-pulls them",
    if_stopped: "the deploy finishes, the images stay",
};

/// The steps this deploy walks, and where it stands in them. The
/// transcript is what each step said; this is what they are.
pub struct Phase {
    pub name: &'static str,
    pub state: Tone,
    pub says: &'static str,
}

pub const PHASES: [Phase; 6] = [
    Phase {
        name: "sync",
        state: Tone::Success,
        says: "4.2 s · 118 files",
    },
    Phase {
        name: "gates",
        state: Tone::Success,
        says: "2 of 2 passed",
    },
    Phase {
        name: "pull",
        state: Tone::Success,
        says: "6 digests pinned",
    },
    Phase {
        name: "up",
        state: Tone::Info,
        says: "recreating 2 of 6",
    },
    Phase {
        name: "health",
        state: Tone::Warning,
        says: "retrying, 30 s",
    },
    Phase {
        name: "prune",
        state: Tone::MutedInk,
        says: "waiting on an answer",
    },
];

const KEYS: [(&str, &str); 3] = [
    ("↑↓", "scroll"),
    ("a / s", "answer"),
    ("esc", "background — the deploy keeps running"),
];

pub fn draw(frame: &mut Frame, th: &Theme, asking: bool, reveal_ms: u32, motion: Motion) {
    let screen = frame.area();

    // The title glitches where a register declares it, which is the same
    // routine homelab runs over this title by hand.
    let title = "Deploy media — live";
    let glitched =
        th.a.fx
            .alarm
            .glitch
            .map(|g: Glitch| g.text(title, reveal_ms, 0x30DA1, motion))
            .unwrap_or_else(|| title.to_string());

    let popup = Popup::new(
        th,
        &glitched,
        (
            screen.width.saturating_sub(8),
            screen.height.saturating_sub(4),
        ),
    );
    let inner = popup.render_over(screen, frame.buffer_mut());
    let [panes, transfer, gauge, footer] = Layout::vertical([
        Constraint::Min(4),
        Constraint::Length(2),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(inner);

    // Two panes: where the deploy stands, and what it has been saying.
    let [steps, feed] =
        Layout::horizontal([Constraint::Length(34), Constraint::Min(30)]).areas(panes);
    draw_steps(frame, th, steps);
    draw_feed(frame, th, feed);
    if asking {
        draw_ask(frame, th, feed);
    }

    // The transfer this deploy is pushing: its name and what has gone over,
    // then the stream itself, which nobody can put a fraction on.
    let [label, flow] = Layout::vertical([Constraint::Length(1); 2]).areas(transfer);
    Paragraph::new(Line::from(vec![
        Span::styled("⇅ ", Style::new().fg(th.c.primary)),
        Span::styled(
            "media/jellyfin.tar".to_string(),
            Style::new().fg(th.c.popover_foreground),
        ),
        Span::styled(
            "  412 MB".to_string(),
            Style::new().fg(th.c.muted_foreground),
        ),
    ]))
    .style(Style::new().bg(th.c.popover))
    .render(label, frame.buffer_mut());
    Stream::new(th, reveal_ms, motion).render(flow, frame.buffer_mut());

    // The gauge, with the sentence written across it rather than a
    // percentage beside it: on one row there is no space for both, and
    // what an operator wants here is what is happening.
    Meter::new(th, "deploy", 0.62)
        .thresholds(2.0, 2.0)
        .across("streaming over TLS…")
        .render(gauge, frame.buffer_mut());

    Paragraph::new(Line::from(
        [
            vec![Span::styled(
                format!("{} ", spinner(th, reveal_ms, motion)),
                Style::new().fg(th.c.primary),
            )],
            KeyHints::new(th, &KEYS).footer().spans,
        ]
        .concat(),
    ))
    .style(Style::new().bg(th.c.popover))
    .render(footer, frame.buffer_mut());
}

/// The left pane: the steps, their state and what each one has to say
/// for itself — all three in columns that do not move.
fn draw_steps(frame: &mut Frame, th: &Theme, area: Rect) {
    let names: Vec<&str> = PHASES.iter().map(|p| p.name).collect();
    let column = kp_tui::label_column(th, &names) + 2;
    let mut lines = vec![Line::from(Span::styled(
        "  where it stands".to_string(),
        Style::new().fg(th.c.muted_foreground),
    ))];
    for phase in PHASES.iter() {
        let ink = th.ink(phase.state, th.id.palette().popover);
        lines.push(Line::from(vec![
            Span::raw("  "),
            Badge::state(th, phase.state),
            Span::raw(" "),
            Span::styled(
                format!("{:<column$}", phase.name),
                Style::new().fg(ink).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                phase.says.to_string(),
                Style::new().fg(th.c.muted_foreground),
            ),
        ]));
    }
    Paragraph::new(lines)
        .style(Style::new().bg(th.c.popover))
        .render(area, frame.buffer_mut());
}

/// The right pane: the transcript, one line per step, each in the ink its
/// tone deserves on the surface it lands on.
fn draw_feed(frame: &mut Frame, th: &Theme, area: Rect) {
    let lines: Vec<Line<'static>> = TRANSCRIPT
        .iter()
        .map(|step| {
            Line::from(Span::styled(
                format!("  {}", step.text),
                Style::new().fg(th.ink(step.tone, th.id.palette().popover)),
            ))
        })
        .collect();
    Paragraph::new(lines)
        .style(Style::new().bg(th.c.popover))
        .render(area, frame.buffer_mut());
}

/// The question, over the foot of the transcript: what it asks, and what
/// each answer does — not only the two words.
fn draw_ask(frame: &mut Frame, th: &Theme, over: Rect) {
    let h = 9.min(over.height);
    let box_rect = Rect {
        y: over.y + over.height.saturating_sub(h),
        height: h,
        ..over
    };
    let heading = format!("{} · {}", ASK.op, ASK.step);
    let popup = Popup::new(th, &heading, (box_rect.width, h)).kind(PopupKind::Danger);
    let inner = popup.render_over(box_rect, frame.buffer_mut());
    // Both answers start their explanation in the same column, so the
    // two consequences can be read against each other [fix-64].
    let column = kp_tui::label_column(th, &["toelaten", "stoppen"]) + 2;
    let answer = |key: &'static str, word: &'static str, tone: Tone, what: &'static str| {
        Line::from(vec![
            Span::styled(
                format!("  {key} {:<column$}", word),
                Style::new()
                    .fg(th.ink(tone, th.id.palette().popover))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(what.to_string(), Style::new().fg(th.c.muted_foreground)),
        ])
    };
    Paragraph::new(vec![
        Line::from(Span::styled(
            format!("  {}", ASK.what),
            Style::new()
                .fg(th.c.popover_foreground)
                .add_modifier(Modifier::BOLD),
        )),
        Line::default(),
        answer("a", "toelaten", Tone::Success, ASK.if_allowed),
        answer("s", "stoppen", Tone::Danger, ASK.if_stopped),
        Line::default(),
        Line::from(Span::styled(
            "  no answer means unattended; the step does not go ahead".to_string(),
            Style::new().fg(th.c.muted_foreground),
        )),
    ])
    .style(Style::new().bg(th.c.popover))
    .render(inner, frame.buffer_mut());
}
