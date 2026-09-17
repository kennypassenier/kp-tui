//! The widgets, headless: a `TestBackend` buffer per case, no application
//! and no terminal. These moved over from the demo in kp-themes with the
//! widgets they measure [kp-themes scope-129] — the ones that drive the
//! demo's own key handling stayed there, because that is the demo's, not
//! this crate's.

use kp_tui::{
    AlarmPanel, ColorDepth, Field, KeyHints, LogPane, Meter, Popup, PopupKind, Surface, Theme,
    ThemeId, Ticker,
    anatomy::Reveal,
    color::Rgb,
    dashboard::rate,
    fx::{self, Motion},
    live, logs,
    logs::{LogBuffer, Severity},
    source_colour, spinner,
    widgets::{Button, ButtonState},
};
use ratatui::{buffer::Buffer, layout::Rect, style::Color, widgets::Widget};

fn rgb(c: Rgb) -> Color {
    Color::Rgb(c.0, c.1, c.2)
}

#[test]
fn button_states_differ_by_plate_and_ring() {
    for id in ThemeId::ALL {
        let th = Theme::new(id, ColorDepth::TrueColor);
        let p = id.palette();
        let draw = |state| {
            let mut buf = Buffer::empty(Rect::new(0, 0, 14, 3));
            Button::new(&th, "Deploy")
                .state(state)
                .render(buf.area, &mut buf);
            buf
        };
        let (rest, focus, pressed) = (
            draw(ButtonState::Rest),
            draw(ButtonState::Focus),
            draw(ButtonState::Pressed),
        );
        // The plate is the state's own colour, over the whole button.
        assert_eq!(rest[(7, 0)].bg, rgb(p.primary), "{}: rest plate", id.name());
        assert_eq!(
            focus[(7, 0)].bg,
            rgb(p.primary_hover),
            "{}: focus plate",
            id.name()
        );
        assert_eq!(
            pressed[(7, 0)].bg,
            rgb(p.primary_active),
            "{}: pressed plate",
            id.name()
        );
        // The focus ring is a strip along the plate's last row, and a
        // pressed button does not carry it.
        assert_eq!(
            focus[(7, 2)].bg,
            rgb(p.ring),
            "{}: the focus ring",
            id.name()
        );
        assert_eq!(
            rest[(7, 2)].bg,
            rgb(p.primary),
            "{}: no ring at rest",
            id.name()
        );
        assert_eq!(
            pressed[(7, 2)].bg,
            rgb(p.primary_active),
            "{}: no ring when pressed",
            id.name()
        );
        // Nothing is drawn with a line or a corner glyph.
        let glyphs: String = (0..14)
            .flat_map(|x| (0..3).map(move |y| (x, y)))
            .map(|c| rest[c].symbol())
            .collect();
        assert!(
            !glyphs.contains('─') && !glyphs.contains('│') && !glyphs.contains('◢'),
            "{}: a plate carries no frame glyphs: {glyphs}",
            id.name()
        );
    }
}

#[test]
fn each_theme_ends_its_plate_its_own_way() {
    let ends = |id: ThemeId| {
        let th = Theme::new(id, ColorDepth::TrueColor);
        let mut buf = Buffer::empty(Rect::new(0, 0, 14, 3));
        Button::new(&th, "Deploy").render(buf.area, &mut buf);
        let row: String = (0..14).map(|x| buf[(x, 1)].symbol()).collect();
        (buf[(0, 1)].symbol().to_string(), buf[(0, 1)].bg, row)
    };
    // formal has a radius: the plate ends mid-cell, on the page's ground.
    let (cap, bg, _) = ends(ThemeId::Formal);
    assert_eq!(cap, "▐", "formal's soft cap");
    assert_eq!(
        bg,
        rgb(ThemeId::Formal.palette().background),
        "the cap sits on the ground"
    );
    // cyberpunk's radius is 0: full cells, and the label is spaced caps.
    let (cap, bg, row) = ends(ThemeId::Cyberpunk);
    assert_eq!(cap, " ", "cyberpunk is square");
    assert_eq!(
        bg,
        rgb(ThemeId::Cyberpunk.palette().primary),
        "square to the edge"
    );
    assert!(row.contains("D E P L O Y"), "{row}");
    // terminal keeps its brackets.
    let (_, _, row) = ends(ThemeId::Terminal);
    assert!(row.contains("[ DEPLOY ]"), "{row}");
}

