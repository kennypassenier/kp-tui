//! The log pane's model: severities, a bounded buffer with pause, scroll
//! and filter, and two feeds. The live feed tails the system journal
//! read-only through a spawned `journalctl`; the synthetic feed is used
//! only when that is not readable (or `--synthetic-logs` asks for it), and
//! the pane says so in its title.

use std::{
    collections::VecDeque,
    io::{BufRead, BufReader},
    process::{Child, Command, Stdio},
    sync::mpsc::{self, Receiver, TryRecvError},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

/// syslog priorities as the journal stores them. 0 (emerg) and 1 (alert)
/// fold into `Critical`: six levels are what the pane distinguishes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    Critical = 2,
    Error = 3,
    Warning = 4,
    Notice = 5,
    Info = 6,
    Debug = 7,
}

impl Severity {
    pub const ALL: [Severity; 6] = [
        Severity::Debug,
        Severity::Info,
        Severity::Notice,
        Severity::Warning,
        Severity::Error,
        Severity::Critical,
    ];

    pub fn from_priority(p: u8) -> Self {
        match p {
            0..=2 => Severity::Critical,
            3 => Severity::Error,
            4 => Severity::Warning,
            5 => Severity::Notice,
            6 => Severity::Info,
            _ => Severity::Debug,
        }
    }

