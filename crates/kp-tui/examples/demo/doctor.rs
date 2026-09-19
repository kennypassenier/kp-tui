//! homelab's doctor tab, rebuilt on this crate — the sixth proof.
//!
//! The original is `client/src/tui/view/doctor.rs`, 59 lines: the host's
//! own self-diagnosis, read back as text and coloured by what each line
//! says — `[Ok]`, `[Warn]`, `[Fail]`, and an indented `↳` for the detail
//! under a finding. It has an empty state too, with a spinner, because
//! the checks take a moment to run.
//!
//! It is the smallest of homelab's nine screens, and it was left for last
//! with the splash. What it asks of the crate is the one thing the five
//! before it never did: read a tone out of a sentence, and then find an
//! ink for that tone on the plate the line lands on.

use kp_tui::{
    Badge, KeyHints, Stage, Theme, Tone,
    fx::{self, Motion},
    spinner,
    widgets::Panel,
};
use ratatui::{
    Frame,
    layout::{Constraint, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

/// What the host answered, in the shape homelab's own doctor prints it:
/// a heading, then findings, then the detail under a finding that failed.
pub const REPORT: [&str; 14] = [
    "host",
    "  [Ok]   kernel 6.11.4, 19 days up",
    "  [Ok]   docker 27.3.1, 12 containers",
    "  [Warn] 61% of the root pool is used",
    "    ↳ 148 GB free of 380 GB; the nightly snapshot needs 12 GB",
    "network",
    "  [Ok]   TLS pinned, 9F:2C:A1:0E:77:B4",
    "  [Ok]   every stack answers on its own port",
    "stacks",
    "  [Fail] backup: env file missing",
    "    ↳ deploy fails closed; restic cannot read its repository password",
    "  [Warn] media: intent differs from applied",
    "    ↳ 1 image is newer than the manifest pins",
    "  [Ok]   the other three match their manifest",
];

const KEYS: [(&str, &str); 3] = [
    ("r", "run the checks again"),
    ("enter", "the same"),
    ("q", "quit"),
];

/// The tone a line of the report deserves, read out of what it says. This
/// is the whole of homelab's own rule, kept: a marker wins, an indented
/// detail is the note under it, and anything else is a heading.
pub fn tone_of(line: &str) -> Tone {
    if line.contains("[Fail]") {
        Tone::Danger
    } else if line.contains("[Warn]") {
        Tone::Warning
    } else if line.contains("[Ok]") {
        Tone::Success
    } else if line.trim_start().starts_with('↳') {
        Tone::MutedInk
    } else {
        Tone::Ink
    }
}

/// What a tone is called in the report, four cells wide.
fn word(tone: Tone) -> &'static str {
    match tone {
        Tone::Danger => "fail",
        Tone::Warning => "warn",
        _ => "ok",
    }
}

/// The sentence without the marker the tone was read out of: the dot and
/// the word carry it now, and writing it twice reads as a stutter.
fn strip(line: &str) -> String {
    let t = line.trim_start();
    for marker in ["[Ok]", "[Warn]", "[Fail]"] {
        if let Some(rest) = t.strip_prefix(marker) {
            return rest.trim_start().to_string();
        }
    }
    t.to_string()
}

pub fn draw(frame: &mut Frame, th: &Theme, running: bool, reveal_ms: u32, motion: Motion) {
    let screen = frame.area();
    let stage = Stage::new(reveal_ms, motion);
    let [body, footer] =
        Layout::vertical([Constraint::Min(6), Constraint::Length(1)]).areas(screen);

    let panel = Panel::new(th, "Self diagnosis")
        .focused(true)
        .stage(stage)
        .reveal(reveal_ms, motion);
    let inner = panel.block().inner(body);
    frame.render_widget(panel, body);

    let lines: Vec<Line<'static>> = if running {
        vec![
            Line::from(vec![
                Span::styled(
                    format!("{} ", spinner(th, reveal_ms, motion)),
                    Style::new().fg(th.c.primary),
                ),
                Span::styled(
                    "running checks…".to_string(),
                    Style::new().fg(th.c.muted_foreground),
                ),
            ]),
            Line::from(Span::styled(
                "press r or enter to run them again".to_string(),
                Style::new().fg(th.c.border_strong),
            )),
        ]
    } else {
        REPORT
            .iter()
            .enumerate()
            .map(|(i, raw)| {
                let tone = tone_of(raw);
                // Every line arrives in the register's own routine, the
                // way homelab decrypts this report as it lands.
                let text = fx::frame(raw, th.a.reveal, reveal_ms + i as u32 * 40, motion).text;
                let mut style = Style::new().fg(th.ink(tone, th.id.palette().card));
                if matches!(tone, Tone::Ink) {
                    style = style.add_modifier(Modifier::BOLD);
                }
                // A finding wears the state dot its tone deserves and
                // the word that tone is called, in a column four cells
                // wide so the sentences all begin in the same place
                // [fix-64]. The word stays, because a report read with no
                // colour at all must still say which line failed [fix-1].
                match tone {
                    Tone::Success | Tone::Warning | Tone::Danger => Line::from(vec![
                        Span::raw(" "),
                        Badge::state(th, tone),
                        Span::styled(format!(" {:<4} ", word(tone)), style),
                        Span::styled(strip(&text), style),
                    ]),
                    _ => Line::from(Span::styled(format!("  {text}"), style)),
                }
            })
            .collect()
    };
    Paragraph::new(lines)
        .style(Style::new().fg(th.c.card_foreground).bg(th.c.card))
        .render(inner, frame.buffer_mut());

    Paragraph::new(KeyHints::new(th, &KEYS).footer())
        .style(Style::new().bg(th.c.background))
        .render(footer, frame.buffer_mut());
}
