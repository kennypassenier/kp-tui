//! Prints the console screen as ANSI, one theme per argument (default:
//! cyberpunk and terminal). For looking at it without a terminal session:
//! `cargo run --example console-shot -- cyberpunk terminal > out.ansi`.

use std::io::Write;

use kp_tui::{
    ColorDepth, Field, KeyHints, Meter, Popup, PopupKind, Theme, ThemeId, widgets::Panel,
};
use ratatui::{
    Terminal,
    backend::TestBackend,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph},
};

const KEYS: &[(&str, &str)] = &[
    ("s", "screen"),
    ("t", "theme"),
    ("m", "motion"),
    ("Esc", "close"),
    ("q", "quit"),
];

fn ansi(c: Color, fg: bool) -> String {
    match c {
        Color::Rgb(r, g, b) => format!("\x1b[{};2;{r};{g};{b}m", if fg { 38 } else { 48 }),
        _ => String::new(),
    }
}

fn main() {
    let names: Vec<String> = std::env::args().skip(1).collect();
    let names = if names.is_empty() {
        vec!["cyberpunk".into(), "terminal".into()]
    } else {
        names
    };
    let mut out = std::io::stdout().lock();
    for name in names {
        let id = ThemeId::from_name(&name).unwrap_or_else(|| panic!("no theme {name}"));
        let th = Theme::new(id, ColorDepth::TrueColor);
        let mut terminal = Terminal::new(TestBackend::new(96, 26)).expect("backend");
        terminal
            .draw(|f| {
                let screen = f.area();
                f.render_widget(Block::new().style(th.base()), screen);
                let rows = Layout::vertical([
                    Constraint::Length(1),
                    Constraint::Length(5),
                    Constraint::Length(4),
                    Constraint::Min(0),
                    Constraint::Length(1),
                ])
                .areas::<5>(screen);
                f.render_widget(
                    Paragraph::new(Line::from(vec![
                        Span::styled(
                            th.label("console"),
                            Style::new().fg(th.c.primary).add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            format!("  {} · what homelab hand-rolls, from the crate", id.name()),
                            Style::new().fg(th.c.muted_foreground),
                        ),
                    ]))
                    .style(Style::new().bg(th.c.background)),
                    rows[0],
                );
                let panel = Panel::new(&th, "Capacity");
                let inner = panel.block().inner(rows[1]);
                f.render_widget(panel, rows[1]);
                let meters = Layout::vertical([Constraint::Length(1); 3]).areas::<3>(inner);
                for (area, (label, value)) in
                    meters
                        .iter()
                        .zip([("ram", 0.42_f32), ("ssd", 0.78), ("load", 0.94)])
                {
                    f.render_widget(Meter::new(&th, label, value), *area);
                }
                let panel = Panel::new(&th, "Filter").focused(true);
                let inner = panel.block().inner(rows[2]);
                f.render_widget(panel, rows[2]);
                let fields = Layout::vertical([Constraint::Length(1); 2]).areas::<2>(inner);
                f.render_widget(Field::new(&th, "stack", "media"), fields[0]);
                f.render_widget(Field::new(&th, "filter", "web").focused(true), fields[1]);
                let panel = Panel::new(&th, "Keys");
                let inner = panel.block().inner(rows[3]);
                f.render_widget(panel, rows[3]);
                f.render_widget(
                    Paragraph::new(KeyHints::new(&th, KEYS).overlay())
                        .style(Style::new().bg(th.c.card)),
                    inner,
                );
                f.render_widget(KeyHints::new(&th, KEYS), rows[4]);
                let popup = Popup::new(&th, "Restore backup", (52, 7)).kind(PopupKind::Danger);
                let inner = popup.render_over(screen, f.buffer_mut());
                let lines = vec![
                    Line::from(Span::styled(
                        "This replaces the running stack with 2026-09-16.",
                        Style::new().fg(th.c.popover_foreground),
                    )),
                    Line::from(""),
                    Line::from(Span::styled(
                        "Type the stack's name to confirm:",
                        Style::new().fg(th.c.muted_foreground),
                    )),
                ];
                f.render_widget(
                    Paragraph::new(lines).style(Style::new().bg(th.c.popover)),
                    inner,
                );
                let field = Rect {
                    y: inner.bottom() - 1,
                    height: 1,
                    ..inner
                };
                f.render_widget(Field::new(&th, "name", "medi").focused(true), field);
            })
            .expect("draw");
        let buf = terminal.backend().buffer().clone();
        writeln!(out, "\n{name} (truecolor)").unwrap();
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                let cell = &buf[(x, y)];
                let mut style = String::from("\x1b[0m");
                style.push_str(&ansi(cell.fg, true));
                style.push_str(&ansi(cell.bg, false));
                if cell.modifier.contains(Modifier::BOLD) {
                    style.push_str("\x1b[1m");
                }
                write!(out, "{style}{}", cell.symbol()).unwrap();
            }
            writeln!(out, "\x1b[0m").unwrap();
        }
    }
}
