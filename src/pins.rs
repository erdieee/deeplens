use crate::settings::AppSettings;
use crate::types::SearchResultKind;

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Pins {
    entries: Vec<PinEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PinEntry {
    path: String,
    kind: String,
    pinned_unix: u64,
}

impl Pins {
    pub fn load() -> Self {
        let Some(path) = pins_path() else {
            return Self::default();
        };

        fs::read_to_string(path)
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default()
    }

    pub fn is_pinned(&self, path: &Path) -> bool {
        let path = path.to_string_lossy();
        self.entries.iter().any(|entry| entry.path == path)
    }

    pub fn pin(
        &mut self,
        path: &Path,
        kind: SearchResultKind,
        settings: &AppSettings,
    ) -> Result<(), String> {
        if !settings.pins_enabled {
            return Ok(());
        }

        let path = path.to_string_lossy().to_string();

        if let Some(entry) = self.entries.iter_mut().find(|entry| entry.path == path) {
            entry.kind = pin_kind(kind).to_owned();
            entry.pinned_unix = unix_now();
        } else {
            self.entries.push(PinEntry {
                path,
                kind: pin_kind(kind).to_owned(),
                pinned_unix: unix_now(),
            });
        }

        self.trim(settings.max_pinned_items);
        self.save().map(|_| ())
    }

    pub fn unpin(&mut self, path: &Path) -> Result<(), String> {
        let path = path.to_string_lossy();
        self.entries.retain(|entry| entry.path != path);
        self.save().map(|_| ())
    }

    pub fn score_for(&self, path: &Path, settings: &AppSettings) -> i64 {
        if !settings.pins_enabled || !self.is_pinned(path) {
            return 0;
        }

        settings.pin_rank_boost as i64
    }

    pub fn trim(&mut self, max_items: usize) {
        self.entries
            .sort_by(|left, right| right.pinned_unix.cmp(&left.pinned_unix));
        self.entries.truncate(max_items);
    }

    pub fn save(&self) -> Result<PathBuf, String> {
        let path = pins_path().ok_or_else(|| String::from("Could not locate pins folder."))?;
        let parent = path
            .parent()
            .ok_or_else(|| String::from("Could not locate pins folder."))?;

        fs::create_dir_all(parent).map_err(|error| error.to_string())?;

        let text = serde_json::to_string_pretty(self).map_err(|error| error.to_string())?;
        fs::write(&path, text).map_err(|error| error.to_string())?;

        Ok(path)
    }
}

pub fn pins_path() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(".deeplens").join("pins.json"))
}

fn pin_kind(kind: SearchResultKind) -> &'static str {
    match kind {
        SearchResultKind::FileContent => "file",
        SearchResultKind::FolderName => "folder",
        SearchResultKind::Application => "app",
        SearchResultKind::Calculator => "calculator",
        SearchResultKind::Clipboard => "clipboard",
    }
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::{PinEntry, Pins};
    use crate::settings::AppSettings;
    use std::path::Path;

    #[test]
    fn scores_pinned_paths() {
        let settings = AppSettings {
            pin_rank_boost: 750,
            ..AppSettings::default()
        };
        let pins = Pins {
            entries: vec![PinEntry {
                path: String::from("/tmp/example.txt"),
                kind: String::from("file"),
                pinned_unix: super::unix_now(),
            }],
        };

        assert_eq!(
            pins.score_for(Path::new("/tmp/example.txt"), &settings),
            750
        );
        assert_eq!(pins.score_for(Path::new("/tmp/other.txt"), &settings), 0);
    }

    #[test]
    fn disabled_pins_do_not_score() {
        let settings = AppSettings {
            pins_enabled: false,
            ..AppSettings::default()
        };
        let pins = Pins {
            entries: vec![PinEntry {
                path: String::from("/tmp/example.txt"),
                kind: String::from("file"),
                pinned_unix: super::unix_now(),
            }],
        };

        assert_eq!(pins.score_for(Path::new("/tmp/example.txt"), &settings), 0);
    }
}
