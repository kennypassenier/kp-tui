//! Machine stats read straight from `/proc`, no crate.
//!
//! Why not `sysinfo`: the dashboard needs six files (`stat`, `meminfo`,
//! `loadavg`, `net/dev`, `diskstats`, `<pid>/stat`), each a few lines of
//! whitespace splitting, and on Linux `sysinfo` reads the same files. Its
//! disk I/O is per mounted filesystem, so partitions and bind mounts would
//! need de-duplicating, and a probe with the same readings (sysinfo 0.39.6)
//! was 167,840 bytes larger than an empty binary. The cost of this choice
//! is that the dashboard is Linux-only; `sysinfo` earns its place the day
//! a consumer runs on macOS or Windows.
//!
//! The parsers are pure functions over the file text, so tests feed them
//! fixtures; [`Sampler`] is the only part that touches the filesystem.

use std::{
    fs, io,
    path::{Path, PathBuf},
    time::Instant,
};

/// Cumulative jiffies of one CPU line in `/proc/stat`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CpuTimes {
    pub total: u64,
    /// idle + iowait
    pub idle: u64,
}

/// Cumulative counters at one instant. Rates come from two of these.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Counters {
    /// Index 0 is the `cpu` total line, then `cpu0`, `cpu1`, ...
    pub cpu: Vec<CpuTimes>,
    pub mem_total_kib: u64,
    pub mem_available_kib: u64,
    pub load: [f64; 3],
    pub net_rx_bytes: u64,
    pub net_tx_bytes: u64,
    pub disk_read_bytes: u64,
    pub disk_write_bytes: u64,
    /// utime + stime of a process, in clock ticks.
    pub self_ticks: u64,
    pub child_ticks: u64,
}

/// What the dashboard draws: rates and percentages over one interval.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Sample {
    pub cpu_total: f64,
    pub cores: Vec<f64>,
    pub mem_used_pct: f64,
    pub mem_used_bytes: u64,
    pub mem_total_bytes: u64,
    pub load: [f64; 3],
    pub rx_bps: f64,
    pub tx_bps: f64,
    pub disk_read_bps: f64,
    pub disk_write_bps: f64,
    /// The demo's own CPU, in percent of one core.
    pub self_cpu_pct: f64,
    /// The spawned `journalctl`, in percent of one core.
    pub child_cpu_pct: f64,
}

pub fn parse_stat(text: &str) -> Vec<CpuTimes> {
    text.lines()
        .filter(|l| l.starts_with("cpu"))
        .map(|l| {
            let n: Vec<u64> = l
                .split_whitespace()
                .skip(1)
                .filter_map(|v| v.parse().ok())
                .collect();
            // user nice system idle iowait irq softirq steal (guest is already in user)
            let total = n.iter().take(8).sum();
            let idle = n.get(3).copied().unwrap_or(0) + n.get(4).copied().unwrap_or(0);
            CpuTimes { total, idle }
        })
        .collect()
}

/// `(MemTotal, MemAvailable)` in KiB.
pub fn parse_meminfo(text: &str) -> (u64, u64) {
    let field = |name: &str| {
        text.lines()
            .find(|l| l.starts_with(name))
            .and_then(|l| l.split_whitespace().nth(1))
            .and_then(|v| v.parse().ok())
            .unwrap_or(0)
    };
    (field("MemTotal:"), field("MemAvailable:"))
}

pub fn parse_loadavg(text: &str) -> [f64; 3] {
    let mut it = text.split_whitespace().map(|v| v.parse().unwrap_or(0.0));
    [
        it.next().unwrap_or(0.0),
        it.next().unwrap_or(0.0),
        it.next().unwrap_or(0.0),
    ]
}

/// Received and sent bytes over every interface except loopback.
pub fn parse_net_dev(text: &str) -> (u64, u64) {
    text.lines()
        .filter_map(|l| l.split_once(':'))
        .filter(|(name, _)| name.trim() != "lo")
        .map(|(_, rest)| {
            let n: Vec<u64> = rest
                .split_whitespace()
                .filter_map(|v| v.parse().ok())
                .collect();
            (
                n.first().copied().unwrap_or(0),
                n.get(8).copied().unwrap_or(0),
            )
        })
        .fold((0, 0), |(r, t), (a, b)| (r + a, t + b))
}

/// Read and written bytes over the devices `is_disk` accepts. Partitions
/// and virtual devices are left out so nothing is counted twice.
pub fn parse_diskstats(text: &str, is_disk: impl Fn(&str) -> bool) -> (u64, u64) {
    text.lines()
        .map(|l| l.split_whitespace().collect::<Vec<_>>())
        .filter(|f| f.len() > 9 && is_disk(f[2]))
        .map(|f| {
            // fields 6 and 10 are sectors read and written; a sector is 512 bytes here
            let s = |i: usize| f[i].parse::<u64>().unwrap_or(0) * 512;
            (s(5), s(9))
        })
        .fold((0, 0), |(r, w), (a, b)| (r + a, w + b))
}

