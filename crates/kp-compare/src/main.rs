//! homelab's own screens beside the kp-tui rebuilds of them.
//!
//! Kenny asked for the comparison and then asked why it was written in
//! Python: he never asked for Python, and it was a speed decision rather
//! than a judgement (2026-09-19). This is the same tool in the language the
//! project is written in, and it needs nothing installed.
//!
//! Both columns are read by the same terminal emulator at the same size, so
//! what differs on the page differs on a real terminal. The left column is
//! `homelab tui --offline` driven in a pseudo-terminal; the right is the
//! demo's one-frame shot of the same screen, replayed through the same
//! emulator. The data differs — homelab runs on its own fake host — and the
//! demo draws no tab bar, so the rows of chrome on the left are screen on
//! the right.
//!
//!     cargo run -p kp-compare -- --out /tmp/homelab-vs-kp-tui.html
//!
//! `--theme` picks the theme of the right column, `--size WxH` the terminal
//! both are read at, and `--homelab` the binary to drive.

use std::{
    io::{Read, Write},
    process::Command,
    time::{Duration, Instant},
};

use portable_pty::{CommandBuilder, PtySize, native_pty_system};

/// One row of the page: the screen homelab calls it, the keys that reach
/// that tab, the demo's name for the rebuild, and what to say about it.
struct Pair {
    keys: &'static str,
    screen: &'static str,
    caption: &'static str,
}

const PAIRS: [Pair; 5] = [
    Pair {
        keys: "\t",
        screen: "fleet",
        caption: "De stacks: het scherm waarop Homelab opent",
    },
    Pair {
        keys: "",
        screen: "ops",
        caption: "Het dashboard: host, capaciteit, containers, overdrachten",
    },
    Pair {
        keys: "\t\t\t\t",
        screen: "settings",
        caption: "De instellingen: velden en waardes die je doorstapt",
    },
    Pair {
        keys: "\t\t",
        screen: "logs",
        caption: "Het logboek: de bron-kiezer en de stroom regels",
    },
    Pair {
        keys: "\tD",
        screen: "deploy",
        caption: "Het deploy-venster: het enige scherm dat over alles heen ligt",
    },
];

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let opt = |name: &str, fallback: &str| -> String {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1))
            .cloned()
            .unwrap_or_else(|| fallback.to_string())
    };
    let theme = opt("--theme", "cyberpunk");
    let homelab = opt("--homelab", "homelab");
    let out = opt("--out", "homelab-vs-kp-tui.html");
    let size = opt("--size", "104x28");
    let (cols, rows) = size
        .split_once('x')
        .and_then(|(w, h)| Some((w.parse().ok()?, h.parse().ok()?)))
        .unwrap_or((104u16, 28u16));

    if args.iter().any(|a| a == "--designs") {
        return designs(&theme, cols, rows, &out);
    }
    let mut sections = String::new();
    for pair in &PAIRS {
        let left = in_pty(&homelab, &["tui", "--offline"], pair.keys, cols, rows);
        let right = from_shot(pair.screen, &theme, cols, rows);
        sections.push_str(&format!(
            "\n<section>\n  <h2>{}</h2>\n  <div class=\"pair\">\n    \
             <figure><figcaption>Homelab Rust — de echte client, met een nep-host</figcaption>{}</figure>\n    \
             <figure><figcaption>kp-tui — dezelfde schermopbouw, thema {}</figcaption>{}</figure>\n  </div>\n</section>",
            pair.caption, left, theme, right
        ));
    }
    std::fs::write(&out, page(&sections, &theme))?;
    println!("{out}: {} pair(s) at {cols}x{rows}", PAIRS.len());
    Ok(())
}

