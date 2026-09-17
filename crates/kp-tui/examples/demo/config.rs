//! The persisted choice: two `key = value` lines. A real crate would use
//! `toml` + `dirs`; the demo avoids both to keep the dependency count at
//! what the research is about.

use std::{fs, io, path::PathBuf};

use kp_tui::ThemeId;
use kp_tui::fx::Motion;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Config {
    pub theme: ThemeId,
    pub motion: Motion,
}

impl Default for Config {
    fn default() -> Self {
        // formal is the package's default theme (anatomy.md).
        Config {
            theme: ThemeId::Formal,
            motion: Motion::Full,
        }
    }
}

/// `KP_TUI_CONFIG`, else `$XDG_CONFIG_HOME/kp-tui-demo/config`, else
/// `$HOME/.config/kp-tui-demo/config`.
pub fn default_path(env: impl Fn(&str) -> Option<String>) -> Option<PathBuf> {
    if let Some(p) = env("KP_TUI_CONFIG") {
        return Some(PathBuf::from(p));
    }
    let base = env("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| env("HOME").map(|h| PathBuf::from(h).join(".config")))?;
    Some(base.join("kp-tui-demo").join("config"))
}

impl Config {
    pub fn parse(text: &str) -> Self {
        let mut c = Config::default();
        for line in text.lines() {
            let Some((k, v)) = line.split_once('=') else {
                continue;
            };
            match (k.trim(), v.trim()) {
                ("theme", v) => c.theme = ThemeId::from_name(v).unwrap_or(c.theme),
                ("reduced_motion", "true") => c.motion = Motion::Reduced,
                ("reduced_motion", "false") => c.motion = Motion::Full,
                _ => {}
            }
        }
        c
    }

    pub fn render(&self) -> String {
        format!(
            "theme = {}\nreduced_motion = {}\n",
            self.theme.name(),
            self.motion == Motion::Reduced
        )
    }

    pub fn load(path: &std::path::Path) -> Self {
        fs::read_to_string(path)
            .map(|t| Self::parse(&t))
            .unwrap_or_default()
    }

    pub fn save(&self, path: &std::path::Path) -> io::Result<()> {
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        fs::write(path, self.render())
    }
}
