//! The live dashboard: machine stats from `/proc`, two scrolling charts,
//! per-core bars, sparklines, and the journal streaming underneath. Every
//! colour comes from `&Theme`, so the theme key repaints all of it on the
//! next frame.
//!
//! The state here is plain data. `main.rs` feeds it samples and log lines;
//! tests feed it fixed ones and draw with `TestBackend`.

use std::collections::VecDeque;

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Margin, Rect},
    style::{Modifier, Style},
    symbols::Marker,
    text::{Line, Span},
    widgets::{
        Axis, Bar, BarChart, BarGroup, Block, Chart, Dataset, GraphType, LegendPosition, Paragraph,
        Sparkline,
    },
};

use crate::color::{ColorDepth, Role};
use crate::fx::Motion;
use crate::live::Sample;
use crate::logs::{LogBuffer, Severity};
use crate::theme::Theme;
use crate::widgets::{Panel, ThemedTabs, mix};

/// How much history the charts show.
pub const WINDOW_S: f64 = 60.0;
/// A value crossing its threshold upwards pulses for this long.
pub const PULSE_MS: u32 = 1200;
/// The delay between one panel title's reveal and the next.
pub const STAGGER_MS: u32 = 90;

pub const SCREENS: [&str; 2] = ["Dashboard", "Components"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Metric {
    Cpu,
    Memory,
    Load,
}

impl Metric {
    /// The value, and whether it is at or over its threshold: CPU 70 %,
    /// memory 85 %, the one-minute load at one per core.
    fn hot(self, s: &Sample) -> (f64, bool) {
        match self {
            Metric::Cpu => (s.cpu_total, s.cpu_total >= 70.0),
            Metric::Memory => (s.mem_used_pct, s.mem_used_pct >= 85.0),
            Metric::Load => {
                let per_core = s.load[0] / s.cores.len().max(1) as f64;
                (per_core, per_core >= 1.0)
            }
        }
    }
}

pub struct Dashboard {
    /// `(seconds since start, sample)`, oldest first.
    pub history: VecDeque<(f64, Sample)>,
    pub now_s: f64,
    pub logs: LogBuffer,
    /// What the log pane's title says the feed is.
    pub feed_label: String,
    pub feed_live: bool,
    /// Started pulses: which value, and at which `clock_ms`.
    pub pulses: Vec<(Metric, u32)>,
    pub clock_ms: u32,
    /// Mean time of a `terminal.draw`, measured by the loop.
    pub draw_ms: f32,
    pub fps: u32,
}

impl Default for Dashboard {
    fn default() -> Self {
        Dashboard {
            history: VecDeque::new(),
            now_s: 0.0,
            logs: LogBuffer::default(),
            feed_label: "no feed".into(),
            feed_live: false,
            pulses: Vec::new(),
            clock_ms: 0,
            draw_ms: 0.0,
            fps: 15,
        }
    }
}

impl Dashboard {
    pub fn latest(&self) -> Option<&Sample> {
        self.history.back().map(|(_, s)| s)
    }

    pub fn push_sample(&mut self, t: f64, s: Sample) {
        if let Some((_, prev)) = self.history.back() {
            for m in [Metric::Cpu, Metric::Memory, Metric::Load] {
                if !m.hot(prev).1 && m.hot(&s).1 {
                    self.pulses.retain(|(p, _)| *p != m);
                    self.pulses.push((m, self.clock_ms));
                }
            }
        }
        self.now_s = t;
        self.history.push_back((t, s));
        while self
            .history
            .front()
            .is_some_and(|(t0, _)| *t0 < t - WINDOW_S - 1.0)
        {
            self.history.pop_front();
        }
    }

    pub fn tick(&mut self, ms: u32) {
        self.clock_ms = self.clock_ms.saturating_add(ms);
        let now = self.clock_ms;
        self.pulses
            .retain(|(_, at)| now.saturating_sub(*at) < PULSE_MS);
    }

    /// 0.0 at rest, up to 1.0 at the top of a beat: two beats that fade.
    pub fn pulse(&self, m: Metric, motion: Motion) -> f32 {
        if motion == Motion::Reduced {
            return 0.0;
        }
        self.pulses
            .iter()
            .find(|(p, _)| *p == m)
            .map(|(_, at)| {
                let t = self.clock_ms.saturating_sub(*at) as f32 / PULSE_MS as f32;
                (1.0 - t).max(0.0) * (std::f32::consts::PI * 2.0 * t).sin().abs()
            })
            .unwrap_or(0.0)
    }
}

pub struct DashAreas {
    pub header: Rect,
    pub tabs: Rect,
    pub tiles: [Rect; 6],
    pub cpu: Rect,
    pub net: Rect,
    pub cores: Rect,
    pub memdisk: Rect,
    pub logs: Rect,
    pub footer: Rect,
}

pub fn dash_areas(area: Rect) -> DashAreas {
    // Charts take two fifths of what the fixed rows leave, at least nine
    // rows; the log pane takes the rest.
    let fixed = 1 + 1 + 3 + 6 + 1;
    let body = area.height.saturating_sub(fixed);
    let charts = (body * 2 / 5).max(9).min(body);
    let [header, tabs, tiles, charts, bars, logs, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Length(charts),
        Constraint::Length(6),
        Constraint::Min(0),
        Constraint::Length(1),
    ])
    .areas(area);
    let tiles = Layout::horizontal([Constraint::Ratio(1, 6); 6]).areas(tiles);
    let [cpu, net] = Layout::horizontal([Constraint::Percentage(50); 2]).areas(charts);
    let [cores, memdisk] = Layout::horizontal([Constraint::Percentage(50); 2]).areas(bars);
    DashAreas {
        header,
        tabs,
        tiles,
        cpu,
        net,
        cores,
        memdisk,
        logs,
        footer,
    }
}