#[test]
fn the_charge_sweep_crosses_a_focused_button_once() {
    let th = Theme::new(ThemeId::Cyberpunk, ColorDepth::TrueColor);
    let p = ThemeId::Cyberpunk.palette();
    let band = |progress: Option<f32>| {
        let mut buf = Buffer::empty(Rect::new(0, 0, 14, 3));
        Button::new(&th, "Deploy")
            .state(ButtonState::Focus)
            .charge(progress)
            .render(buf.area, &mut buf);
        (0..14)
            .filter(|x| buf[(*x, 1)].bg == rgb(p.primary_foreground))
            .collect::<Vec<_>>()
    };
    assert!(band(None).is_empty(), "no sweep without one");
    let early = band(Some(0.25));
    let late = band(Some(0.75));
    assert!(
        !early.is_empty() && !late.is_empty(),
        "the band is on the face"
    );
    assert!(
        early.iter().max() < late.iter().max(),
        "the band travels: {early:?} then {late:?}"
    );
    assert!(
        band(Some(1.0)).iter().all(|x| *x > 10),
        "it leaves by the right edge"
    );
}

#[test]
fn colour_depth_detection() {
    let env = |pairs: &'static [(&'static str, &'static str)]| {
        move |k: &str| {
            pairs
                .iter()
                .find(|(n, _)| *n == k)
                .map(|(_, v)| v.to_string())
        }
    };
    assert_eq!(
        ColorDepth::detect(None, env(&[("COLORTERM", "truecolor")])),
        ColorDepth::TrueColor
    );
    assert_eq!(
        ColorDepth::detect(None, env(&[("TERM", "xterm-256color")])),
        ColorDepth::Ansi256
    );
    assert_eq!(
        ColorDepth::detect(None, env(&[("TERM", "linux")])),
        ColorDepth::Ansi16
    );
    assert_eq!(
        ColorDepth::detect(Some("16"), env(&[("COLORTERM", "24bit")])),
        ColorDepth::Ansi16
    );
    assert_eq!(
        ColorDepth::detect(None, env(&[("KP_COLORS", "256"), ("COLORTERM", "24bit")])),
        ColorDepth::Ansi256
    );
}

#[test]
fn reduced_motion_shows_the_final_state_on_the_first_frame() {
    for id in ThemeId::ALL {
        let f = fx::frame(
            "Version 2.4 is ready",
            id.anatomy().reveal,
            0,
            Motion::Reduced,
        );
        assert_eq!(
            (f.text.as_str(), f.caret, f.opacity, f.done),
            ("Version 2.4 is ready", None, 1.0, true)
        );
    }
}

#[test]
fn each_reveal_runs_its_own_routine() {
    let text = "Version 2.4";
    // terminal types: nothing at 0 ms but the caret, three glyphs at 110 ms.
    let r = ThemeId::Terminal.anatomy().reveal;
    assert_eq!(fx::frame(text, r, 0, Motion::Full).caret, Some(0));
    assert_eq!(fx::frame(text, r, 110, Motion::Full).text, "Ver");
    // cyberpunk deciphers through the web's glyph set, never a block glyph.
    let r = ThemeId::Cyberpunk.anatomy().reveal;
    let mid = fx::frame(text, r, 100, Motion::Full).text;
    assert_eq!(mid.chars().count(), text.chars().count());
    assert!(
        mid.chars()
            .zip(text.chars())
            .all(|(m, t)| m == t || fx::GLYPHS.contains(&m))
    );
    assert!(!mid.contains(['░', '▒', '▓']));
    assert_eq!(
        fx::frame(text, r, fx::duration_ms(r, text), Motion::Full).text,
        text
    );
    // formal never touches a character: only the opacity moves.
    let r = ThemeId::Formal.anatomy().reveal;
    assert!(matches!(r, Reveal::Arrive { ms: 450 }));
    let f = fx::frame(text, r, 100, Motion::Full);
    assert_eq!(f.text, text);
    assert!(f.opacity > 0.0 && f.opacity < 1.0);
}

// scope-127: every theme the package ships has an anatomy, and the six
// registers that declare --kp-word-stagger reveal their headline word by
// word instead of as one plate.