/// The twenty-five design directions, each as a picture with its paragraph.
///
/// The demo owns the table; this asks for it (`--designs`) rather than
/// keeping a second copy that could drift from it.
fn designs(theme: &str, cols: u16, rows: u16, out: &str) -> std::io::Result<()> {
    let listed = Command::new("cargo")
        .args(["run", "--quiet", "--example", "demo", "--", "--designs"])
        .output()?
        .stdout;
    let listed = String::from_utf8_lossy(&listed).to_string();
    let mut sections = String::new();
    let mut screen_now = String::new();
    let mut count = 0;
    for line in listed.lines() {
        let mut parts = line.splitn(4, '\t');
        let (Some(n), Some(screen), Some(name), Some(idea)) =
            (parts.next(), parts.next(), parts.next(), parts.next())
        else {
            continue;
        };
        if screen != screen_now {
            sections.push_str(&format!("\n<h2>{}</h2>", heading(screen)));
            screen_now = screen.to_string();
        }
        let shot = shot_at(
            &["--screen", "design", "--variant", n],
            theme,
            cols,
            rows,
            "2600",
        );
        sections.push_str(&format!(
            "\n<section class=\"design\">\n  <h3>{}. {}</h3>\n  <p class=\"idea\">{}</p>\n  {}\n</section>",
            count + 1,
            name,
            idea,
            shot
        ));
        count += 1;
    }
    std::fs::write(out, designs_page(&sections, theme))?;
    println!("{out}: {count} design(s) at {cols}x{rows}");
    Ok(())
}

fn heading(screen: &str) -> &'static str {
    match screen {
        "stacks" => "De stacks",
        "dashboard" => "Het dashboard",
        "settings" => "De instellingen",
        "logs" => "Het logboek",
        _ => "Het deploy-venster",
    }
}