/// Bytes per second in four cells or so: `812B`, `1.2K`, `34M`.
pub fn rate(bps: f64) -> String {
    const UNITS: [&str; 5] = ["B", "K", "M", "G", "T"];
    let mut v = bps.max(0.0);
    let mut i = 0;
    while v >= 1000.0 && i < UNITS.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    if v < 10.0 && i > 0 {
        format!("{v:.1}{}", UNITS[i])
    } else {
        format!("{v:.0}{}", UNITS[i])
    }
}

/// The level's look. The tag text carries the level on its own, so 16
/// colours (where two roles can land on the same hue) stay readable;
/// error and critical are also bold, and critical sits on a plate.
pub fn severity_style(th: &Theme, s: Severity) -> Style {
    let c = &th.c;
    match s {
        Severity::Debug => Style::new().fg(c.muted_foreground),
        Severity::Info => Style::new().fg(c.info_foreground),
        Severity::Notice => Style::new().fg(c.success_foreground),
        Severity::Warning => Style::new()
            .fg(c.warning_foreground)
            .add_modifier(Modifier::BOLD),
        Severity::Error => Style::new().fg(c.destructive).add_modifier(Modifier::BOLD),
        Severity::Critical => Style::new()
            .bg(c.destructive)
            .fg(c.destructive_foreground)
            .add_modifier(Modifier::BOLD),
    }
}

fn message_style(th: &Theme, s: Severity) -> Style {
    let c = &th.c;
    match s {
        Severity::Debug => Style::new().fg(c.muted_foreground),
        Severity::Info | Severity::Notice => Style::new().fg(c.card_foreground),
        Severity::Warning => Style::new().fg(c.warning_foreground),
        Severity::Error | Severity::Critical => Style::new().fg(c.destructive),
    }
}

pub struct View<'a> {
    pub theme: &'a Theme,
    pub motion: Motion,
    pub reveal_ms: u32,
    pub header: String,
}