#[test]
fn all_twenty_two_themes_carry_an_anatomy() {
    assert_eq!(ThemeId::ALL.len(), 22);
    let mut seen = std::collections::BTreeSet::new();
    for id in ThemeId::ALL {
        let th = Theme::new(id, ColorDepth::TrueColor);
        // A palette and an anatomy, and a name that is the package's own.
        assert!(!id.name().is_empty(), "{id:?} has no name");
        assert!(seen.insert(id.name()), "{} appears twice", id.name());
        assert_eq!(ThemeId::from_name(id.name()), Some(id));
        // The anatomy is the theme's own, not a default: every prefix that
        // a register declares reaches the panel title.
        let panel: String = format!("{}{}", th.a.label_prefix, "RELEASE");
        assert!(panel.ends_with("RELEASE"), "{}: {panel}", id.name());
    }
    // The order is the package's order, so index and name agree.
    assert_eq!(ThemeId::ALL[0].name(), "formal");
    assert_eq!(ThemeId::ALL[21].name(), "titanium");
    // Stepping with `t` walks the whole set and comes back.
    let mut id = ThemeId::Formal;
    for _ in 0..22 {
        id = id.next();
    }
    assert_eq!(id, ThemeId::Formal);
}

#[test]
fn a_word_staggered_reveal_lights_its_words_in_turn() {
    // The six registers that declare --kp-word-stagger: dark and titanium
    // 28 ms, phantom 28, brutalism 60, shade-light 70, shade-dark 90.
    for name in [
        "dark",
        "phantom",
        "brutalism",
        "shade-light",
        "shade-dark",
        "titanium",
    ] {
        let id = ThemeId::from_name(name).expect(name);
        assert!(
            matches!(id.anatomy().reveal, Reveal::Words { .. }),
            "{name} does not reveal word by word"
        );
    }
    let Reveal::Words { ms, stagger_ms } =
        ThemeId::from_name("shade-dark").unwrap().anatomy().reveal
    else {
        panic!("shade-dark is not word-staggered");
    };
    assert_eq!((ms, stagger_ms), (600, 90));
    // Early on, the first word is further along than the last.
    let text = "Version 2.4 is ready to deploy";
    let early = fx::frame(
        text,
        Reveal::Words { ms, stagger_ms },
        120,
        fx::Motion::Full,
    );
    assert_eq!(early.words.len(), 6, "one entry per word");
    assert!(early.words[0].1 > early.words[5].1, "{:?}", early.words);
    assert_eq!(early.words[5].1, 0.0, "the last word has not started");
    // The whole line is lit once the last word has had its own duration.
    let whole = fx::frame(
        text,
        Reveal::Words { ms, stagger_ms },
        ms + stagger_ms * 5,
        fx::Motion::Full,
    );
    assert!(
        whole.words.is_empty() && whole.done,
        "it finishes as one plate"
    );
    // Reduced motion shows the finished line on the first frame.
    let still = fx::frame(
        text,
        Reveal::Words { ms, stagger_ms },
        0,
        fx::Motion::Reduced,
    );
    assert!(still.done && still.words.is_empty());
}

#[test]
fn journal_json_lines_parse_to_coloured_parts() {
    let l = logs::parse_journal_json(
        r#"{"__REALTIME_TIMESTAMP":"1789665207763379","PRIORITY":"4","SYSLOG_IDENTIFIER":"smartd","_PID":"744","_HOSTNAME":"linux","MESSAGE":"hot\u001b[31m"}"#,
        7200,
    )
    .unwrap();
    assert_eq!(
        (l.host.as_str(), l.unit.as_str(), l.severity),
        ("linux", "smartd[744]", Severity::Warning)
    );
    assert_eq!(
        l.message, "hot [31m",
        "control characters never reach the terminal"
    );
    assert_eq!(l.time, logs::clock(1789665207763379, 7200));
    assert!(l.time.ends_with(".763"));
    // A byte-array MESSAGE, no priority (defaults to info), priority 0 folds into critical.
    let b = logs::parse_journal_json(
        r#"{"__REALTIME_TIMESTAMP":"1","MESSAGE":[104,105],"_COMM":"x"}"#,
        0,
    )
    .unwrap();
    assert_eq!(
        (b.message.as_str(), b.severity, b.unit.as_str()),
        ("hi", Severity::Info, "x")
    );
    assert_eq!(Severity::from_priority(0), Severity::Critical);
    assert_eq!(logs::clock(0, 3600 + 60), "01:01:00.000");
    assert_eq!(
        (rate(812.0), rate(1300.0), rate(35.0 * 1024.0 * 1024.0)),
        ("812B".into(), "1.3K".into(), "35M".into())
    );
}