    /// Fixed width, so the level is readable without any colour at all.
    pub fn tag(self) -> &'static str {
        match self {
            Severity::Debug => "DEBUG ",
            Severity::Info => "INFO  ",
            Severity::Notice => "NOTICE",
            Severity::Warning => "WARN  ",
            Severity::Error => "ERROR ",
            Severity::Critical => "CRIT  ",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Severity::Debug => "debug",
            Severity::Info => "info",
            Severity::Notice => "notice",
            Severity::Warning => "warning",
            Severity::Error => "error",
            Severity::Critical => "critical",
        }
    }

    /// The next stricter filter, wrapping back to "everything".
    pub fn stricter(self) -> Self {
        let i = Self::ALL.iter().position(|s| *s == self).unwrap();
        Self::ALL[(i + 1) % Self::ALL.len()]
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogLine {
    pub seq: u64,
    /// `HH:MM:SS.mmm`, local time.
    pub time: String,
    pub host: String,
    /// `SYSLOG_IDENTIFIER[_PID]` or the command name.
    pub unit: String,
    pub severity: Severity,
    pub message: String,
}

impl LogLine {
    pub fn new(time: &str, host: &str, unit: &str, severity: Severity, message: &str) -> Self {
        LogLine {
            seq: 0,
            time: time.into(),
            host: host.into(),
            unit: unit.into(),
            severity,
            message: message.into(),
        }
    }
}

/// `HH:MM:SS.mmm` from microseconds since the epoch and a UTC offset.
impl LogLine {
    /// The line's own timestamp in milliseconds past midnight, read back
    /// out of the `HH:MM:SS.mmm` string it was handed. `None` when the
    /// feed wrote something else.
    pub fn at_ms(&self) -> Option<u64> {
        let (hms, ms) = self.time.split_once('.')?;
        let mut parts = hms.split(':');
        let h: u64 = parts.next()?.parse().ok()?;
        let m: u64 = parts.next()?.parse().ok()?;
        let s: u64 = parts.next()?.parse().ok()?;
        if parts.next().is_some() {
            return None;
        }
        Some(((h * 60 + m) * 60 + s) * 1000 + ms.parse::<u64>().ok()?)
    }
}

pub fn clock(micros: u64, offset_secs: i64) -> String {
    let secs = (micros / 1_000_000) as i64 + offset_secs;
    let day = secs.rem_euclid(86_400);
    format!(
        "{:02}:{:02}:{:02}.{:03}",
        day / 3600,
        day / 60 % 60,
        day % 60,
        micros / 1000 % 1000
    )
}

/// The machine's UTC offset, asked once of `date +%z` (`+0200`). The demo
/// has no time-zone crate; 0 when `date` is missing.
pub fn local_offset_secs() -> i64 {
    let Ok(out) = Command::new("date")
        .arg("+%z")
        .stderr(Stdio::null())
        .output()
    else {
        return 0;
    };
    let s = String::from_utf8_lossy(&out.stdout);
    let s = s.trim();
    if s.len() != 5 {
        return 0;
    }
    let sign = if s.starts_with('-') { -1 } else { 1 };
    let (h, m) = (
        s[1..3].parse::<i64>().unwrap_or(0),
        s[3..5].parse::<i64>().unwrap_or(0),
    );
    sign * (h * 3600 + m * 60)
}

/// Terminal cells must not receive control characters from a log message.
fn clean(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect()
}

/// One line of `journalctl -o json`. `MESSAGE` is a byte array when the
/// journal holds non-UTF-8; it is decoded lossily.
pub fn parse_journal_json(line: &str, offset_secs: i64) -> Option<LogLine> {
    let v: serde_json::Value = serde_json::from_str(line).ok()?;
    let text = |k: &str| -> Option<String> {
        match &v[k] {
            serde_json::Value::String(s) => Some(s.clone()),
            serde_json::Value::Array(a) => {
                let bytes: Vec<u8> = a
                    .iter()
                    .filter_map(|b| b.as_u64().map(|b| b as u8))
                    .collect();
                Some(String::from_utf8_lossy(&bytes).into_owned())
            }
            _ => None,
        }
    };
    let micros = text("__REALTIME_TIMESTAMP")?.parse::<u64>().ok()?;
    let severity =
        Severity::from_priority(text("PRIORITY").and_then(|p| p.parse().ok()).unwrap_or(6));
    let ident = text("SYSLOG_IDENTIFIER")
        .or_else(|| text("_COMM"))
        .unwrap_or_else(|| "?".into());
    let unit = match text("_PID") {
        Some(pid) => format!("{ident}[{pid}]"),
        None => ident,
    };
    Some(LogLine {
        seq: 0,
        time: clock(micros, offset_secs),
        host: clean(&text("_HOSTNAME").unwrap_or_default()),
        unit: clean(&unit),
        severity,
        message: clean(&text("MESSAGE").unwrap_or_default()),
    })
}

// ── Buffer ──────────────────────────────────────────────────────────────

/// A bounded buffer the pane reads. Pausing pins the view to the last line
/// seen; lines keep arriving underneath and are counted.
#[derive(Clone, Debug)]
pub struct LogBuffer {
    lines: VecDeque<LogLine>,
    cap: usize,
    next_seq: u64,
    /// The newest `seq` the view shows; `None` follows the stream.
    pin: Option<u64>,
    /// Lines hidden below the view, counted in filtered lines.
    scroll: usize,
    /// Show this severity and everything more severe.
    pub filter: Severity,
    /// Show only this unit; `None` shows every one of them. homelab's
    /// selector really filters, and a selector that only paints itself is
    /// a lost feature, not a simpler one [fix-65].
    source: Option<String>,
}

impl Default for LogBuffer {
    fn default() -> Self {
        Self::new(4000)
    }
}

impl LogBuffer {
    pub fn new(cap: usize) -> Self {
        LogBuffer {
            lines: VecDeque::new(),
            cap,
            next_seq: 1,
            pin: None,
            scroll: 0,
            filter: Severity::Debug,
            source: None,
        }
    }

    /// The unit the selector points at, or `None` for all of them.
    pub fn source(&self) -> Option<&str> {
        self.source.as_deref()
    }

    /// Point the selector at one unit, or at all of them. The view drops
    /// back to the tail, because a scroll offset counted in the old
    /// selection means nothing in the new one.
    pub fn select_source(&mut self, unit: Option<&str>) {
        self.source = unit.map(str::to_string);
        self.scroll = 0;
    }

    pub fn push(&mut self, mut line: LogLine) {
        line.seq = self.next_seq;
        self.next_seq += 1;
        if self.lines.len() == self.cap {
            self.lines.pop_front();
        }
        self.lines.push_back(line);
    }

    pub fn len(&self) -> usize {
        self.lines.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    pub fn paused(&self) -> bool {
        self.pin.is_some()
    }

    fn last_seq(&self) -> u64 {
        self.next_seq - 1
    }

    pub fn toggle_pause(&mut self) {
        self.pin = if self.pin.is_some() {
            None
        } else {
            Some(self.last_seq())
        };
        self.scroll = 0;
    }

    pub fn follow(&mut self) {
        self.pin = None;
        self.scroll = 0;
    }

    /// Scrolling back pins the view, as `less +F` does.
    pub fn scroll_up(&mut self, n: usize) {
        self.pin.get_or_insert(self.last_seq());
        self.scroll = self.scroll.saturating_add(n).min(self.shown().count());
    }

    /// Scrolling forward to the last line lets go of the pin again, the
    /// way homelab's log tab does: the tail is where following resumes,
    /// and asking for it twice would be a keystroke nobody presses
    /// [fix-65].
    pub fn scroll_down(&mut self, n: usize) {
        self.scroll = self.scroll.saturating_sub(n);
        if self.scroll == 0 {
            self.pin = None;
        }
    }

    pub fn cycle_filter(&mut self) {
        self.filter = self.filter.stricter();
        self.scroll = 0;
    }

    fn shown(&self) -> impl DoubleEndedIterator<Item = &LogLine> {
        let pin = self.pin.unwrap_or(u64::MAX);
        let source = self.source.as_deref();
        self.lines.iter().filter(move |l| {
            l.severity <= self.filter && l.seq <= pin && source.is_none_or(|s| l.unit == s)
        })
    }

    /// Lines that arrived after the pin, whatever the filter.
    pub fn unseen(&self) -> u64 {
        self.pin.map(|p| self.last_seq() - p).unwrap_or(0)
    }

    /// How many lines the filter and the pin leave visible.
    pub fn shown_count(&self) -> usize {
        self.shown().count()
    }

    /// How far back the view is scrolled, in lines.
    pub fn scroll(&self) -> usize {
        self.scroll
    }

    /// The shown lines split into `n` buckets of equal *time*, oldest
    /// first: how many landed in each, and how many of those were an
    /// error or worse.
    ///
    /// By the clock and not by position, because by position every bucket
    /// holds the same number of lines by construction — a band drawn that
    /// way is flat whatever the feed did. Lines whose timestamp does not
    /// parse fall back to their place in the buffer.
    pub fn buckets(&self, n: usize) -> (Vec<u32>, Vec<u32>) {
        let lines: Vec<&LogLine> = self.shown().collect();
        if n == 0 || lines.is_empty() {
            return (Vec::new(), Vec::new());
        }
        let mut counts = vec![0u32; n];
        let mut errors = vec![0u32; n];
        let stamps: Vec<Option<u64>> = lines.iter().map(|l| l.at_ms()).collect();
        let first = stamps.iter().flatten().min().copied();
        let last = stamps.iter().flatten().max().copied();
        let span = match (first, last) {
            (Some(a), Some(b)) if b > a => Some((a, b - a)),
            _ => None,
        };
        for (i, line) in lines.iter().enumerate() {
            let b = match (span, stamps[i]) {
                (Some((from, width)), Some(at)) => {
                    (((at - from) as u128 * n as u128) / width as u128) as usize
                }
                _ => i * n / lines.len(),
            }
            .min(n - 1);
            counts[b] += 1;
            if line.severity <= Severity::Error {
                errors[b] += 1;
            }
        }
        (counts, errors)
    }

    /// How wide the host and the unit columns have to be for every line
    /// in the buffer to start its message in the same place [fix-64].
    /// Capped, so one long unit name cannot eat the row.
    pub fn columns(&self) -> (usize, usize) {
        let width = |f: fn(&LogLine) -> &str| {
            self.lines
                .iter()
                .map(|l| f(l).chars().count())
                .max()
                .unwrap_or(0)
                .min(16)
        };
        (width(|l| &l.host), width(|l| &l.unit))
    }

    /// The `height` lines the pane shows, oldest first.
    pub fn visible(&self, height: usize) -> Vec<&LogLine> {
        let total = self.shown().count();
        let scroll = self.scroll.min(total.saturating_sub(height));
        let mut v: Vec<&LogLine> = self.shown().rev().skip(scroll).take(height).collect();
        v.reverse();
        v
    }
}

// ── Feeds ───────────────────────────────────────────────────────────────

pub enum Feed {
    Journal {
        child: Child,
        rx: Receiver<LogLine>,
        got_any: bool,
        started: Instant,
    },
    Synthetic {
        next_at: Instant,
        state: u64,
        offset: i64,
        reason: &'static str,
    },
}

impl Feed {
    /// `journalctl --follow` as a child: read-only, no privileges asked
    /// for. What it shows is what this user may read.
    pub fn journal() -> std::io::Result<Feed> {
        let offset = local_offset_secs();
        let mut child = Command::new("journalctl")
            .args([
                "--follow",
                "--lines=300",
                "--output=json",
                "--output-fields=PRIORITY,SYSLOG_IDENTIFIER,_COMM,_PID,_HOSTNAME,MESSAGE",
                "--no-pager",
                "--quiet",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        let out = child.stdout.take().expect("piped");
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            for line in BufReader::new(out).lines().map_while(Result::ok) {
                if let Some(l) = parse_journal_json(&line, offset)
                    && tx.send(l).is_err()
                {
                    break;
                }
            }
        });
        Ok(Feed::Journal {
            child,
            rx,
            got_any: false,
            started: Instant::now(),
        })
    }

    pub fn synthetic(reason: &'static str) -> Feed {
        Feed::Synthetic {
            next_at: Instant::now(),
            state: 0x2545_F491_4F6C_DD1D,
            offset: local_offset_secs(),
            reason,
        }
    }

    pub fn live(&self) -> bool {
        matches!(self, Feed::Journal { .. })
    }

    pub fn child_pid(&self) -> Option<u32> {
        match self {
            Feed::Journal { child, .. } => Some(child.id()),
            Feed::Synthetic { .. } => None,
        }
    }

    pub fn label(&self) -> String {
        match self {
            Feed::Journal { .. } => "system journal · live · read-only".into(),
            Feed::Synthetic { reason, .. } => format!("synthetic stream, not real · {reason}"),
        }
    }

    /// Moves what arrived into `buf`, at most `max` lines per call so a
    /// burst cannot stall a frame. A journal that ends without a single
    /// line turns into the synthetic feed.
    pub fn drain(&mut self, buf: &mut LogBuffer, max: usize) {
        let mut unreadable = false;
        match self {
            Feed::Journal {
                rx,
                got_any,
                started,
                ..
            } => {
                for _ in 0..max {
                    match rx.try_recv() {
                        Ok(line) => {
                            *got_any = true;
                            buf.push(line);
                        }
                        Err(TryRecvError::Empty) => break,
                        Err(TryRecvError::Disconnected) => {
                            unreadable = !*got_any && started.elapsed() < Duration::from_secs(10);
                            break;
                        }
                    }
                }
            }
            Feed::Synthetic {
                next_at,
                state,
                offset,
                ..
            } => {
                let now = Instant::now();
                let mut n = 0;
                while *next_at <= now && n < max {
                    *state ^= *state << 13;
                    *state ^= *state >> 7;
                    *state ^= *state << 17;
                    buf.push(synthetic_line(*state, *offset));
                    *next_at += Duration::from_millis(120 + *state % 700);
                    n += 1;
                }
            }
        }
        if unreadable {
            // Dropping the journal feed reaps the child.
            *self = Feed::synthetic("the journal was not readable");
        }
    }
}

impl Drop for Feed {
    fn drop(&mut self) {
        if let Feed::Journal { child, .. } = self {
            // Our own child, stopped through its handle.
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

const SYNTHETIC: &[(Severity, &str, &str)] = &[
    (
        Severity::Info,
        "restic[2231]",
        "snapshot 4f1c9a02 saved, 1,284 files changed, 212 MiB added",
    ),
    (
        Severity::Info,
        "nginx[918]",
        "10.10.10.6 GET /api/health 200 3 ms",
    ),
    (
        Severity::Debug,
        "dockerd[1102]",
        "container web-2 health check passed in 41 ms",
    ),
    (
        Severity::Notice,
        "systemd[1]",
        "Started backup.service - nightly off-site backup.",
    ),
    (
        Severity::Warning,
        "smartd[744]",
        "Device /dev/nvme0n1, temperature 71 C reached the warning limit",
    ),
    (
        Severity::Info,
        "sshd[40117]",
        "Accepted publickey for kenny from 10.10.10.3 port 51522",
    ),
    (
        Severity::Error,
        "caddy[1310]",
        "certificate renewal for media.home failed: DNS challenge timed out",
    ),
    (
        Severity::Debug,
        "kernel",
        "eno1: link speed 1000 Mbps, full duplex",
    ),
    (
        Severity::Notice,
        "systemd[1]",
        "Finished logrotate.service - rotated 14 log files.",
    ),
    (
        Severity::Critical,
        "kernel",
        "EXT4-fs error (device sdb1): inode 1318 has a bad checksum",
    ),
    (
        Severity::Info,
        "jellyfin[2890]",
        "Playback started: episode 4 on the living-room client",
    ),
    (
        Severity::Warning,
        "dockerd[1102]",
        "container worker-1 restarted 3 times in 10 minutes",
    ),
];

fn synthetic_line(r: u64, offset: i64) -> LogLine {
    // Mostly the quiet levels, now and then a loud one.
    let pick = match r % 20 {
        0 => 9,
        1 | 2 => 6,
        3..=5 => 4,
        6 => 11,
        _ => [0, 1, 2, 3, 5, 7, 8, 10][(r >> 8) as usize % 8],
    };
    let (severity, unit, message) = SYNTHETIC[pick];
    let micros = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_micros() as u64)
        .unwrap_or(0);
    LogLine::new(&clock(micros, offset), "synthetic", unit, severity, message)
}