fn panel<'a>(v: &'a View, title: &'a str, index: u32) -> Panel<'a> {
    Panel::new(v.theme, title).reveal(v.reveal_ms.saturating_sub(index * STAGGER_MS), v.motion)
}

pub fn draw(frame: &mut Frame, d: &Dashboard, v: &View) {
    let th = v.theme;
    let c = &th.c;
    let area = frame.area();
    frame.render_widget(Block::new().style(th.base()), area);
    let a = dash_areas(area);

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                th.label("kp-themes"),
                Style::new().fg(c.primary).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  {}", v.header),
                Style::new().fg(c.muted_foreground),
            ),
        ])),
        a.header,
    );
    frame.render_widget(ThemedTabs::new(th, &SCREENS, 0), a.tabs);

    draw_tiles(frame, d, v, &a);
    draw_cpu(frame, d, v, a.cpu);
    draw_net(frame, d, v, a.net);
    draw_cores(frame, d, v, a.cores);
    draw_memdisk(frame, d, v, a.memdisk);
    draw_logs(frame, d, v, a.logs);

    let keys = " s screen · t theme · p pause · f filter · ↑/↓ PgUp/PgDn scroll · End follow · m motion · r replay · q quit";
    frame.render_widget(
        Paragraph::new(keys).style(Style::new().bg(c.secondary).fg(c.secondary_foreground)),
        a.footer,
    );
}

fn inner(p: &Panel, area: Rect) -> Rect {
    p.block().inner(area)
}

fn draw_tiles(frame: &mut Frame, d: &Dashboard, v: &View, a: &DashAreas) {
    let th = v.theme;
    let c = &th.c;
    let muted = Style::new().fg(c.muted_foreground);
    let s = d.latest();
    let hot_or = |m: Metric, normal| match s.map(|s| m.hot(s).1) {
        Some(true) => c.warning_foreground,
        _ => normal,
    };
    // A pulse lights the value's ground with the soft warning plate and
    // lets it fall back; 16 colours cannot mix, so it flips to inverse.
    let pulsed = |m: Metric, style: Style| {
        let k = d.pulse(m, v.motion);
        if k <= 0.0 {
            return style;
        }
        match th.depth {
            ColorDepth::Ansi16 if k > 0.5 => style.add_modifier(Modifier::REVERSED),
            ColorDepth::Ansi16 => style,
            depth => {
                let p = th.id.palette();
                style.bg(depth.resolve(Role::Surface, mix(p.card, p.warning, k)))
            }
        }
    };
    let waiting = || vec![Span::styled("reading…", muted)];
    let lines: [(&str, Vec<Span>); 6] = [
        (
            "CPU",
            s.map(|s| {
                let st = pulsed(
                    Metric::Cpu,
                    Style::new()
                        .fg(hot_or(Metric::Cpu, c.chart_1))
                        .add_modifier(Modifier::BOLD),
                );
                vec![Span::styled(format!(" {:.1} % ", s.cpu_total), st)]
            })
            .unwrap_or_else(waiting),
        ),
        (
            "Memory",
            s.map(|s| {
                let st = pulsed(
                    Metric::Memory,
                    Style::new()
                        .fg(hot_or(Metric::Memory, c.chart_2))
                        .add_modifier(Modifier::BOLD),
                );
                vec![
                    Span::styled(format!(" {:.0} % ", s.mem_used_pct), st),
                    Span::styled(
                        format!("{:.1}G", s.mem_used_bytes as f64 / (1u64 << 30) as f64),
                        muted,
                    ),
                ]
            })
            .unwrap_or_else(waiting),
        ),
        (
            "Load",
            s.map(|s| {
                let st = pulsed(
                    Metric::Load,
                    Style::new().fg(hot_or(Metric::Load, c.card_foreground)),
                );
                vec![Span::styled(
                    format!(" {:.2} {:.2} {:.2} ", s.load[0], s.load[1], s.load[2]),
                    st,
                )]
            })
            .unwrap_or_else(waiting),
        ),
        (
            "Network",
            s.map(|s| {
                vec![
                    Span::styled(format!("↓{} ", rate(s.rx_bps)), Style::new().fg(c.chart_3)),
                    Span::styled(format!("↑{}", rate(s.tx_bps)), Style::new().fg(c.chart_4)),
                ]
            })
            .unwrap_or_else(waiting),
        ),
        (
            "Disk",
            s.map(|s| {
                vec![
                    Span::styled("r ", muted),
                    Span::styled(
                        format!("{} ", rate(s.disk_read_bps)),
                        Style::new().fg(c.chart_5),
                    ),
                    Span::styled("w ", muted),
                    Span::styled(rate(s.disk_write_bps), Style::new().fg(c.chart_5)),
                ]
            })
            .unwrap_or_else(waiting),
        ),
        (
            "This demo",
            s.map(|_| {
                // Clock ticks are 10 ms, so one half-second sample reads 0 or
                // 2 %; the mean over the window is the honest figure.
                let own = d.history.iter().map(|(_, s)| s.self_cpu_pct).sum::<f64>()
                    / d.history.len() as f64;
                vec![
                    Span::styled(format!("{own:.1} % "), Style::new().fg(c.card_foreground)),
                    Span::styled(format!("{:.1}ms", d.draw_ms), muted),
                ]
            })
            .unwrap_or_else(waiting),
        ),
    ];
    for (i, ((title, spans), area)) in lines.into_iter().zip(a.tiles).enumerate() {
        let p = panel(v, title, i as u32);
        let inner = inner(&p, area);
        frame.render_widget(p, area);
        frame.render_widget(
            Paragraph::new(Line::from(spans)).alignment(Alignment::Center),
            inner,
        );
    }
}