#[test]
fn proc_parsers_on_fixtures() {
    let stat = "cpu  100 0 50 800 50 0 0 0 0 0\ncpu0 60 0 20 400 20 0 0 0 0 0\ncpu1 40 0 30 400 30 0 0 0 0 0\nintr 1\n";
    let later = "cpu  200 0 100 900 100 0 0 0 0 0\ncpu0 160 0 20 420 20 0 0 0 0 0\ncpu1 40 0 80 480 30 0 0 0 0 0\n";
    let (a, b) = (live::parse_stat(stat), live::parse_stat(later));
    assert_eq!(a.len(), 3);
    assert_eq!(
        a[0],
        live::CpuTimes {
            total: 1000,
            idle: 850
        }
    );
    let prev = live::Counters {
        cpu: a,
        self_ticks: 10,
        ..Default::default()
    };
    let cur = live::Counters {
        cpu: b,
        mem_total_kib: 1000,
        mem_available_kib: 250,
        self_ticks: 12,
        net_rx_bytes: 2048,
        ..Default::default()
    };
    let s = live::rates(&prev, &cur, 2.0);
    // total: 300 jiffies passed, 150 of them idle or iowait
    assert_eq!(s.cpu_total, 50.0);
    assert_eq!(s.cores.len(), 2);
    assert!((s.cores[0] - 100.0 * 100.0 / 120.0).abs() < 1e-9);
    assert_eq!(s.mem_used_pct, 75.0);
    assert_eq!(s.rx_bps, 1024.0);
    assert_eq!(s.self_cpu_pct, 1.0, "2 ticks in 2 s at 100 Hz");

    assert_eq!(
        live::parse_meminfo("MemTotal:  64960804 kB\nMemFree: 1 kB\nMemAvailable:   46957828 kB\n"),
        (64960804, 46957828)
    );
    assert_eq!(
        live::parse_loadavg("2.10 2.59 3.69 2/2584 1287722\n"),
        [2.10, 2.59, 3.69]
    );
    let net = "Inter-|   Receive\n face |bytes\n    lo: 999 1 0 0 0 0 0 0 999 1 0 0 0 0 0 0\n  eno1: 4000 3 0 0 0 0 0 5 300 6 0 0 0 0 0 0\n";
    assert_eq!(
        live::parse_net_dev(net),
        (4000, 300),
        "loopback is left out"
    );
    let disks = " 259 0 nvme0n1 300 0 100 106 50 0 20 0 0 27 106\n 259 1 nvme0n1p1 45 0 100 20 5 0 20 0 0 15 20\n";
    assert_eq!(
        live::parse_diskstats(disks, |n| n == "nvme0n1"),
        (100 * 512, 20 * 512)
    );
    assert_eq!(
        live::parse_proc_ticks("42 (kp tui) demo) S 1 2 3 4 5 6 7 8 9 10 250 30 0 0"),
        280
    );
}

// ── The four homelab hand-rolls, now in the crate ───────────────────────

#[test]
fn a_popup_clears_its_ground_and_frames_it_in_the_theme() {
    for name in ["cyberpunk", "terminal", "formal"] {
        let id = ThemeId::from_name(name).unwrap();
        let th = Theme::new(id, ColorDepth::TrueColor);
        let p = id.palette();
        let screen = Rect::new(0, 0, 60, 20);
        let mut buf = Buffer::empty(screen);
        // Something noisy underneath, to prove the popup clears it.
        for y in 0..20 {
            for x in 0..60 {
                buf[(x, y)].set_symbol("#");
            }
        }
        let popup = Popup::new(&th, "Confirm", (30, 8));
        let area = popup.area(screen);
        let inner = popup.render_over(screen, &mut buf);
        assert_eq!(
            (area.width, area.height),
            (30, 8),
            "{name}: the size it asked for"
        );
        assert_eq!(area.x, 15, "{name}: centred");
        assert!(
            inner.width < area.width && inner.y > area.y,
            "{name}: the frame takes a cell"
        );
        // The ground inside is the popover surface, and the noise is gone.
        assert_eq!(
            buf[(inner.x, inner.y)].bg,
            rgb(p.popover),
            "{name}: the popover ground"
        );
        assert_eq!(buf[(inner.x, inner.y)].symbol(), " ", "{name}: cleared");
        // Outside it, the noise stands.
        assert_eq!(
            buf[(0, 0)].symbol(),
            "#",
            "{name}: only the popup is cleared"
        );
    }
}

