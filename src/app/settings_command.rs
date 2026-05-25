use crate::settings::SettingsForm;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsField {
    MaxDisplayedResults,
    MaxSearchEventsPerTick,
    SearchEventLimitMultiplier,
    HistoryEnabled,
    MaxHistoryItems,
    HistoryFrequencyBoost,
    HistoryRecencyBoost,
    HistoryRecencyDays,
    PinsEnabled,
    MaxPinnedItems,
    PinRankBoost,
    CalculatorEnabled,
    CalculatorRequiresPrefix,
    CalculatorCommand,
    ClipboardHistoryEnabled,
    MaxClipboardItems,
    MaxClipboardTextBytes,
    ClipboardPollMs,
    CustomCommandsEnabled,
    MaxCustomCommandResults,
    PreviewEnabled,
    ActionsEnabled,
    SearchDebounceMs,
    DoubleClickMs,
    MinQueryChars,
    WindowWidth,
    IdleHeight,
    CompactHeight,
    ExpandedHeight,
    ResultRowHeight,
    TerminalApp,
    ExcludedFolders,
    WebSearchShortcuts,
    GlobalShortcut,
    SubmitShortcut,
    PreviousResultShortcut,
    NextResultShortcut,
    AcceptCompletionShortcuts,
    CancelShortcut,
    SettingsShortcut,
    RevealShortcut,
    CopyPathShortcut,
    TerminalShortcut,
    PreviewShortcut,
    ActionsShortcut,
    SurfaceColor,
    PromptColor,
    TextColor,
    MutedColor,
    FaintColor,
    BorderColor,
    SelectedColor,
    ChipColor,
    AccentColor,
    HighlightColor,
    ErrorColor,
}