fn axis_labels(th: &Theme, labels: [String; 3]) -> Vec<Line<'static>> {
    labels
        .into_iter()
        .map(|l| Line::from(Span::styled(l, Style::new().fg(th.c.muted_foreground))))
        .collect()
}

fn time_axis(th: &Theme) -> Axis<'static> {
    Axis::default()
        .bounds([0.0, WINDOW_S])
        .labels(axis_labels(
            th,
            ["-60s".into(), "-30s".into(), "now".into()],
        ))
        .style(Style::new().fg(th.c.border_strong))
}

/// Points as `(x, y)` with x = 0 at the left edge of the window.
fn series(d: &Dashboard, f: impl Fn(&Sample) -> f64) -> Vec<(f64, f64)> {
    d.history
        .iter()
        .map(|(t, s)| (t - d.now_s + WINDOW_S, f(s)))
        .filter(|(x, _)| *x >= 0.0)
        .collect()
}

fn legend_chart<'a>(th: &Theme, datasets: Vec<Dataset<'a>>, y: Axis<'a>) -> Chart<'a> {
    Chart::new(datasets)
        .x_axis(time_axis(th))
        .y_axis(y)
        .style(Style::new().bg(th.c.card).fg(th.c.card_foreground))
        .legend_position(Some(LegendPosition::TopLeft))
        .hidden_legend_constraints((Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)))
}

fn draw_cpu(frame: &mut Frame, d: &Dashboard, v: &View, area: Rect) {
    let th = v.theme;
    let p = panel(v, "CPU · last 60 s", 6);
    let inner = inner(&p, area);
    frame.render_widget(p, area);
    let cores = d.latest().map(|s| s.cores.len()).unwrap_or(1).max(1) as f64;
    let total = series(d, |s| s.cpu_total);
    let load = series(d, |s| (100.0 * s.load[0] / cores).min(100.0));
    let datasets = vec![
        Dataset::default()
            .name("total %")
            .marker(Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::new().fg(th.c.chart_1))
            .data(&total),
        Dataset::default()
            .name("load per core")
            .marker(Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::new().fg(th.c.chart_5))
            .data(&load),
    ];
    let y = Axis::default()
        .bounds([0.0, 100.0])
        .labels(axis_labels(th, ["0%".into(), "50%".into(), "100%".into()]))
        .style(Style::new().fg(th.c.border_strong));
    frame.render_widget(
        legend_chart(th, datasets, y),
        inner.inner(Margin::new(1, 0)),
    );
}