fn designs_page(sections: &str, theme: &str) -> String {
    format!(
        r#"<!doctype html><meta charset="utf-8"><title>Vijf ontwerpen per scherm</title>
<style>
 body {{ background:#0b0f12; color:#d0d0d0; font-family:"Adwaita Sans",system-ui,sans-serif; margin:0; padding:24px 16px 48px }}
 h1 {{ font-weight:500; font-size:22px; margin:0 0 4px }}
 h2 {{ font-weight:500; font-size:18px; margin:40px 0 4px; color:#e6e6e6; border-bottom:1px solid #1d262c; padding-bottom:6px }}
 h3 {{ font-weight:500; font-size:15px; margin:22px 0 6px; color:#e6e6e6 }}
 p.lead, p.idea {{ color:#9aa4ad; max-width:78ch; line-height:1.65; margin:0 0 10px }}
 pre.shot {{ font-family:"FiraCode Nerd Font Mono","Adwaita Mono","DejaVu Sans Mono",monospace;
   font-size:12px; line-height:1.15; margin:0; padding:10px; border:1px solid #1d262c; border-radius:8px;
   overflow-x:auto; white-space:pre; background:#0b0f12; width:max-content; max-width:100% }}
</style>
<h1>Vijf ontwerpen per scherm, vijfentwintig in totaal</h1>
<p class="lead">Elk beeld is een richting, geen scherm: het toont het idee op de maat die het idee nodig heeft,
met de voorbeelden van de herbouw ernaast. Niets hiervan zit aan een toets — dat is de bedoeling, want een
richting die je niet kiest heeft dan maar één tekenfunctie gekost. Alles in thema {theme}; geen van deze
beelden noemt zelf een kleur.</p>
{sections}
"#
    )
}

/// One shot of the demo, replayed through the emulator.
fn shot_of(extra: &[&str], theme: &str, cols: u16, rows: u16) -> String {
    shot_at(extra, theme, cols, rows, "1200")
}

/// The same, at a chosen moment of the screen's own motion. A design
/// picture is taken later than a comparison shot: at 1200 ms a register
/// that deciphers its titles is still deciphering one, and a picture meant
/// to be judged should not be caught mid-word.
fn shot_at(extra: &[&str], theme: &str, cols: u16, rows: u16, at: &str) -> String {
    let size = format!("{cols}x{rows}");
    let mut args: Vec<&str> = vec!["run", "--quiet", "--example", "demo", "--"];
    args.extend_from_slice(extra);
    args.extend_from_slice(&[
        "--shot",
        "--at",
        at,
        "--size",
        &size,
        "--colors",
        "truecolor",
        "--theme",
        theme,
    ]);
    let out = Command::new("cargo")
        .args(&args)
        .output()
        .expect("the demo runs")
        .stdout;
    let body = out.splitn(2, |b| *b == b'\n').nth(1).unwrap_or(&[]);
    let mut fed = Vec::with_capacity(body.len() + rows as usize);
    for byte in body.iter().copied() {
        if byte == b'\n' {
            fed.push(b'\r');
        }
        fed.push(byte);
    }
    while fed.ends_with(b"\n") || fed.ends_with(b"\r") {
        fed.pop();
    }
    let mut parser = vt100::Parser::new(rows, cols, 0);
    parser.process(&fed);
    html(parser.screen(), cols, rows)
}

/// Run a full-screen program in a pseudo-terminal, press `keys`, and read
/// the screen it ends on.
fn in_pty(program: &str, args: &[&str], keys: &str, cols: u16, rows: u16) -> String {
    let pty = native_pty_system();
    let pair = pty
        .openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })
        .expect("a pseudo-terminal");
    let mut cmd = CommandBuilder::new(program);
    for a in args {
        cmd.arg(a);
    }
    cmd.env("TERM", "xterm-256color");
    cmd.env("COLORTERM", "truecolor");
    let mut child = pair.slave.spawn_command(cmd).expect("homelab starts");
    drop(pair.slave);

    let mut parser = vt100::Parser::new(rows, cols, 0);
    let mut reader = pair.master.try_clone_reader().expect("a reader");
    let mut writer = pair.master.take_writer().expect("a writer");
    // The emulator answers what a terminal answers; crossterm will not start
    // until something replies to its cursor-position request.
    let (tx, rx) = std::sync::mpsc::channel::<Vec<u8>>();
    std::thread::spawn(move || {
        let mut chunk = [0u8; 8192];
        while let Ok(n) = reader.read(&mut chunk) {
            if n == 0 || tx.send(chunk[..n].to_vec()).is_err() {
                return;
            }
        }
    });
    let drain = |parser: &mut vt100::Parser, writer: &mut Box<dyn Write + Send>, secs: f32| {
        let until = Instant::now() + Duration::from_secs_f32(secs);
        while Instant::now() < until {
            match rx.recv_timeout(until.saturating_duration_since(Instant::now())) {
                Ok(bytes) => {
                    parser.process(&bytes);
                    if bytes.windows(4).any(|w| w == b"\x1b[6n") {
                        let (y, x) = parser.screen().cursor_position();
                        let _ = write!(writer, "\x1b[{};{}R", y + 1, x + 1);
                        let _ = writer.flush();
                    }
                }
                Err(_) => break,
            }
        }
    };
    drain(&mut parser, &mut writer, 5.0);
    for key in keys.chars() {
        let _ = write!(writer, "{key}");
        let _ = writer.flush();
        drain(&mut parser, &mut writer, 0.8);
    }
    drain(&mut parser, &mut writer, 1.5);
    let html = html(parser.screen(), cols, rows);
    let _ = write!(writer, "q");
    let _ = writer.flush();
    std::thread::sleep(Duration::from_millis(200));
    let _ = child.kill();
    let _ = child.wait();
    html
}

/// The demo's one-frame shot of a screen, replayed through the same
/// emulator so both columns are read the same way.
fn from_shot(screen: &str, theme: &str, cols: u16, rows: u16) -> String {
    shot_of(&["--screen", screen], theme, cols, rows)
}

/// A screen as one `<pre>`, a span per run of equal styling.
fn html(screen: &vt100::Screen, cols: u16, rows: u16) -> String {
    let mut out = String::from("<pre class=\"shot\">");
    for y in 0..rows {
        if y > 0 {
            out.push('\n');
        }
        let mut run = String::new();
        let mut style: Option<(String, String, bool)> = None;
        for x in 0..cols {
            let cell = screen.cell(y, x);
            let (text, here) = match cell {
                Some(c) => {
                    let mut fg = colour(c.fgcolor(), "#d0d0d0");
                    let mut bg = colour(c.bgcolor(), "#0b0f12");
                    if c.inverse() {
                        std::mem::swap(&mut fg, &mut bg);
                    }
                    let t = c.contents();
                    (
                        if t.is_empty() {
                            " ".to_string()
                        } else {
                            t.to_string()
                        },
                        (fg, bg, c.bold()),
                    )
                }
                None => (
                    " ".to_string(),
                    ("#d0d0d0".to_string(), "#0b0f12".to_string(), false),
                ),
            };
            if style.as_ref() != Some(&here) {
                if let Some(was) = style.take() {
                    out.push_str(&span(&was, &run));
                    run.clear();
                }
                style = Some(here);
            }
            run.push_str(&text);
        }
        if let Some(was) = style {
            out.push_str(&span(&was, &run));
        }
    }
    out.push_str("</pre>");
    out
}

fn span(style: &(String, String, bool), text: &str) -> String {
    format!(
        "<span style=\"color:{};background:{}{}\">{}</span>",
        style.0,
        style.1,
        if style.2 { ";font-weight:600" } else { "" },
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
    )
}

fn colour(c: vt100::Color, fallback: &str) -> String {
    match c {
        vt100::Color::Default => fallback.to_string(),
        vt100::Color::Rgb(r, g, b) => format!("#{r:02x}{g:02x}{b:02x}"),
        vt100::Color::Idx(i) => {
            // The 256-colour cube, so a theme rendered at a lower depth
            // still comes out as the colour it picked.
            let (r, g, b) = match i {
                0..=7 => [
                    (0, 0, 0),
                    (205, 0, 0),
                    (0, 205, 0),
                    (205, 205, 0),
                    (0, 0, 238),
                    (205, 0, 205),
                    (0, 205, 205),
                    (229, 229, 229),
                ][i as usize],
                8..=15 => [
                    (127, 127, 127),
                    (255, 0, 0),
                    (0, 255, 0),
                    (255, 255, 0),
                    (92, 92, 255),
                    (255, 0, 255),
                    (0, 255, 255),
                    (255, 255, 255),
                ][i as usize - 8],
                16..=231 => {
                    let n = i - 16;
                    let step = |v: u8| if v == 0 { 0u8 } else { 55 + v * 40 };
                    (step(n / 36), step((n % 36) / 6), step(n % 6))
                }
                _ => {
                    let v = 8 + (i - 232) * 10;
                    (v, v, v)
                }
            };
            format!("#{r:02x}{g:02x}{b:02x}")
        }
    }
}

fn page(sections: &str, theme: &str) -> String {
    format!(
        r#"<!doctype html><meta charset="utf-8"><title>Homelab naast kp-tui</title>
<style>
 body {{ background:#0b0f12; color:#d0d0d0; font-family:"Adwaita Sans",system-ui,sans-serif; margin:0; padding:24px 16px 48px }}
 h1 {{ font-weight:500; font-size:22px; margin:0 0 4px }}
 p.lead {{ color:#9aa4ad; max-width:70ch; line-height:1.6; margin:0 0 28px }}
 h2 {{ font-weight:500; font-size:16px; margin:32px 0 10px; color:#e6e6e6 }}
 .pair {{ display:flex; gap:18px; flex-wrap:wrap }}
 figure {{ margin:0; flex:1 1 720px; min-width:0 }}
 figcaption {{ color:#8b959e; font-size:12px; margin:0 0 6px }}
 pre.shot {{ font-family:"FiraCode Nerd Font Mono","Adwaita Mono","DejaVu Sans Mono",monospace;
   font-size:12px; line-height:1.15; margin:0; padding:10px; border:1px solid #1d262c; border-radius:8px;
   overflow-x:auto; white-space:pre; background:#0b0f12 }}
</style>
<h1>Homelab Rust naast kp-tui, vijf schermen</h1>
<p class="lead">Links Homelab's eigen client (<code>homelab tui --offline</code>), rechts hetzelfde scherm
herbouwd op kp-tui in thema {theme}. Beide zijn door dezelfde terminal-emulator gehaald, dus wat hier
verschilt verschilt ook in een echte terminal. De gegevens verschillen wel: Homelab draait op zijn eigen
nep-host, de herbouw op de vaste voorbeelden van de demo, en de demo tekent geen tabbalk, dus die rijen
chrome links zijn rechts scherm.</p>
{sections}
"#
    )
}