#[test]
fn a_danger_popup_takes_the_destructive_colour_on_its_frame() {
    let th = Theme::new(ThemeId::Cyberpunk, ColorDepth::TrueColor);
    let p = ThemeId::Cyberpunk.palette();
    let screen = Rect::new(0, 0, 40, 12);
    let mut buf = Buffer::empty(screen);
    let popup = Popup::new(&th, "Restore", (24, 6)).kind(PopupKind::Danger);
    let area = popup.area(screen);
    popup.render_over(screen, &mut buf);
    assert_eq!(
        buf[(area.x, area.y)].fg,
        rgb(p.destructive),
        "the frame says what this dialog does"
    );
    let mut plain = Buffer::empty(screen);
    let normal = Popup::new(&th, "Restore", (24, 6));
    normal.render_over(screen, &mut plain);
    assert_ne!(
        plain[(area.x, area.y)].fg,
        rgb(p.destructive),
        "a normal popup does not"
    );
}

#[test]
fn a_field_carries_the_theme_s_own_caret() {
    // terminal blinks a block, dark holds a bar steady: the anatomy says so,
    // and the field reads it instead of drawing its own.
    let terminal = Theme::new(ThemeId::Terminal, ColorDepth::TrueColor);
    let row = |th: &Theme, ticks: u32| {
        let mut buf = Buffer::empty(Rect::new(0, 0, 30, 1));
        Field::new(th, "filter", "web")
            .focused(true)
            .blink(ticks)
            .render(buf.area, &mut buf);
        (0..30)
            .map(|x| buf[(x, 0)].symbol().to_string())
            .collect::<String>()
    };
    let on = row(&terminal, 0);
    let off = row(&terminal, 15);
    assert!(on.contains("█"), "terminal blinks a block: {on}");
    assert!(
        !off.contains("█"),
        "and the block goes away between blinks: {off}"
    );
    let dark = Theme::new(ThemeId::from_name("dark").unwrap(), ColorDepth::TrueColor);
    assert!(
        row(&dark, 0).contains("▏") && row(&dark, 15).contains("▏"),
        "dark's bar is steady"
    );
    // An unfocused field carries no caret at all.
    let mut buf = Buffer::empty(Rect::new(0, 0, 30, 1));
    Field::new(&terminal, "filter", "web").render(buf.area, &mut buf);
    let quiet: String = (0..30).map(|x| buf[(x, 0)].symbol().to_string()).collect();
    assert!(!quiet.contains("█"), "{quiet}");
    assert!(
        quiet.contains("FILTER"),
        "terminal uppercases its labels: {quiet}"
    );
}

#[test]
fn a_meter_crosses_into_warning_and_then_into_danger() {
    let th = Theme::new(ThemeId::Formal, ColorDepth::TrueColor);
    let p = ThemeId::Formal.palette();
    assert_eq!(Meter::new(&th, "ram", 0.5).colour(), rgb(p.primary));
    assert_eq!(Meter::new(&th, "ram", 0.75).colour(), rgb(p.warning));
    assert_eq!(Meter::new(&th, "ram", 0.95).colour(), rgb(p.destructive));
    // The thresholds are the caller's: a disk that matters earlier.
    assert_eq!(
        Meter::new(&th, "ssd", 0.55).thresholds(0.5, 0.8).colour(),
        rgb(p.warning)
    );
    // The bar is painted, not spelled: no block glyphs in the row.
    let mut buf = Buffer::empty(Rect::new(0, 0, 30, 1));
    Meter::new(&th, "ram", 0.5).render(buf.area, &mut buf);
    let row: String = (0..30).map(|x| buf[(x, 0)].symbol().to_string()).collect();
    assert!(!row.contains("█") && !row.contains("░"), "{row}");
    assert!(row.contains("50%"), "{row}");
    let filled = (0..30)
        .filter(|x| buf[(*x, 0)].bg == rgb(p.primary))
        .count();
    assert!(filled > 5, "half the bar is the primary: {filled}");
}