fn draw_net(frame: &mut Frame, d: &Dashboard, v: &View, area: Rect) {
    let th = v.theme;
    let p = panel(v, "Network · last 60 s", 7);
    let inner = inner(&p, area);
    frame.render_widget(p, area);
    let rx = series(d, |s| s.rx_bps);
    let tx = series(d, |s| s.tx_bps);
    // A round ceiling over the window: 1, 2 or 5 times a power of 1024-ish
    // steps, so the labels stay short.
    let peak = rx.iter().chain(&tx).map(|(_, y)| *y).fold(1024.0, f64::max);
    let mut top = 1024.0;
    'find: for scale in [1.0, 1024.0, 1024.0 * 1024.0, 1024.0 * 1024.0 * 1024.0] {
        for step in [1.0, 2.0, 5.0, 10.0, 20.0, 50.0, 100.0, 200.0, 500.0] {
            if step * scale >= peak {
                top = step * scale;
                break 'find;
            }
        }
    }
    let datasets = vec![
        Dataset::default()
            .name("↓ received")
            .marker(Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::new().fg(th.c.chart_3))
            .data(&rx),
        Dataset::default()
            .name("↑ sent")
            .marker(Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::new().fg(th.c.chart_4))
            .data(&tx),
    ];
    let y = Axis::default()
        .bounds([0.0, top])
        .labels(axis_labels(
            th,
            [
                "0".into(),
                format!("{}/s", rate(top / 2.0)),
                format!("{}/s", rate(top)),
            ],
        ))
        .style(Style::new().fg(th.c.border_strong));
    frame.render_widget(
        legend_chart(th, datasets, y),
        inner.inner(Margin::new(1, 0)),
    );
}

fn draw_cores(frame: &mut Frame, d: &Dashboard, v: &View, area: Rect) {
    let th = v.theme;
    let c = &th.c;
    let cores: Vec<f64> = d.latest().map(|s| s.cores.clone()).unwrap_or_default();
    let busiest = cores.iter().copied().fold(0.0, f64::max);
    let title = format!("Cores · {} · busiest {busiest:.0} %", cores.len());
    let p = panel(v, &title, 8);
    let inner = inner(&p, area).inner(Margin::new(1, 0));
    frame.render_widget(p, area);
    if cores.is_empty() {
        return;
    }
    let bars: Vec<Bar> = cores
        .iter()
        .map(|&pct| {
            let fg = if pct >= 90.0 {
                c.destructive
            } else if pct >= 70.0 {
                c.warning_foreground
            } else {
                c.chart_1
            };
            Bar::default()
                .value(pct.round() as u64)
                .text_value(String::new())
                .style(Style::new().fg(fg))
        })
        .collect();
    let n = cores.len() as u16;
    let width = ((inner.width + 1) / n).saturating_sub(1).max(1);
    let chart = BarChart::default()
        .data(BarGroup::default().bars(&bars))
        .bar_width(width)
        .bar_gap(1)
        .max(100)
        .style(Style::new().bg(c.card));
    frame.render_widget(chart, inner);
}

