//! `--shot`: the demo's own screens, drawn headless and printed as ANSI.
//!
//! No terminal is opened, so this runs in a pipe, in a test and over ssh.
//! It goes through `App::draw` like the running demo, which is the point:
//! a picture that came from somewhere else would drift from the screen it
//! claims to show.

use std::io::{self, Write};

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use kp_tui::{ColorDepth, Theme, ThemeId, live::Sample};
use ratatui::{Terminal, backend::TestBackend, style::Color};

use crate::app::App;

fn ansi(c: Color, fg: bool) -> String {
    match c {
        Color::Rgb(r, g, b) => format!("\x1b[{};2;{r};{g};{b}m", if fg { 38 } else { 48 }),
        Color::Indexed(i) => format!("\x1b[{};5;{i}m", if fg { 38 } else { 48 }),
        _ => String::new(),
    }
}

/// One reading, so a ticker and the dashboard have something to show
/// without a running sampler.
fn fixture() -> Sample {
    Sample {
        cpu_total: 23.0,
        cores: vec![31.0, 18.0, 22.0, 21.0],
        mem_used_pct: 61.0,
        mem_used_bytes: 9_878_000_000,
        mem_total_bytes: 16_000_000_000,
        load: [0.84, 0.71, 0.66],
        rx_bps: 412_000.0,
        tx_bps: 96_000.0,
        disk_read_bps: 1_200_000.0,
        disk_write_bps: 310_000.0,
        self_cpu_pct: 1.4,
        child_cpu_pct: 0.6,
    }
}

/// `--keys pst` presses p, s, t before the frame is drawn, so a shot can
/// show a screen that only exists after a key (the palette is one).
fn press(app: &mut App, keys: &str) {
    for c in keys.chars() {
        let code = match c {
            '↓' => KeyCode::Down,
            '↑' => KeyCode::Up,
            '⏎' => KeyCode::Enter,
            c => KeyCode::Char(c),
        };
        app.key(KeyEvent {
            code,
            modifiers: crossterm::event::KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        });
    }
}

pub fn print(
    app: &mut App,
    themes: &str,
    (w, h): (u16, u16),
    at: u32,
    keys: &str,
) -> io::Result<()> {
    let mut out = io::stdout().lock();
    app.dash.push_sample(0.0, fixture());
    app.dash.draw_ms = 0.42;
    for name in themes.split(',').filter(|n| !n.is_empty()) {
        let id = ThemeId::from_name(name)
            .ok_or_else(|| io::Error::other(format!("no theme called {name}")))?;
        app.theme = Theme::new(id, app.depth);
        app.config.theme = id;
        app.reveal_ms = 0;
        app.alarm_ms = 0;
        app.tick(at);
        press(app, keys);
        let Ok(mut terminal) = Terminal::new(TestBackend::new(w, h));
        let Ok(_) = terminal.draw(|f| app.draw(f));
        let buf = terminal.backend().buffer().clone();
        writeln!(out, "\x1b[0m{name}")?;
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                let cell = &buf[(x, y)];
                write!(
                    out,
                    "{}{}{}",
                    ansi(cell.bg, false),
                    ansi(cell.fg, true),
                    cell.symbol()
                )?;
            }
            writeln!(out, "\x1b[0m")?;
        }
        writeln!(out)?;
    }
    if app.depth == ColorDepth::Ansi16 {
        writeln!(out, "(sixteen colours: no texture, by design)")?;
    }
    out.flush()
}