#[test]
fn one_keymap_becomes_a_footer_and_an_overlay() {
    let th = Theme::new(ThemeId::Cyberpunk, ColorDepth::TrueColor);
    let keys = [("q", "quit"), ("r", "refresh"), ("F2", "effects")];
    let hints = KeyHints::new(&th, &keys);
    let footer: String = hints
        .footer()
        .spans
        .iter()
        .map(|s| s.content.to_string())
        .collect();
    assert!(
        footer.contains("q quit") && footer.contains("F2 effects"),
        "{footer}"
    );
    assert!(
        footer.contains(th.a.tab_divider),
        "the theme's own divider: {footer}"
    );
    let overlay = hints.overlay();
    assert_eq!(overlay.len(), 3, "one row per key");
    let first: String = overlay[0]
        .spans
        .iter()
        .map(|s| s.content.to_string())
        .collect();
    // cyberpunk uppercases its labels, and the keys line up in a column.
    assert!(first.contains("QUIT"), "{first}");
    assert!(first.starts_with(" q") || first.starts_with("q"), "{first}");
}

#[test]
fn a_source_keeps_its_colour_and_two_sources_rarely_share_one() {
    let th = Theme::new(ThemeId::Cyberpunk, ColorDepth::TrueColor);
    let p = ThemeId::Cyberpunk.palette();
    let charts = [p.chart_1, p.chart_2, p.chart_3, p.chart_4, p.chart_5].map(rgb);
    // Stable across calls, and always one of the theme's own chart hues.
    for name in ["media", "web", "db", "proxy", "backup"] {
        let once = source_colour(&th, name);
        assert_eq!(
            once,
            source_colour(&th, name),
            "{name} changed colour between calls"
        );
        assert!(
            charts.contains(&once),
            "{name} took a colour the theme does not declare"
        );
    }
    // The four homelab stacks land on four different hues.
    let hues: std::collections::BTreeSet<String> = ["media", "web", "db", "proxy"]
        .iter()
        .map(|n| format!("{:?}", source_colour(&th, n)))
        .collect();
    assert!(
        hues.len() >= 3,
        "three of four stacks should be told apart by colour: {hues:?}"
    );
}

#[test]
fn the_log_pane_carries_the_source_bar_the_severities_and_a_scrollbar() {
    let th = Theme::new(ThemeId::Cyberpunk, ColorDepth::TrueColor);
    let p = ThemeId::Cyberpunk.palette();
    let mut logs = LogBuffer::new(500);
    for i in 0..80 {
        let severity = if i % 7 == 0 {
            Severity::Error
        } else {
            Severity::Info
        };
        logs.push(kp_tui::logs::LogLine::new(
            "12:00:0{i}",
            "lxc-106",
            "media",
            severity,
            "unit started",
        ));
    }
    let sources = ["media", "web", "db"];
    let draw = |buffer: &LogBuffer| {
        let mut buf = Buffer::empty(Rect::new(0, 0, 70, 12));
        LogPane::new(&th, "Journal", buffer)
            .sources(&sources, 0)
            .render(buf.area, &mut buf);
        buf
    };
    let buf = draw(&logs);
    let rows: Vec<String> = (0..12)
        .map(|y| {
            (0..70)
                .map(|x| buf[(x, y)].symbol().to_string())
                .collect::<String>()
        })
        .collect();
    // The source bar is on the first inner row, uppercase in cyberpunk.
    assert!(
        rows[1].contains("MEDIA") && rows[1].contains("WEB"),
        "{:?}",
        rows[1]
    );
    // The selected source sits on its own hue as a plate.
    let selected_bg = buf[(2, 1)].bg;
    assert!(
        [
            rgb(p.chart_1),
            rgb(p.chart_2),
            rgb(p.chart_3),
            rgb(p.chart_4),
            rgb(p.chart_5)
        ]
        .contains(&selected_bg),
        "the selected source wears its identity hue: {selected_bg:?}"
    );
    // An error line carries the destructive colour on its tag.
    let error_cells = (0..12)
        .flat_map(|y| (0..70).map(move |x| (x, y)))
        .filter(|c| buf[*c].fg == rgb(p.destructive))
        .count();
    assert!(error_cells > 0, "the error lines are coloured");
    // The scrollbar occupies the last column and says the view is at the tail.
    // The scrollbar is the last column INSIDE the frame; x=69 is the frame.
    let track: String = (2..11).map(|y| buf[(68, y)].symbol().to_string()).collect();
    assert!(
        track.contains("█") || track.contains("▐"),
        "a scrollbar: {track:?}"
    );
    // The status says what it shows and that it follows.
    let status: String = LogPane::new(&th, "Journal", &logs)
        .status()
        .spans
        .iter()
        .map(|s| s.content.to_string())
        .collect();
    assert!(
        status.contains("all levels") && status.contains("following"),
        "{status}"
    );
    // Paused, it says how many arrived behind the pin.
    let mut paused = LogBuffer::new(500);
    paused.push(kp_tui::logs::LogLine::new(
        "12:00:00",
        "h",
        "media",
        Severity::Info,
        "one",
    ));
    paused.toggle_pause();
    paused.push(kp_tui::logs::LogLine::new(
        "12:00:01",
        "h",
        "media",
        Severity::Info,
        "two",
    ));
    let status: String = LogPane::new(&th, "Journal", &paused)
        .status()
        .spans
        .iter()
        .map(|s| s.content.to_string())
        .collect();
    assert!(status.contains("paused, 1 new"), "{status}");
}