/// utime + stime from `/proc/<pid>/stat`. The command name may hold spaces
/// and parentheses, so fields are counted after the last `)`.
pub fn parse_proc_ticks(text: &str) -> u64 {
    let Some((_, rest)) = text.rsplit_once(')') else {
        return 0;
    };
    let f: Vec<&str> = rest.split_whitespace().collect();
    // after ')': state(0) ppid(1) ... utime(11) stime(12)
    let n = |i: usize| f.get(i).and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);
    n(11) + n(12)
}

/// Clock ticks per second. `USER_HZ` is 100 on every Linux architecture
/// ratatui targets; `getconf CLK_TCK` printed 100 on the measuring machine.
pub const CLK_TCK: f64 = 100.0;

pub fn rates(prev: &Counters, cur: &Counters, secs: f64) -> Sample {
    let secs = secs.max(1e-3);
    let busy = |a: CpuTimes, b: CpuTimes| {
        let dt = b.total.saturating_sub(a.total);
        if dt == 0 {
            0.0
        } else {
            100.0 * (dt - b.idle.saturating_sub(a.idle).min(dt)) as f64 / dt as f64
        }
    };
    let pcts: Vec<f64> = prev
        .cpu
        .iter()
        .zip(&cur.cpu)
        .map(|(a, b)| busy(*a, *b))
        .collect();
    let per = |a: u64, b: u64| b.saturating_sub(a) as f64 / secs;
    let used = cur.mem_total_kib.saturating_sub(cur.mem_available_kib);
    Sample {
        cpu_total: pcts.first().copied().unwrap_or(0.0),
        cores: pcts.iter().skip(1).copied().collect(),
        mem_used_pct: if cur.mem_total_kib == 0 {
            0.0
        } else {
            100.0 * used as f64 / cur.mem_total_kib as f64
        },
        mem_used_bytes: used * 1024,
        mem_total_bytes: cur.mem_total_kib * 1024,
        load: cur.load,
        rx_bps: per(prev.net_rx_bytes, cur.net_rx_bytes),
        tx_bps: per(prev.net_tx_bytes, cur.net_tx_bytes),
        disk_read_bps: per(prev.disk_read_bytes, cur.disk_read_bytes),
        disk_write_bps: per(prev.disk_write_bytes, cur.disk_write_bytes),
        self_cpu_pct: 100.0 * per(prev.self_ticks, cur.self_ticks) / CLK_TCK,
        child_cpu_pct: 100.0 * per(prev.child_ticks, cur.child_ticks) / CLK_TCK,
    }
}

/// Reads the counters from a `/proc` root and turns two readings into a
/// [`Sample`].
pub struct Sampler {
    proc: PathBuf,
    child: Option<u32>,
    last: Option<(Instant, Counters)>,
}

impl Sampler {
    pub fn new(child: Option<u32>) -> Self {
        Sampler {
            proc: PathBuf::from("/proc"),
            child,
            last: None,
        }
    }

    fn read(&self) -> io::Result<Counters> {
        let get = |p: &str| fs::read_to_string(self.proc.join(p));
        let (mem_total_kib, mem_available_kib) = parse_meminfo(&get("meminfo")?);
        let (net_rx_bytes, net_tx_bytes) = get("net/dev")
            .map(|t| parse_net_dev(&t))
            .unwrap_or_default();
        // A physical block device has a `device` link; zram, loop and dm do not.
        let (disk_read_bytes, disk_write_bytes) = get("diskstats")
            .map(|t| {
                parse_diskstats(&t, |name| {
                    Path::new("/sys/block").join(name).join("device").exists()
                })
            })
            .unwrap_or_default();
        let ticks = |pid: &str| {
            get(&format!("{pid}/stat"))
                .map(|t| parse_proc_ticks(&t))
                .unwrap_or(0)
        };
        Ok(Counters {
            cpu: parse_stat(&get("stat")?),
            mem_total_kib,
            mem_available_kib,
            load: get("loadavg")
                .map(|t| parse_loadavg(&t))
                .unwrap_or_default(),
            net_rx_bytes,
            net_tx_bytes,
            disk_read_bytes,
            disk_write_bytes,
            self_ticks: ticks("self"),
            child_ticks: self.child.map(|p| ticks(&p.to_string())).unwrap_or(0),
        })
    }

    /// `None` on the first call (a rate needs two readings) or when `/proc`
    /// is unreadable.
    pub fn sample(&mut self) -> Option<Sample> {
        let now = Instant::now();
        let cur = self.read().ok()?;
        let out = self
            .last
            .as_ref()
            .map(|(at, prev)| rates(prev, &cur, now.duration_since(*at).as_secs_f64()));
        self.last = Some((now, cur));
        out
    }
}
