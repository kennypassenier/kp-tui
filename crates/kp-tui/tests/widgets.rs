//! The widgets, headless: a `TestBackend` buffer per case, no application
//! and no terminal. These moved over from the demo in kp-themes with the
//! widgets they measure [kp-themes scope-129] — the ones that drive the
//! demo's own key handling stayed there, because that is the demo's, not
//! this crate's.

use kp_tui::{
    ColorDepth, Theme, ThemeId,
    anatomy::Reveal,
    color::Rgb,
    dashboard::rate,
    fx::{self, Motion},
    live, logs,
    logs::Severity,
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