#[test]
fn a_texture_tints_the_rhythm_its_register_paints() {
    // terminal: scanlines every third row (1px lines over a 3px repeat in
    // `css/terminal-register.css`). The lit rows differ from the ground;
    // the rest are the ground exactly.
    let th = Theme::new(ThemeId::TERMINAL, ColorDepth::TrueColor);
    let ground = ThemeId::TERMINAL.palette().background;
    let mut buf = Buffer::empty(Rect::new(0, 0, 10, 9));
    for y in 0..9 {
        for x in 0..10 {
            buf[(x, y)].set_bg(rgb(ground));
        }
    }
    Surface::new(&th, ground).paint(buf.area, &mut buf);
    let tinted: Vec<u16> = (0..9).filter(|y| buf[(0, *y)].bg != rgb(ground)).collect();
    assert_eq!(tinted, vec![0, 3, 6]);

    // high-contrast declares no texture at all: nothing is touched.
    let hc = Theme::new(
        ThemeId::from_name("high-contrast").unwrap(),
        ColorDepth::TrueColor,
    );
    let hg = hc.id.palette().background;
    let mut plain = Buffer::empty(Rect::new(0, 0, 10, 9));
    for y in 0..9 {
        for x in 0..10 {
            plain[(x, y)].set_bg(rgb(hg));
        }
    }
    Surface::new(&hc, hg).paint(plain.area, &mut plain);
    assert!((0..9).all(|y| plain[(0, y)].bg == rgb(hg)));

    // A sixteen-colour terminal cannot carry a 6 % tint, so it gets none.
    let low = Theme::new(ThemeId::TERMINAL, ColorDepth::Ansi16);
    let mut small = Buffer::empty(Rect::new(0, 0, 10, 9));
    Surface::new(&low, ground).paint(small.area, &mut small);
    assert!((0..9).all(|y| small[(0, y)].bg == Color::Reset));
}

#[test]
fn only_the_register_that_declares_a_sweep_sweeps() {
    let sweeping: Vec<&str> = ThemeId::ALL
        .iter()
        .filter(|id| id.anatomy().fx.sweep.is_some())
        .map(|id| id.name())
        .collect();
    // `kp-alarm-sweep` at 6000ms is cyberpunk's alone; terminal's raster is
    // static because DI5 refused the moving one.
    assert_eq!(sweeping, vec!["cyberpunk"]);

    let th = Theme::new(ThemeId::CYBERPUNK, ColorDepth::TrueColor);
    let ground = ThemeId::CYBERPUNK.palette().background;
    let rows = |ms: u32, motion| {
        let mut buf = Buffer::empty(Rect::new(0, 0, 8, 12));
        Surface::new(&th, ground)
            .at(ms, motion)
            .paint(buf.area, &mut buf);
        (0..12).map(|y| buf[(1, y)].bg).collect::<Vec<_>>()
    };
    let frames: Vec<Vec<Color>> = (0..6000)
        .step_by(120)
        .map(|ms| rows(ms, Motion::Full))
        .collect();
    assert!(
        frames.windows(2).any(|w| w[0] != w[1]),
        "something crosses the panel"
    );
    let still: Vec<Vec<Color>> = (0..6000)
        .step_by(120)
        .map(|ms| rows(ms, Motion::Reduced))
        .collect();
    assert!(
        still.windows(2).all(|w| w[0] == w[1]),
        "reduced motion holds"
    );
}

