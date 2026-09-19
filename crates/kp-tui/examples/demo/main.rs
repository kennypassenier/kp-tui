//! `kp-tui-demo [--screen dashboard|…|deploy|doctor|splash] [--theme NAME]
//!  [--colors truecolor|256|16] [--reduced-motion] [--config PATH] [--fps N]
//!  [--synthetic-logs] [--exit-after SECONDS] [--shot] [--at MS] [--size WxH]
//!  [--keys CHARS]`
//!
//! `--exit-after` is for measuring: the demo quits by itself and prints its
//! own CPU time, frame count and mean draw time to stderr.
//!
//! `--shot` needs no terminal: it draws the chosen screen once, at `--at`
//! milliseconds into its motion, and prints it as ANSI — one block per
//! theme in `--theme a,b,c`. That is the same code path the demo runs, so
//! a picture of it cannot drift from what the demo shows.

use std::{
    env, fs, io,
    path::PathBuf,
    time::{Duration, Instant},
};

use crossterm::{cursor::SetCursorStyle, event, execute};
mod app;
mod config;
mod deploy;
mod doctor;
mod fleet;
mod logstream;
mod ops;
mod settings;
mod splash;

use app::{App, Screen};
mod shot;
use config::Config;
use kp_tui::{
    ThemeId,
    color::ColorDepth,
    fx::Motion,
    live::{self, Sampler},
    logs::Feed,
};

const SAMPLE_EVERY: Duration = Duration::from_millis(500);

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    let arg = |name: &str| {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };
    let flag = |name: &str| args.iter().any(|a| a == name);
    let getenv = |k: &str| env::var(k).ok();

    let path = arg("--config")
        .map(PathBuf::from)
        .or_else(|| config::default_path(getenv));
    let mut cfg = path.as_deref().map(Config::load).unwrap_or_default();
    if let Some(t) = arg("--theme").as_deref().and_then(ThemeId::from_name) {
        cfg.theme = t;
    }
    if flag("--reduced-motion") || getenv("KP_REDUCED_MOTION").as_deref() == Some("1") {
        cfg.motion = Motion::Reduced;
    }
    let depth = ColorDepth::detect(arg("--colors").as_deref(), getenv);
    let fps: u32 = arg("--fps")
        .and_then(|v| v.parse().ok())
        .unwrap_or(15)
        .clamp(1, 60);
    let frame_every = Duration::from_micros(1_000_000 / fps as u64);
    let exit_after = arg("--exit-after")
        .and_then(|v| v.parse::<f64>().ok())
        .map(Duration::from_secs_f64);

    let mut app = App::new(cfg, depth, path);
    app.screen = match arg("--screen").as_deref() {
        Some("components") => Screen::Components,
        Some("console") => Screen::Console,
        Some("effects") => Screen::Effects,
        Some("fleet") => Screen::Fleet,
        Some("ops") => Screen::Ops,
        Some("settings") => Screen::Settings,
        Some("logs") => Screen::LogStream,
        Some("deploy") => Screen::Deploy,
        Some("doctor") => Screen::Doctor,
        Some("splash") => Screen::Splash,
        _ => Screen::Dashboard,
    };
    if flag("--shot") {
        let size = arg("--size")
            .and_then(|v| {
                let (w, h) = v.split_once('x')?;
                Some((w.parse().ok()?, h.parse().ok()?))
            })
            .unwrap_or((118, 30));
        let at = arg("--at").and_then(|v| v.parse().ok()).unwrap_or(0);
        let themes = arg("--theme").unwrap_or_else(|| "cyberpunk,terminal".into());
        let keys = arg("--keys").unwrap_or_default();
        return shot::print(&mut app, &themes, size, at, &keys);
    }
    app.dash.fps = fps;

    let mut feed = if flag("--synthetic-logs") {
        Feed::synthetic("asked for with --synthetic-logs")
    } else {
        Feed::journal().unwrap_or_else(|_| Feed::synthetic("journalctl could not be started"))
    };
    let mut sampler = Sampler::new(feed.child_pid());
    sampler.sample(); // a rate needs a first reading

    let mut terminal = ratatui::init();
    let mut cursor = app.theme.a.cursor;
    execute!(io::stdout(), cursor)?;
    let start = Instant::now();
    let mut last = start;
    let mut next_sample = start + SAMPLE_EVERY;
    let mut next_frame = start;
    let (mut frames, mut draw_total) = (0u64, Duration::ZERO);
    let result = loop {
        let now = Instant::now();
        if now >= next_sample {
            if let Some(s) = sampler.sample() {
                app.dash
                    .push_sample(now.duration_since(start).as_secs_f64(), s);
            }
            next_sample = (next_sample + SAMPLE_EVERY).max(now + Duration::from_millis(1));
        }
        feed.drain(&mut app.dash.logs, 400);
        app.dash.feed_live = feed.live();
        app.dash.feed_label = feed.label();
        app.tick(now.duration_since(last).as_millis() as u32);
        last = now;

        if now >= next_frame {
            let t0 = Instant::now();
            let drawn = terminal.draw(|f| app.draw(f));
            if let Err(e) = drawn {
                break Err(e);
            }
            let took = t0.elapsed();
            frames += 1;
            draw_total += took;
            // A running mean over about a second of frames.
            let ms = took.as_secs_f32() * 1000.0;
            app.dash.draw_ms += (ms - app.dash.draw_ms) / fps.min(frames as u32) as f32;
            next_frame = (next_frame + frame_every).max(now + Duration::from_millis(1));
        }

        // Sleep in the poll until the next frame or sample is due; a key
        // wakes it early and is drawn at the next frame.
        let wait = next_frame
            .min(next_sample)
            .saturating_duration_since(Instant::now());
        if event::poll(wait)?
            && let event::Event::Key(k) = event::read()?
        {
            app.key(k);
        }
        if app.theme.a.cursor != cursor {
            cursor = app.theme.a.cursor;
            execute!(io::stdout(), cursor)?;
        }
        if app.quit || exit_after.is_some_and(|d| start.elapsed() >= d) {
            break Ok(());
        }
    };
    execute!(io::stdout(), SetCursorStyle::DefaultUserShape)?;
    ratatui::restore();

    if exit_after.is_some() {
        let secs = start.elapsed().as_secs_f64();
        let ticks = |p: &str| {
            fs::read_to_string(p)
                .map(|t| live::parse_proc_ticks(&t))
                .unwrap_or(0) as f64
        };
        let own = ticks("/proc/self/stat") / live::CLK_TCK;
        let child = feed
            .child_pid()
            .map(|p| ticks(&format!("/proc/{p}/stat")) / live::CLK_TCK)
            .unwrap_or(0.0);
        eprintln!(
            "ran {secs:.1} s · {frames} frames ({:.1} fps) · mean draw {:.2} ms · demo cpu {own:.2} s ({:.2} % of one core) \
             · journalctl cpu {child:.2} s ({:.2} %) · log lines {} · feed {}",
            frames as f64 / secs,
            draw_total.as_secs_f64() * 1000.0 / frames.max(1) as f64,
            100.0 * own / secs,
            100.0 * child / secs,
            app.dash.logs.len(),
            feed.label(),
        );
    }
    drop(feed);
    result
}
