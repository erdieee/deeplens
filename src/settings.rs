use crate::shortcuts::{validate_global_shortcut, validate_shortcut, validate_shortcut_list};
use crate::web_search::validate_web_shortcuts;

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub max_displayed_results: usize,
    pub max_search_events_per_tick: usize,
    pub search_event_limit_multiplier: usize,
    pub history_enabled: bool,
    pub max_history_items: usize,
    pub history_frequency_boost: usize,
    pub history_recency_boost: usize,
    pub history_recency_days: u64,
    pub calculator_enabled: bool,
    pub calculator_requires_prefix: bool,
    pub calculator_command: String,
    pub preview_enabled: bool,
    pub search_debounce_ms: u64,
    pub double_click_ms: u64,
    pub min_query_chars: usize,
    pub window_width: f32,
    pub idle_height: f32,
    pub compact_height: f32,
    pub expanded_height: f32,
    pub result_row_height: f32,
    pub terminal_app: String,
    pub excluded_folders: String,
    pub web_search_shortcuts: String,
    pub global_shortcut: String,
    pub submit_shortcut: String,
    pub previous_result_shortcut: String,
    pub next_result_shortcut: String,
    pub accept_completion_shortcuts: String,
    pub cancel_shortcut: String,
    pub settings_shortcut: String,
    pub reveal_shortcut: String,
    pub copy_path_shortcut: String,
    pub terminal_shortcut: String,
    pub preview_shortcut: String,
    pub surface_color: String,
    pub prompt_color: String,
    pub text_color: String,
    pub muted_color: String,
    pub faint_color: String,
    pub border_color: String,
    pub selected_color: String,
    pub chip_color: String,
    pub accent_color: String,
    pub highlight_color: String,
    pub error_color: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            max_displayed_results: 300,
            max_search_events_per_tick: 160,
            search_event_limit_multiplier: 20,
            history_enabled: true,
            max_history_items: 500,
            history_frequency_boost: 35,
            history_recency_boost: 300,
            history_recency_days: 30,
            calculator_enabled: true,
            calculator_requires_prefix: false,
            calculator_command: String::from("numbat"),
            preview_enabled: true,
            search_debounce_ms: 650,
            double_click_ms: 420,
            min_query_chars: 3,
            window_width: 720.0,
            idle_height: 172.0,
            compact_height: 420.0,
            expanded_height: 560.0,
            result_row_height: 82.0,
            terminal_app: String::from("Terminal"),
            excluded_folders: String::from(
                "Library, Library/Caches, Library/Developer, .Trash, node_modules, target, .git",
            ),
            web_search_shortcuts: String::from(
                "g=https://www.google.com/search?q={query}, google=https://www.google.com/search?q={query}, gh=https://github.com/search?q={query}, github=https://github.com/search?q={query}, yt=https://www.youtube.com/results?search_query={query}, youtube=https://www.youtube.com/results?search_query={query}, docs=https://www.google.com/search?q={query}+documentation",
            ),
            global_shortcut: String::from("cmd+shift+space"),
            submit_shortcut: String::from("enter"),
            previous_result_shortcut: String::from("up"),
            next_result_shortcut: String::from("down"),
            accept_completion_shortcuts: String::from("tab,right"),
            cancel_shortcut: String::from("esc"),
            settings_shortcut: String::from("cmd+,"),
            reveal_shortcut: String::from("cmd+enter"),
            copy_path_shortcut: String::from("cmd+c"),
            terminal_shortcut: String::from("cmd+t"),
            preview_shortcut: String::from("space"),
            surface_color: String::from("#111214"),
            prompt_color: String::from("#121315"),
            text_color: String::from("#E0E0DB"),
            muted_color: String::from("#9E9E99"),
            faint_color: String::from("#5C5E63"),
            border_color: String::from("#42454D"),
            selected_color: String::from("#25272E"),
            chip_color: String::from("#242528"),
            accent_color: String::from("#758FBD"),
            highlight_color: String::from("#B3C7F2"),
            error_color: String::from("#F57161"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SettingsForm {
    pub max_displayed_results: String,
    pub max_search_events_per_tick: String,
    pub search_event_limit_multiplier: String,
    pub history_enabled: String,
    pub max_history_items: String,
    pub history_frequency_boost: String,
    pub history_recency_boost: String,
    pub history_recency_days: String,
    pub calculator_enabled: String,
    pub calculator_requires_prefix: String,
    pub calculator_command: String,
    pub preview_enabled: String,
    pub search_debounce_ms: String,
    pub double_click_ms: String,
    pub min_query_chars: String,
    pub window_width: String,
    pub idle_height: String,
    pub compact_height: String,
    pub expanded_height: String,
    pub result_row_height: String,
    pub terminal_app: String,
    pub excluded_folders: String,
    pub web_search_shortcuts: String,
    pub global_shortcut: String,
    pub submit_shortcut: String,
    pub previous_result_shortcut: String,
    pub next_result_shortcut: String,
    pub accept_completion_shortcuts: String,
    pub cancel_shortcut: String,
    pub settings_shortcut: String,
    pub reveal_shortcut: String,
    pub copy_path_shortcut: String,
    pub terminal_shortcut: String,
    pub preview_shortcut: String,
    pub surface_color: String,
    pub prompt_color: String,
    pub text_color: String,
    pub muted_color: String,
    pub faint_color: String,
    pub border_color: String,
    pub selected_color: String,
    pub chip_color: String,
    pub accent_color: String,
    pub highlight_color: String,
    pub error_color: String,
}

impl From<&AppSettings> for SettingsForm {
    fn from(settings: &AppSettings) -> Self {
        Self {
            max_displayed_results: settings.max_displayed_results.to_string(),
            max_search_events_per_tick: settings.max_search_events_per_tick.to_string(),
            search_event_limit_multiplier: settings.search_event_limit_multiplier.to_string(),
            history_enabled: settings.history_enabled.to_string(),
            max_history_items: settings.max_history_items.to_string(),
            history_frequency_boost: settings.history_frequency_boost.to_string(),
            history_recency_boost: settings.history_recency_boost.to_string(),
            history_recency_days: settings.history_recency_days.to_string(),
            calculator_enabled: settings.calculator_enabled.to_string(),
            calculator_requires_prefix: settings.calculator_requires_prefix.to_string(),
            calculator_command: settings.calculator_command.clone(),
            preview_enabled: settings.preview_enabled.to_string(),
            search_debounce_ms: settings.search_debounce_ms.to_string(),
            double_click_ms: settings.double_click_ms.to_string(),
            min_query_chars: settings.min_query_chars.to_string(),
            window_width: format_float(settings.window_width),
            idle_height: format_float(settings.idle_height),
            compact_height: format_float(settings.compact_height),
            expanded_height: format_float(settings.expanded_height),
            result_row_height: format_float(settings.result_row_height),
            terminal_app: settings.terminal_app.clone(),
            excluded_folders: settings.excluded_folders.clone(),
            web_search_shortcuts: settings.web_search_shortcuts.clone(),
            global_shortcut: settings.global_shortcut.clone(),
            submit_shortcut: settings.submit_shortcut.clone(),
            previous_result_shortcut: settings.previous_result_shortcut.clone(),
            next_result_shortcut: settings.next_result_shortcut.clone(),
            accept_completion_shortcuts: settings.accept_completion_shortcuts.clone(),
            cancel_shortcut: settings.cancel_shortcut.clone(),
            settings_shortcut: settings.settings_shortcut.clone(),
            reveal_shortcut: settings.reveal_shortcut.clone(),
            copy_path_shortcut: settings.copy_path_shortcut.clone(),
            terminal_shortcut: settings.terminal_shortcut.clone(),
            preview_shortcut: settings.preview_shortcut.clone(),
            surface_color: settings.surface_color.clone(),
            prompt_color: settings.prompt_color.clone(),
            text_color: settings.text_color.clone(),
            muted_color: settings.muted_color.clone(),
            faint_color: settings.faint_color.clone(),
            border_color: settings.border_color.clone(),
            selected_color: settings.selected_color.clone(),
            chip_color: settings.chip_color.clone(),
            accent_color: settings.accent_color.clone(),
            highlight_color: settings.highlight_color.clone(),
            error_color: settings.error_color.clone(),
        }
    }
}

impl SettingsForm {
    pub fn parse(&self) -> Result<AppSettings, String> {
        let settings = AppSettings {
            max_displayed_results: parse_usize(&self.max_displayed_results, "Max results")?,
            max_search_events_per_tick: parse_usize(
                &self.max_search_events_per_tick,
                "Max search events per tick",
            )?,
            search_event_limit_multiplier: parse_usize(
                &self.search_event_limit_multiplier,
                "Search event limit multiplier",
            )?,
            history_enabled: parse_bool(&self.history_enabled, "History enabled")?,
            max_history_items: parse_usize(&self.max_history_items, "Max history items")?,
            history_frequency_boost: parse_usize(
                &self.history_frequency_boost,
                "History frequency boost",
            )?,
            history_recency_boost: parse_usize(
                &self.history_recency_boost,
                "History recency boost",
            )?,
            history_recency_days: parse_u64(&self.history_recency_days, "History recency days")?,
            calculator_enabled: parse_bool(&self.calculator_enabled, "Calculator enabled")?,
            calculator_requires_prefix: parse_bool(
                &self.calculator_requires_prefix,
                "Calculator requires prefix",
            )?,
            calculator_command: parse_non_empty_text(
                &self.calculator_command,
                "Calculator command",
            )?,
            preview_enabled: parse_bool(&self.preview_enabled, "Preview enabled")?,
            search_debounce_ms: parse_u64(&self.search_debounce_ms, "Debounce")?,
            double_click_ms: parse_u64(&self.double_click_ms, "Double-click window")?,
            min_query_chars: parse_usize(&self.min_query_chars, "Minimum query characters")?,
            window_width: parse_f32(&self.window_width, "Window width")?,
            idle_height: parse_f32(&self.idle_height, "Idle height")?,
            compact_height: parse_f32(&self.compact_height, "Compact height")?,
            expanded_height: parse_f32(&self.expanded_height, "Expanded height")?,
            result_row_height: parse_f32(&self.result_row_height, "Result row height")?,
            terminal_app: parse_non_empty_text(&self.terminal_app, "Terminal app")?,
            excluded_folders: parse_comma_list(&self.excluded_folders, "Excluded folders")?,
            web_search_shortcuts: validate_web_shortcuts(&self.web_search_shortcuts)?,
            global_shortcut: validate_global_shortcut(&self.global_shortcut, "Global shortcut")?,
            submit_shortcut: validate_shortcut(&self.submit_shortcut, "Submit shortcut")?,
            previous_result_shortcut: validate_shortcut(
                &self.previous_result_shortcut,
                "Previous result shortcut",
            )?,
            next_result_shortcut: validate_shortcut(
                &self.next_result_shortcut,
                "Next result shortcut",
            )?,
            accept_completion_shortcuts: validate_shortcut_list(
                &self.accept_completion_shortcuts,
                "Accept completion shortcuts",
            )?,
            cancel_shortcut: validate_shortcut(&self.cancel_shortcut, "Cancel shortcut")?,
            settings_shortcut: validate_shortcut(&self.settings_shortcut, "Settings shortcut")?,
            reveal_shortcut: validate_shortcut(&self.reveal_shortcut, "Reveal shortcut")?,
            copy_path_shortcut: validate_shortcut(&self.copy_path_shortcut, "Copy path shortcut")?,
            terminal_shortcut: validate_shortcut(&self.terminal_shortcut, "Terminal shortcut")?,
            preview_shortcut: validate_shortcut(&self.preview_shortcut, "Preview shortcut")?,
            surface_color: parse_hex_color(&self.surface_color, "Surface color")?,
            prompt_color: parse_hex_color(&self.prompt_color, "Prompt color")?,
            text_color: parse_hex_color(&self.text_color, "Text color")?,
            muted_color: parse_hex_color(&self.muted_color, "Muted color")?,
            faint_color: parse_hex_color(&self.faint_color, "Faint color")?,
            border_color: parse_hex_color(&self.border_color, "Border color")?,
            selected_color: parse_hex_color(&self.selected_color, "Selected color")?,
            chip_color: parse_hex_color(&self.chip_color, "Chip color")?,
            accent_color: parse_hex_color(&self.accent_color, "Accent color")?,
            highlight_color: parse_hex_color(&self.highlight_color, "Highlight color")?,
            error_color: parse_hex_color(&self.error_color, "Error color")?,
        };

        if settings.max_displayed_results == 0 {
            return Err(String::from("Max results must be at least 1."));
        }

        if settings.max_search_events_per_tick == 0 {
            return Err(String::from(
                "Max search events per tick must be at least 1.",
            ));
        }

        if settings.search_event_limit_multiplier == 0 {
            return Err(String::from(
                "Search event limit multiplier must be at least 1.",
            ));
        }

        if settings.max_history_items == 0 {
            return Err(String::from("Max history items must be at least 1."));
        }

        if settings.min_query_chars == 0 {
            return Err(String::from("Minimum query characters must be at least 1."));
        }

        if settings.window_width < 360.0 {
            return Err(String::from("Window width must be at least 360."));
        }

        if settings.idle_height < 120.0
            || settings.compact_height < settings.idle_height
            || settings.expanded_height < settings.compact_height
            || settings.result_row_height < 44.0
        {
            return Err(String::from(
                "Window and row sizes are outside the supported range.",
            ));
        }

        Ok(settings)
    }
}

impl AppSettings {
    pub fn excluded_folder_patterns(&self) -> Vec<String> {
        comma_list_items(&self.excluded_folders)
    }
}

pub fn load() -> AppSettings {
    let Some(path) = settings_path() else {
        return AppSettings::default();
    };

    fs::read_to_string(path)
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

pub fn save(settings: &AppSettings) -> Result<PathBuf, String> {
    let path = settings_path().ok_or_else(|| String::from("Could not locate settings folder."))?;
    let parent = path
        .parent()
        .ok_or_else(|| String::from("Could not locate settings folder."))?;

    fs::create_dir_all(parent).map_err(|error| error.to_string())?;

    let text = serde_json::to_string_pretty(settings).map_err(|error| error.to_string())?;
    fs::write(&path, text).map_err(|error| error.to_string())?;

    Ok(path)
}

pub fn settings_path() -> Option<PathBuf> {
    home_dir().map(|home| home.join(".deeplens").join("settings.json"))
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

fn parse_usize(value: &str, label: &str) -> Result<usize, String> {
    value
        .trim()
        .parse()
        .map_err(|_| format!("{label} must be a whole number."))
}

fn parse_u64(value: &str, label: &str) -> Result<u64, String> {
    value
        .trim()
        .parse()
        .map_err(|_| format!("{label} must be a whole number."))
}

fn parse_bool(value: &str, label: &str) -> Result<bool, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "yes" | "on" | "1" => Ok(true),
        "false" | "no" | "off" | "0" => Ok(false),
        _ => Err(format!("{label} must be true or false.")),
    }
}

fn parse_f32(value: &str, label: &str) -> Result<f32, String> {
    value
        .trim()
        .parse()
        .map_err(|_| format!("{label} must be a number."))
}

fn parse_hex_color(value: &str, label: &str) -> Result<String, String> {
    let value = value.trim();
    let hex = value.strip_prefix('#').unwrap_or(value);

    if hex.len() == 6 && hex.chars().all(|character| character.is_ascii_hexdigit()) {
        Ok(format!("#{hex}").to_ascii_uppercase())
    } else {
        Err(format!("{label} must be a hex color like #111214."))
    }
}

fn parse_non_empty_text(value: &str, label: &str) -> Result<String, String> {
    let value = value.trim();

    if value.is_empty() {
        Err(format!("{label} cannot be empty."))
    } else {
        Ok(value.to_owned())
    }
}

fn parse_comma_list(value: &str, label: &str) -> Result<String, String> {
    let values = comma_list_items(value);

    if values
        .iter()
        .any(|value| value.contains('\n') || value.contains('\r'))
    {
        return Err(format!("{label} must be a comma-separated list."));
    }

    Ok(values.join(", "))
}

fn comma_list_items(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn format_float(value: f32) -> String {
    if value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        value.to_string()
    }
}