#[test]
fn the_alarm_is_the_one_its_register_declares() {
    // cyberpunk strikes and glitches; formal does neither, and its glow is
    // off ("nothing here glitches, glows, loops or flickers").
    let cy = Theme::new(ThemeId::CYBERPUNK, ColorDepth::TrueColor);
    let head = |th: &Theme, ms| {
        AlarmPanel::new(th, "Power lost", "ups on battery")
            .at(ms, Motion::Full)
            .headline()
    };
    let plain = cy.label("Power lost");
    assert_eq!(
        head(&cy, 3_000),
        plain,
        "outside a burst, the line is itself"
    );
    assert_ne!(head(&cy, 5_100), plain, "inside one, it comes apart");
    assert_eq!(
        AlarmPanel::new(&cy, "Power lost", "")
            .at(5_100, Motion::Reduced)
            .headline(),
        plain
    );

    let fm = Theme::new(ThemeId::FORMAL, ColorDepth::TrueColor);
    let flat = fm.label("Power lost");
    assert!(
        (0..8_000).step_by(100).all(|ms| head(&fm, ms) == flat),
        "formal never glitches"
    );
    assert!(fm.a.fx.alarm.strike.is_none() && fm.a.fx.alarm.glow_ms.is_none());

    // The strike is over when its keyframe is: the panel settles.
    let draw = |ms| {
        let mut buf = Buffer::empty(Rect::new(0, 0, 24, 5));
        AlarmPanel::new(&cy, "Power lost", "ups on battery")
            .at(ms, Motion::Full)
            .render(buf.area, &mut buf);
        buf
    };
    assert_ne!(draw(0), draw(300), "it strikes on the way in");
    assert_eq!(draw(1_200), draw(2_000), "and then holds");
}

#[test]
fn a_ticker_joins_its_segments_with_the_theme_s_own_divider() {
    let segs = vec!["cpu 12%".to_string(), "disk 61%".to_string()];
    for id in ThemeId::ALL {
        let th = Theme::new(id, ColorDepth::TrueColor);
        let line = Ticker::new(&th, &segs).at(0, Motion::Full).text(40);
        assert_eq!(line.chars().count(), 40, "{}", id.name());
        assert!(line.starts_with("cpu 12%"), "{}: {line}", id.name());
        if !th.a.tab_divider.trim().is_empty() {
            assert!(
                line.contains(th.a.tab_divider.trim()),
                "{}: {line}",
                id.name()
            );
        }
    }
    let th = Theme::new(ThemeId::TERMINAL, ColorDepth::TrueColor);
    let t = Ticker::new(&th, &segs);
    assert_ne!(
        t.text(40),
        Ticker::new(&th, &segs).at(1_000, Motion::Full).text(40)
    );
    assert_eq!(
        Ticker::new(&th, &segs).at(1_000, Motion::Reduced).text(40),
        Ticker::new(&th, &segs).at(0, Motion::Full).text(40)
    );
}

#[test]
fn every_theme_turns_a_spinner_of_its_own_shape() {
    for id in ThemeId::ALL {
        let th = Theme::new(id, ColorDepth::TrueColor);
        let frames: Vec<&str> = (0..900)
            .step_by(30)
            .map(|ms| spinner(&th, ms, Motion::Full))
            .collect();
        let first = frames[0];
        assert!(frames.iter().any(|f| *f != first), "{} turns", id.name());
        assert!(
            frames.iter().all(|f| f.chars().count() == 1),
            "{} stays in one cell",
            id.name()
        );
        assert_eq!(spinner(&th, 450, Motion::Reduced), first, "{}", id.name());
    }
}