fn draw_memdisk(frame: &mut Frame, d: &Dashboard, v: &View, area: Rect) {
    let th = v.theme;
    let c = &th.c;
    let muted = Style::new().fg(c.muted_foreground);
    let p = panel(v, "Memory · Disk", 9);
    let inner = inner(&p, area).inner(Margin::new(1, 0));
    frame.render_widget(p, area);
    let [mem_line, mem_spark, disk_line, disk_spark] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .areas(inner);
    let take = |w: u16, f: &dyn Fn(&Sample) -> u64| -> Vec<u64> {
        let n = d.history.len().saturating_sub(w as usize);
        d.history.iter().skip(n).map(|(_, s)| f(s)).collect()
    };
    if let Some(s) = d.latest() {
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("used ", muted),
                Span::styled(
                    format!("{:.1} %", s.mem_used_pct),
                    Style::new().fg(c.chart_2),
                ),
                Span::styled(
                    format!(
                        " · {:.1} of {:.1} GiB",
                        s.mem_used_bytes as f64 / (1u64 << 30) as f64,
                        s.mem_total_bytes as f64 / (1u64 << 30) as f64
                    ),
                    muted,
                ),
            ])),
            mem_line,
        );
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("read ", muted),
                Span::styled(
                    format!("{}/s", rate(s.disk_read_bps)),
                    Style::new().fg(c.chart_5),
                ),
                Span::styled(" · write ", muted),
                Span::styled(
                    format!("{}/s", rate(s.disk_write_bps)),
                    Style::new().fg(c.chart_5),
                ),
            ])),
            disk_line,
        );
    }
    let mem = take(mem_spark.width, &|s| (s.mem_used_pct * 10.0) as u64);
    frame.render_widget(
        Sparkline::default()
            .data(&mem)
            .max(1000)
            .style(Style::new().fg(c.chart_2).bg(c.card)),
        mem_spark,
    );
    let [r, w] = Layout::horizontal([Constraint::Percentage(50); 2])
        .spacing(1)
        .areas(disk_spark);
    let reads = take(r.width, &|s| s.disk_read_bps as u64);
    let writes = take(w.width, &|s| s.disk_write_bps as u64);
    for (data, rect) in [(reads, r), (writes, w)] {
        frame.render_widget(
            Sparkline::default()
                .data(&data)
                .style(Style::new().fg(c.chart_5).bg(c.card)),
            rect,
        );
    }
}

fn draw_logs(frame: &mut Frame, d: &Dashboard, v: &View, area: Rect) {
    let th = v.theme;
    let c = &th.c;
    let muted = Style::new().fg(c.muted_foreground);
    let l = &d.logs;
    let filter = if l.filter == Severity::Debug {
        "all levels".to_string()
    } else {
        format!("{} and up", l.filter.name())
    };
    let state = if l.paused() {
        format!("paused, {} new", l.unseen())
    } else {
        "following".to_string()
    };
    let status = Line::from(vec![
        Span::styled(format!(" {filter} · "), muted),
        Span::styled(
            state,
            if l.paused() {
                Style::new()
                    .fg(c.warning_foreground)
                    .add_modifier(Modifier::BOLD)
            } else {
                muted
            },
        ),
        Span::styled(" ", muted),
    ]);
    let title = if d.feed_live {
        "Journal · live · read-only"
    } else {
        "Log stream · synthetic, not real"
    };
    let p = panel(v, title, 10).status(status);
    let inner = inner(&p, area).inner(Margin::new(1, 0));
    frame.render_widget(p, area);
    if !d.feed_live && !d.feed_label.is_empty() && l.is_empty() {
        frame.render_widget(
            Paragraph::new(Span::styled(d.feed_label.clone(), muted)),
            inner,
        );
        return;
    }
    let lines: Vec<Line> = l
        .visible(inner.height as usize)
        .into_iter()
        .map(|line| {
            Line::from(vec![
                Span::styled(format!("{} ", line.time), muted),
                Span::styled(format!("{} ", line.host), Style::new().fg(c.border_strong)),
                Span::styled(format!("{} ", line.unit), Style::new().fg(c.primary)),
                Span::styled(line.severity.tag(), severity_style(th, line.severity)),
                Span::styled(" ", Style::new()),
                Span::styled(line.message.clone(), message_style(th, line.severity)),
            ])
        })
        .collect();
    frame.render_widget(Paragraph::new(lines).style(Style::new().bg(c.card)), inner);
}