impl SettingsField {
    pub(super) fn canonical_name(self) -> &'static str {
        match self {
            SettingsField::MaxDisplayedResults => "max_displayed_results",
            SettingsField::MaxSearchEventsPerTick => "max_search_events_per_tick",
            SettingsField::SearchEventLimitMultiplier => "search_event_limit_multiplier",
            SettingsField::HistoryEnabled => "history_enabled",
            SettingsField::MaxHistoryItems => "max_history_items",
            SettingsField::HistoryFrequencyBoost => "history_frequency_boost",
            SettingsField::HistoryRecencyBoost => "history_recency_boost",
            SettingsField::HistoryRecencyDays => "history_recency_days",
            SettingsField::PinsEnabled => "pins_enabled",
            SettingsField::MaxPinnedItems => "max_pinned_items",
            SettingsField::PinRankBoost => "pin_rank_boost",
            SettingsField::CalculatorEnabled => "calculator_enabled",
            SettingsField::CalculatorRequiresPrefix => "calculator_requires_prefix",
            SettingsField::CalculatorCommand => "calculator_command",
            SettingsField::ClipboardHistoryEnabled => "clipboard_history_enabled",
            SettingsField::MaxClipboardItems => "max_clipboard_items",
            SettingsField::MaxClipboardTextBytes => "max_clipboard_text_bytes",
            SettingsField::ClipboardPollMs => "clipboard_poll_ms",
            SettingsField::CustomCommandsEnabled => "custom_commands_enabled",
            SettingsField::MaxCustomCommandResults => "max_custom_command_results",
            SettingsField::PreviewEnabled => "preview_enabled",
            SettingsField::ActionsEnabled => "actions_enabled",
            SettingsField::SearchDebounceMs => "search_debounce_ms",
            SettingsField::DoubleClickMs => "double_click_ms",
            SettingsField::MinQueryChars => "min_query_chars",
            SettingsField::WindowWidth => "window_width",
            SettingsField::IdleHeight => "idle_height",
            SettingsField::CompactHeight => "compact_height",
            SettingsField::ExpandedHeight => "expanded_height",
            SettingsField::ResultRowHeight => "result_row_height",
            SettingsField::TerminalApp => "terminal_app",
            SettingsField::ExcludedFolders => "excluded_folders",
            SettingsField::WebSearchShortcuts => "web_search_shortcuts",
            SettingsField::GlobalShortcut => "global_shortcut",
            SettingsField::SubmitShortcut => "submit_shortcut",
            SettingsField::PreviousResultShortcut => "previous_result_shortcut",
            SettingsField::NextResultShortcut => "next_result_shortcut",
            SettingsField::AcceptCompletionShortcuts => "accept_completion_shortcuts",
            SettingsField::CancelShortcut => "cancel_shortcut",
            SettingsField::SettingsShortcut => "settings_shortcut",
            SettingsField::RevealShortcut => "reveal_shortcut",
            SettingsField::CopyPathShortcut => "copy_path_shortcut",
            SettingsField::TerminalShortcut => "terminal_shortcut",
            SettingsField::PreviewShortcut => "preview_shortcut",
            SettingsField::ActionsShortcut => "actions_shortcut",
            SettingsField::SurfaceColor => "surface_color",
            SettingsField::PromptColor => "prompt_color",
            SettingsField::TextColor => "text_color",
            SettingsField::MutedColor => "muted_color",
            SettingsField::FaintColor => "faint_color",
            SettingsField::BorderColor => "border_color",
            SettingsField::SelectedColor => "selected_color",
            SettingsField::ChipColor => "chip_color",
            SettingsField::AccentColor => "accent_color",
            SettingsField::HighlightColor => "highlight_color",
            SettingsField::ErrorColor => "error_color",
        }
    }

    fn from_name(name: &str) -> Option<Self> {
        match normalized_setting_name(name).as_str() {
            "maxdisplayedresults" | "maxresults" => Some(Self::MaxDisplayedResults),
            "maxsearcheventspertick" | "maxeventspertick" | "eventbatch" => {
                Some(Self::MaxSearchEventsPerTick)
            }
            "searcheventlimitmultiplier" | "eventlimitmultiplier" | "matchlimitmultiplier" => {
                Some(Self::SearchEventLimitMultiplier)
            }
            "historyenabled" | "history" | "recentsenabled" | "recents" => {
                Some(Self::HistoryEnabled)
            }
            "maxhistoryitems" | "historyitems" | "maxrecents" => Some(Self::MaxHistoryItems),
            "historyfrequencyboost" | "frequencyboost" | "frequentboost" => {
                Some(Self::HistoryFrequencyBoost)
            }
            "historyrecencyboost" | "recencyboost" | "recentboost" => {
                Some(Self::HistoryRecencyBoost)
            }
            "historyrecencydays" | "recencydays" | "recentdays" => Some(Self::HistoryRecencyDays),
            "pinsenabled" | "pins" => Some(Self::PinsEnabled),
            "maxpinneditems" | "pinneditems" | "maxpins" => Some(Self::MaxPinnedItems),
            "pinrankboost" | "pinboost" => Some(Self::PinRankBoost),
            "calculatorenabled" | "calculator" | "calc" => Some(Self::CalculatorEnabled),
            "calculatorrequiresprefix" | "calcrequiresprefix" | "calcprefix" => {
                Some(Self::CalculatorRequiresPrefix)
            }
            "calculatorcommand" | "calccommand" | "numbatcommand" => Some(Self::CalculatorCommand),
            "clipboardhistoryenabled" | "clipboardhistory" | "clipboard" | "clips" => {
                Some(Self::ClipboardHistoryEnabled)
            }
            "maxclipboarditems" | "clipboarditems" | "maxclips" => Some(Self::MaxClipboardItems),
            "maxclipboardtextbytes" | "clipboardtextbytes" | "maxclipbytes" => {
                Some(Self::MaxClipboardTextBytes)
            }
            "clipboardpollms" | "clippoll" | "clipboardpoll" => Some(Self::ClipboardPollMs),
            "customcommandsenabled" | "customcommands" | "commands" | "cmds" => {
                Some(Self::CustomCommandsEnabled)
            }
            "maxcustomcommandresults" | "maxcommands" | "commandresults" => {
                Some(Self::MaxCustomCommandResults)
            }
            "previewenabled" | "preview" | "quicklook" => Some(Self::PreviewEnabled),
            "actionsenabled" | "actions" | "actionsmenu" => Some(Self::ActionsEnabled),
            "searchdebouncems" | "debouncems" | "debounce" => Some(Self::SearchDebounceMs),
            "doubleclickms" | "doubleclick" => Some(Self::DoubleClickMs),
            "minquerychars" | "minchars" => Some(Self::MinQueryChars),
            "windowwidth" | "width" => Some(Self::WindowWidth),
            "idleheight" => Some(Self::IdleHeight),
            "compactheight" => Some(Self::CompactHeight),
            "expandedheight" => Some(Self::ExpandedHeight),
            "resultrowheight" | "rowheight" => Some(Self::ResultRowHeight),
            "terminalapp" | "terminal" => Some(Self::TerminalApp),
            "excludedfolders" | "excludefolders" | "skippedfolders" | "skipfolders" => {
                Some(Self::ExcludedFolders)
            }
            "websearchshortcuts" | "webshortcuts" | "searchshortcuts" => {
                Some(Self::WebSearchShortcuts)
            }
            "globalshortcut" | "globalhotkey" | "showhide" => Some(Self::GlobalShortcut),
            "submitshortcut" | "openshortcut" | "submit" => Some(Self::SubmitShortcut),
            "previousresultshortcut" | "previousshortcut" | "prevshortcut" | "upshortcut" => {
                Some(Self::PreviousResultShortcut)
            }
            "nextresultshortcut" | "nextshortcut" | "downshortcut" => {
                Some(Self::NextResultShortcut)
            }
            "acceptcompletionshortcuts" | "acceptcompletion" | "completionshortcut" => {
                Some(Self::AcceptCompletionShortcuts)
            }
            "cancelshortcut" | "escapeshortcut" => Some(Self::CancelShortcut),
            "settingsshortcut" | "settingshotkey" => Some(Self::SettingsShortcut),
            "revealshortcut" | "reveal" => Some(Self::RevealShortcut),
            "copypathshortcut" | "copyshortcut" | "copy" => Some(Self::CopyPathShortcut),
            "terminalshortcut" | "terminalhotkey" => Some(Self::TerminalShortcut),
            "previewshortcut" | "quicklookshortcut" => Some(Self::PreviewShortcut),
            "actionsshortcut" | "actionshortcut" | "actionsmenuhotkey" => {
                Some(Self::ActionsShortcut)
            }
            "surfacecolor" | "surface" => Some(Self::SurfaceColor),
            "promptcolor" | "prompt" => Some(Self::PromptColor),
            "textcolor" | "text" => Some(Self::TextColor),
            "mutedcolor" | "muted" => Some(Self::MutedColor),
            "faintcolor" | "faint" => Some(Self::FaintColor),
            "bordercolor" | "border" => Some(Self::BorderColor),
            "selectedcolor" | "selected" => Some(Self::SelectedColor),
            "chipcolor" | "chip" => Some(Self::ChipColor),
            "accentcolor" | "accent" => Some(Self::AccentColor),
            "highlightcolor" | "highlight" => Some(Self::HighlightColor),
            "errorcolor" | "error" => Some(Self::ErrorColor),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum SetCommand {
    Usage,
    MissingValue { field: SettingsField },
    UnknownSetting { name: String },
    Ready { field: SettingsField, value: String },
}

pub(super) fn parse_set_command(input: &str) -> Option<SetCommand> {
    let trimmed = input.trim();

    if !trimmed.starts_with("/set") {
        return None;
    }

    let rest = trimmed.strip_prefix("/set").unwrap_or_default();
    if !rest.is_empty() && !rest.starts_with(char::is_whitespace) {
        return None;
    }

    let rest = rest.trim();
    if rest.is_empty() {
        return Some(SetCommand::Usage);
    }

    let mut parts = rest.splitn(2, char::is_whitespace);
    let name = parts.next().unwrap_or_default();
    let Some(field) = SettingsField::from_name(name) else {
        return Some(SetCommand::UnknownSetting {
            name: name.to_owned(),
        });
    };

    let Some(value) = parts
        .next()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Some(SetCommand::MissingValue { field });
    };

    Some(SetCommand::Ready {
        field,
        value: value.to_owned(),
    })
}

pub(super) fn set_command_status(command: SetCommand) -> String {
    match command {
        SetCommand::Usage => format!(
            "Use /set <setting> <value>. Settings: {}",
            SETTINGS_COMMAND_NAMES.join(", ")
        ),
        SetCommand::MissingValue { field } => {
            format!("Use /set {} <value>", field.canonical_name())
        }
        SetCommand::UnknownSetting { name } => format!(
            "Unknown setting {name}. Settings: {}",
            SETTINGS_COMMAND_NAMES.join(", ")
        ),
        SetCommand::Ready { field, value } => {
            format!(
                "Press Enter to set {} to {}",
                field.canonical_name(),
                value.trim()
            )
        }
    }
}

pub(super) fn update_settings_form_value(
    form: &mut SettingsForm,
    field: SettingsField,
    value: String,
) {
    match field {
        SettingsField::MaxDisplayedResults => form.max_displayed_results = value,
        SettingsField::MaxSearchEventsPerTick => form.max_search_events_per_tick = value,
        SettingsField::SearchEventLimitMultiplier => form.search_event_limit_multiplier = value,
        SettingsField::HistoryEnabled => form.history_enabled = value,
        SettingsField::MaxHistoryItems => form.max_history_items = value,
        SettingsField::HistoryFrequencyBoost => form.history_frequency_boost = value,
        SettingsField::HistoryRecencyBoost => form.history_recency_boost = value,
        SettingsField::HistoryRecencyDays => form.history_recency_days = value,
        SettingsField::PinsEnabled => form.pins_enabled = value,
        SettingsField::MaxPinnedItems => form.max_pinned_items = value,
        SettingsField::PinRankBoost => form.pin_rank_boost = value,
        SettingsField::CalculatorEnabled => form.calculator_enabled = value,
        SettingsField::CalculatorRequiresPrefix => form.calculator_requires_prefix = value,
        SettingsField::CalculatorCommand => form.calculator_command = value,
        SettingsField::ClipboardHistoryEnabled => form.clipboard_history_enabled = value,
        SettingsField::MaxClipboardItems => form.max_clipboard_items = value,
        SettingsField::MaxClipboardTextBytes => form.max_clipboard_text_bytes = value,
        SettingsField::ClipboardPollMs => form.clipboard_poll_ms = value,
        SettingsField::CustomCommandsEnabled => form.custom_commands_enabled = value,
        SettingsField::MaxCustomCommandResults => form.max_custom_command_results = value,
        SettingsField::PreviewEnabled => form.preview_enabled = value,
        SettingsField::ActionsEnabled => form.actions_enabled = value,
        SettingsField::SearchDebounceMs => form.search_debounce_ms = value,
        SettingsField::DoubleClickMs => form.double_click_ms = value,
        SettingsField::MinQueryChars => form.min_query_chars = value,
        SettingsField::WindowWidth => form.window_width = value,
        SettingsField::IdleHeight => form.idle_height = value,
        SettingsField::CompactHeight => form.compact_height = value,
        SettingsField::ExpandedHeight => form.expanded_height = value,
        SettingsField::ResultRowHeight => form.result_row_height = value,
        SettingsField::TerminalApp => form.terminal_app = value,
        SettingsField::ExcludedFolders => form.excluded_folders = value,
        SettingsField::WebSearchShortcuts => form.web_search_shortcuts = value,
        SettingsField::GlobalShortcut => form.global_shortcut = value,
        SettingsField::SubmitShortcut => form.submit_shortcut = value,
        SettingsField::PreviousResultShortcut => form.previous_result_shortcut = value,
        SettingsField::NextResultShortcut => form.next_result_shortcut = value,
        SettingsField::AcceptCompletionShortcuts => form.accept_completion_shortcuts = value,
        SettingsField::CancelShortcut => form.cancel_shortcut = value,
        SettingsField::SettingsShortcut => form.settings_shortcut = value,
        SettingsField::RevealShortcut => form.reveal_shortcut = value,
        SettingsField::CopyPathShortcut => form.copy_path_shortcut = value,
        SettingsField::TerminalShortcut => form.terminal_shortcut = value,
        SettingsField::PreviewShortcut => form.preview_shortcut = value,
        SettingsField::ActionsShortcut => form.actions_shortcut = value,
        SettingsField::SurfaceColor => form.surface_color = value,
        SettingsField::PromptColor => form.prompt_color = value,
        SettingsField::TextColor => form.text_color = value,
        SettingsField::MutedColor => form.muted_color = value,
        SettingsField::FaintColor => form.faint_color = value,
        SettingsField::BorderColor => form.border_color = value,
        SettingsField::SelectedColor => form.selected_color = value,
        SettingsField::ChipColor => form.chip_color = value,
        SettingsField::AccentColor => form.accent_color = value,
        SettingsField::HighlightColor => form.highlight_color = value,
        SettingsField::ErrorColor => form.error_color = value,
    }
}

fn normalized_setting_name(name: &str) -> String {
    name.chars()
        .filter(|character| *character != '_' && *character != '-')
        .flat_map(char::to_lowercase)
        .collect()
}

const SETTINGS_COMMAND_NAMES: &[&str] = &[
    "max_displayed_results",
    "max_search_events_per_tick",
    "search_event_limit_multiplier",
    "history_enabled",
    "max_history_items",
    "history_frequency_boost",
    "history_recency_boost",
    "history_recency_days",
    "pins_enabled",
    "max_pinned_items",
    "pin_rank_boost",
    "calculator_enabled",
    "calculator_requires_prefix",
    "calculator_command",
    "clipboard_history_enabled",
    "max_clipboard_items",
    "max_clipboard_text_bytes",
    "clipboard_poll_ms",
    "custom_commands_enabled",
    "max_custom_command_results",
    "preview_enabled",
    "actions_enabled",
    "search_debounce_ms",
    "double_click_ms",
    "min_query_chars",
    "window_width",
    "idle_height",
    "compact_height",
    "expanded_height",
    "result_row_height",
    "terminal_app",
    "excluded_folders",
    "web_search_shortcuts",
    "global_shortcut",
    "submit_shortcut",
    "previous_result_shortcut",
    "next_result_shortcut",
    "accept_completion_shortcuts",
    "cancel_shortcut",
    "settings_shortcut",
    "reveal_shortcut",
    "copy_path_shortcut",
    "terminal_shortcut",
    "preview_shortcut",
    "actions_shortcut",
    "surface_color",
    "prompt_color",
    "text_color",
    "muted_color",
    "faint_color",
    "border_color",
    "selected_color",
    "chip_color",
    "accent_color",
    "highlight_color",
    "error_color",
];

#[cfg(test)]
mod tests {
    use super::{parse_set_command, SetCommand, SettingsField};

    #[test]
    fn parses_set_command() {
        assert_eq!(
            parse_set_command("/set result_row_height 90"),
            Some(SetCommand::Ready {
                field: SettingsField::ResultRowHeight,
                value: String::from("90"),
            })
        );
    }

    #[test]
    fn parses_set_aliases() {
        assert_eq!(
            parse_set_command("/set row-height 90"),
            Some(SetCommand::Ready {
                field: SettingsField::ResultRowHeight,
                value: String::from("90"),
            })
        );

        assert_eq!(
            parse_set_command("/set debounce 300"),
            Some(SetCommand::Ready {
                field: SettingsField::SearchDebounceMs,
                value: String::from("300"),
            })
        );

        assert_eq!(
            parse_set_command("/set terminal_app iTerm"),
            Some(SetCommand::Ready {
                field: SettingsField::TerminalApp,
                value: String::from("iTerm"),
            })
        );

        assert_eq!(
            parse_set_command("/set skip-folders node_modules, target"),
            Some(SetCommand::Ready {
                field: SettingsField::ExcludedFolders,
                value: String::from("node_modules, target"),
            })
        );

        assert_eq!(
            parse_set_command("/set event-batch 120"),
            Some(SetCommand::Ready {
                field: SettingsField::MaxSearchEventsPerTick,
                value: String::from("120"),
            })
        );

        assert_eq!(
            parse_set_command("/set match-limit-multiplier 10"),
            Some(SetCommand::Ready {
                field: SettingsField::SearchEventLimitMultiplier,
                value: String::from("10"),
            })
        );

        assert_eq!(
            parse_set_command("/set recents off"),
            Some(SetCommand::Ready {
                field: SettingsField::HistoryEnabled,
                value: String::from("off"),
            })
        );

        assert_eq!(
            parse_set_command("/set recent-boost 250"),
            Some(SetCommand::Ready {
                field: SettingsField::HistoryRecencyBoost,
                value: String::from("250"),
            })
        );

        assert_eq!(
            parse_set_command("/set web-shortcuts g=https://example.com?q={query}"),
            Some(SetCommand::Ready {
                field: SettingsField::WebSearchShortcuts,
                value: String::from("g=https://example.com?q={query}"),
            })
        );

        assert_eq!(
            parse_set_command("/set calc-prefix true"),
            Some(SetCommand::Ready {
                field: SettingsField::CalculatorRequiresPrefix,
                value: String::from("true"),
            })
        );

        assert_eq!(
            parse_set_command("/set preview-shortcut space"),
            Some(SetCommand::Ready {
                field: SettingsField::PreviewShortcut,
                value: String::from("space"),
            })
        );

        assert_eq!(
            parse_set_command("/set actions-shortcut cmd+k"),
            Some(SetCommand::Ready {
                field: SettingsField::ActionsShortcut,
                value: String::from("cmd+k"),
            })
        );

        assert_eq!(
            parse_set_command("/set pin-boost 2000"),
            Some(SetCommand::Ready {
                field: SettingsField::PinRankBoost,
                value: String::from("2000"),
            })
        );

        assert_eq!(
            parse_set_command("/set clipboard off"),
            Some(SetCommand::Ready {
                field: SettingsField::ClipboardHistoryEnabled,
                value: String::from("off"),
            })
        );

        assert_eq!(
            parse_set_command("/set max-clips 50"),
            Some(SetCommand::Ready {
                field: SettingsField::MaxClipboardItems,
                value: String::from("50"),
            })
        );

        assert_eq!(
            parse_set_command("/set commands off"),
            Some(SetCommand::Ready {
                field: SettingsField::CustomCommandsEnabled,
                value: String::from("off"),
            })
        );

        assert_eq!(
            parse_set_command("/set max-commands 20"),
            Some(SetCommand::Ready {
                field: SettingsField::MaxCustomCommandResults,
                value: String::from("20"),
            })
        );
    }

    #[test]
    fn ignores_other_slash_commands() {
        assert_eq!(parse_set_command("/settings row_height 90"), None);
    }
}
