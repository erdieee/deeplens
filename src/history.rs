use crate::settings::AppSettings;
use crate::types::SearchResultKind;

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct SearchHistory {
    entries: Vec<HistoryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HistoryEntry {
    path: String,
    kind: String,
    open_count: u64,
    last_opened_unix: u64,
}

impl SearchHistory {
    pub fn load() -> Self {
        let Some(path) = history_path() else {
            return Self::default();
        };

        fs::read_to_string(path)
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default()
    }

    pub fn record_open(
        &mut self,
        path: &Path,
        kind: SearchResultKind,
        settings: &AppSettings,
    ) -> Result<(), String> {
        if !settings.history_enabled {
            return Ok(());
        }

        let path = path.to_string_lossy().to_string();
        let now = unix_now();

        if let Some(entry) = self.entries.iter_mut().find(|entry| entry.path == path) {
            entry.kind = history_kind(kind).to_owned();
            entry.open_count = entry.open_count.saturating_add(1);
            entry.last_opened_unix = now;
        } else {
            self.entries.push(HistoryEntry {
                path,
                kind: history_kind(kind).to_owned(),
                open_count: 1,
                last_opened_unix: now,
            });
        }

        self.trim(settings.max_history_items);
        self.save().map(|_| ())
    }

    pub fn score_for(&self, path: &Path, settings: &AppSettings) -> i64 {
        if !settings.history_enabled {
            return 0;
        }

        let path = path.to_string_lossy();
        let Some(entry) = self.entries.iter().find(|entry| entry.path == path) else {
            return 0;
        };

        let frequency_score = entry
            .open_count
            .saturating_mul(settings.history_frequency_boost as u64)
            .min(i64::MAX as u64) as i64;

        let age_days = unix_now()
            .saturating_sub(entry.last_opened_unix)
            .saturating_div(86_400);
        let recency_score = if settings.history_recency_days == 0 {
            settings.history_recency_boost as i64
        } else if age_days >= settings.history_recency_days {
            0
        } else {
            let remaining_days = settings.history_recency_days - age_days;
            ((settings.history_recency_boost as u64).saturating_mul(remaining_days)
                / settings.history_recency_days) as i64
        };

        frequency_score.saturating_add(recency_score)
    }

    pub fn trim(&mut self, max_items: usize) {
        self.entries
            .sort_by(|left, right| right.last_opened_unix.cmp(&left.last_opened_unix));
        self.entries.truncate(max_items);
    }

    pub fn save(&self) -> Result<PathBuf, String> {
        let path =
            history_path().ok_or_else(|| String::from("Could not locate history folder."))?;
        let parent = path
            .parent()
            .ok_or_else(|| String::from("Could not locate history folder."))?;

        fs::create_dir_all(parent).map_err(|error| error.to_string())?;

        let text = serde_json::to_string_pretty(self).map_err(|error| error.to_string())?;
        fs::write(&path, text).map_err(|error| error.to_string())?;

        Ok(path)
    }
}

pub fn history_path() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(".deeplens").join("history.json"))
}

fn history_kind(kind: SearchResultKind) -> &'static str {
    match kind {
        SearchResultKind::FileContent => "file",
        SearchResultKind::FolderName => "folder",
        SearchResultKind::Application => "app",
        SearchResultKind::Calculator => "calculator",
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
    use super::{HistoryEntry, SearchHistory};
    use crate::settings::AppSettings;
    use std::path::Path;

    #[test]
    fn scores_recent_and_frequent_entries() {
        let settings = AppSettings {
            history_frequency_boost: 25,
            history_recency_boost: 100,
            history_recency_days: 30,
            ..AppSettings::default()
        };
        let history = SearchHistory {
            entries: vec![HistoryEntry {
                path: String::from("/tmp/example.txt"),
                kind: String::from("file"),
                open_count: 2,
                last_opened_unix: super::unix_now(),
            }],
        };

        assert!(history.score_for(Path::new("/tmp/example.txt"), &settings) >= 150);
        assert_eq!(history.score_for(Path::new("/tmp/other.txt"), &settings), 0);
    }

    #[test]
    fn disabled_history_does_not_score() {
        let settings = AppSettings {
            history_enabled: false,
            ..AppSettings::default()
        };
        let history = SearchHistory {
            entries: vec![HistoryEntry {
                path: String::from("/tmp/example.txt"),
                kind: String::from("file"),
                open_count: 10,
                last_opened_unix: super::unix_now(),
            }],
        };

        assert_eq!(
            history.score_for(Path::new("/tmp/example.txt"), &settings),
            0
        );
    }
}
