use crate::settings::AppSettings;

use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ClipboardHistory {
    entries: Vec<ClipboardEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardEntry {
    pub id: String,
    pub text: String,
    pub copied_unix: u64,
}

impl ClipboardHistory {
    pub fn load() -> Self {
        let Some(path) = clipboard_path() else {
            return Self::default();
        };

        fs::read_to_string(path)
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default()
    }

    pub fn record_text(&mut self, text: &str, settings: &AppSettings) -> Result<bool, String> {
        if !settings.clipboard_history_enabled
            || text.trim().is_empty()
            || text.len() > settings.max_clipboard_text_bytes
        {
            return Ok(false);
        }

        if self.entries.first().is_some_and(|entry| entry.text == text) {
            return Ok(false);
        }

        self.record_text_in_memory(text, settings);
        self.save().map(|_| true)
    }

    pub fn search(&self, query: &str, settings: &AppSettings) -> Vec<ClipboardEntry> {
        if !settings.clipboard_history_enabled {
            return Vec::new();
        }

        let query = query.trim().to_ascii_lowercase();
        self.entries
            .iter()
            .filter(|entry| {
                query.is_empty() || entry.text.to_ascii_lowercase().contains(query.as_str())
            })
            .take(settings.max_displayed_results)
            .cloned()
            .collect()
    }

    pub fn delete(&mut self, id: &str) -> Result<bool, String> {
        let before = self.entries.len();
        self.entries.retain(|entry| entry.id != id);

        if self.entries.len() == before {
            return Ok(false);
        }

        self.save().map(|_| true)
    }

    pub fn trim(&mut self, max_items: usize) {
        self.entries
            .sort_by(|left, right| right.copied_unix.cmp(&left.copied_unix));
        self.entries.truncate(max_items);
    }

    fn record_text_in_memory(&mut self, text: &str, settings: &AppSettings) {
        let id = clipboard_id(text);
        self.entries
            .retain(|entry| entry.id != id && entry.text != text);
        self.entries.insert(
            0,
            ClipboardEntry {
                id,
                text: text.to_owned(),
                copied_unix: unix_now(),
            },
        );
        self.trim(settings.max_clipboard_items);
    }

    pub fn save(&self) -> Result<PathBuf, String> {
        let path = clipboard_path()
            .ok_or_else(|| String::from("Could not locate clipboard history folder."))?;
        let parent = path
            .parent()
            .ok_or_else(|| String::from("Could not locate clipboard history folder."))?;

        fs::create_dir_all(parent).map_err(|error| error.to_string())?;

        let text = serde_json::to_string_pretty(self).map_err(|error| error.to_string())?;
        fs::write(&path, text).map_err(|error| error.to_string())?;

        Ok(path)
    }
}

pub fn parse_clipboard_query(input: &str) -> Option<String> {
    let trimmed = input.trim();
    let mut parts = trimmed.splitn(2, char::is_whitespace);
    let command = parts.next()?.to_ascii_lowercase();

    if command != "clip" && command != "clipboard" {
        return None;
    }

    Some(parts.next().map(str::trim).unwrap_or_default().to_owned())
}

pub fn read_system_clipboard() -> Result<Option<String>, String> {
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("/usr/bin/pbpaste")
            .output()
            .map_err(|error| error.to_string())?;

        if !output.status.success() {
            return Ok(None);
        }

        return Ok(Some(String::from_utf8_lossy(&output.stdout).into_owned()));
    }

    #[cfg(not(target_os = "macos"))]
    {
        Ok(None)
    }
}

pub fn clipboard_path() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(".deeplens").join("clipboard.json"))
}

fn clipboard_id(text: &str) -> String {
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::{parse_clipboard_query, ClipboardHistory};
    use crate::settings::AppSettings;

    #[test]
    fn parses_clipboard_queries() {
        assert_eq!(
            parse_clipboard_query("clip token"),
            Some(String::from("token"))
        );
        assert_eq!(
            parse_clipboard_query("clipboard token"),
            Some(String::from("token"))
        );
        assert_eq!(parse_clipboard_query("clip"), Some(String::new()));
        assert_eq!(parse_clipboard_query("clips token"), None);
    }

    #[test]
    fn records_deduplicated_recent_text() {
        let settings = AppSettings {
            max_clipboard_items: 2,
            ..AppSettings::default()
        };
        let mut history = ClipboardHistory::default();

        history.record_text_in_memory("first", &settings);
        history.record_text_in_memory("second", &settings);
        history.record_text_in_memory("first", &settings);

        let entries = history.search("", &settings);
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].text, "first");
        assert_eq!(entries[1].text, "second");
    }

    #[test]
    fn ignores_empty_disabled_and_huge_text() {
        let mut history = ClipboardHistory::default();
        assert!(!history
            .record_text(
                "value",
                &AppSettings {
                    clipboard_history_enabled: false,
                    ..AppSettings::default()
                },
            )
            .unwrap());
        assert!(!history.record_text("   ", &AppSettings::default()).unwrap());
        assert!(!history
            .record_text(
                "large",
                &AppSettings {
                    max_clipboard_text_bytes: 2,
                    ..AppSettings::default()
                },
            )
            .unwrap());
    }
}
