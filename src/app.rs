mod ghost;
mod settings_command;
mod system_actions;

use crate::app::ghost::ghost_completion;
pub use crate::app::settings_command::SettingsField;
use crate::app::settings_command::{
    parse_set_command, set_command_status, update_settings_form_value, SetCommand,
};
use crate::app::system_actions::{
    open_terminal_at, reveal_in_file_manager, terminal_directory_for,
};
use crate::history::SearchHistory;
use crate::query::{self, ParsedQuery};
use crate::ranking;
use crate::search::{self, SearchHandle, SearchOptions};
use crate::settings::{self, AppSettings, SettingsForm};
use crate::shortcuts::{self, GlobalShortcut};
use crate::types::{GroupedSearchResult, SearchEvent, SearchMode, SearchResultKind};

use iced::widget::operation::AbsoluteOffset;
use iced::widget::scrollable as scrollable_style;
use iced::widget::{
    button, column, container, image, mouse_area, operation, row, scrollable, text, text_input, Id,
    Space,
};
use iced::widget::{
    button as button_style, container as container_style, text_input as input_style,
};
use iced::{
    border, clipboard, event, font, theme, time, window, Alignment, Background, Border, Color,
    Element, Event, Font, Length, Padding, Size, Subscription, Task, Theme,
};
use rfd::FileDialog;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const SCROLL_KEEP_VISIBLE_MARGIN_ROWS: f32 = 2.0;
const SURFACE: Color = Color::from_rgb(0.065, 0.068, 0.078);
const PROMPT: Color = Color::from_rgb(0.070, 0.073, 0.083);
const TEXT: Color = Color::from_rgb(0.88, 0.88, 0.86);
const MUTED: Color = Color::from_rgb(0.62, 0.62, 0.60);
const FAINT: Color = Color::from_rgb(0.36, 0.37, 0.39);
const BORDER: Color = Color::from_rgb(0.26, 0.27, 0.30);
const SELECTED: Color = Color::from_rgb(0.145, 0.155, 0.180);
const CHIP: Color = Color::from_rgb(0.14, 0.145, 0.155);
const ACCENT: Color = Color::from_rgb(0.46, 0.56, 0.74);
const HIGHLIGHT: Color = Color::from_rgb(0.70, 0.78, 0.95);
const ERROR: Color = Color::from_rgb(0.96, 0.44, 0.38);
pub fn run() -> iced::Result {
    iced::daemon(DeeplensApp::new, DeeplensApp::update, DeeplensApp::view)
        .title(title)
        .subscription(DeeplensApp::subscription)
        .theme(theme)
        .style(app_style)
        .run()
}

fn title(app: &DeeplensApp, window: window::Id) -> String {
    if app.settings_window_id == Some(window) {
        String::from("DeepLens Settings")
    } else {
        String::from("DeepLens")
    }
}

fn theme(_: &DeeplensApp, _: window::Id) -> Theme {
    Theme::Light
}

fn app_style(_: &DeeplensApp, _: &Theme) -> theme::Style {
    theme::Style {
        background_color: Color::TRANSPARENT,
        text_color: TEXT,
    }
}

fn main_window_settings(settings: &AppSettings) -> window::Settings {
    window::Settings {
        size: Size::new(settings.window_width, settings.idle_height),
        position: window::Position::Centered,
        decorations: false,
        transparent: true,
        resizable: true,
        level: window::Level::AlwaysOnTop,
        ..window::Settings::default()
    }
}

fn settings_window_settings() -> window::Settings {
    let mut settings = window::Settings {
        size: Size::new(560.0, 680.0),
        position: window::Position::Centered,
        decorations: true,
        transparent: false,
        resizable: true,
        level: window::Level::AlwaysOnTop,
        ..window::Settings::default()
    };

    #[cfg(target_os = "macos")]
    {
        settings.platform_specific.title_hidden = true;
        settings.platform_specific.titlebar_transparent = true;
        settings.platform_specific.fullsize_content_view = true;
    }

    settings
}

#[derive(Debug, Clone)]
pub enum Message {
    MainWindowOpened(window::Id),
    SettingsWindowOpened(window::Id),
    WindowClosed(window::Id),
    StartDrag,
    StartSettingsDrag,
    ChooseFolder,
    FolderChosen(Option<PathBuf>),
    QueryChanged(String),
    Submit,
    Cancel,
    Tick,
    SetSearchMode(SearchMode),
    SelectPrevious,
    SelectNext,
    SelectResult(usize),
    HoverResult(usize),
    ClearHoveredResult(usize),
    AcceptGhostCompletion,
    RevealSelected,
    OpenSelectedInTerminal,
    CopySelectedPath,
    CopyInstallCommand,
    RetrySearch,
    ResultsScrolled(scrollable::Viewport),
    ShowSettings,
    CloseSettings,
    SaveSettings,
    ResetSettings,
    SettingsChanged(SettingsField, String),
    SetSettingsTab(SettingsTab),
    FocusSearchInput,
    KeyboardEvent(Event, event::Status, window::Id),
    Escape,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileFilter {
    All,
    Pdf,
    Docs,
    Text,
    Code,
}

impl FileFilter {
    fn search_globs(self) -> Vec<String> {
        match self {
            FileFilter::All => Vec::new(),
            FileFilter::Pdf => vec![String::from("*.pdf")],
            FileFilter::Docs => ["*.doc", "*.docx", "*.rtf", "*.odt", "*.pages"]
                .into_iter()
                .map(String::from)
                .collect(),
            FileFilter::Text => ["*.txt", "*.md", "*.markdown"]
                .into_iter()
                .map(String::from)
                .collect(),
            FileFilter::Code => [
                "*.rs", "*.toml", "*.json", "*.yaml", "*.yml", "*.js", "*.ts", "*.tsx", "*.jsx",
            ]
            .into_iter()
            .map(String::from)
            .collect(),
        }
    }
}

impl SearchMode {
    const ALL: [SearchMode; 4] = [
        SearchMode::All,
        SearchMode::Applications,
        SearchMode::Folders,
        SearchMode::Files,
    ];

    fn label(self) -> &'static str {
        match self {
            SearchMode::Files => "Files",
            SearchMode::Folders => "Folders",
            SearchMode::Applications => "Apps",
            SearchMode::All => "All",
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum NavigationDirection {
    Up,
    Down,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsTab {
    General,
    Style,
    Shortcuts,
}

impl SettingsTab {
    const ALL: [SettingsTab; 3] = [
        SettingsTab::General,
        SettingsTab::Style,
        SettingsTab::Shortcuts,
    ];

    fn label(self) -> &'static str {
        match self {
            SettingsTab::General => "General",
            SettingsTab::Style => "Style",
            SettingsTab::Shortcuts => "Shortcuts",
        }
    }
}

pub struct DeeplensApp {
    window_id: Option<window::Id>,
    settings_window_id: Option<window::Id>,
    window_height: f32,
    hidden: bool,
    main_window_focused: bool,
    macos_app_active: bool,
    ignore_next_main_unfocus: bool,
    global_shortcut: Option<GlobalShortcut>,
    selected_folder: Option<PathBuf>,
    query: String,
    last_search_query: String,
    active_search_text: String,
    active_exact_phrase: Option<String>,
    pending_search: bool,
    last_query_change: Option<Instant>,
    search_mode: SearchMode,
    results: Vec<GroupedSearchResult>,
    result_count: usize,
    selected_result: Option<usize>,
    selected_result_path: Option<PathBuf>,
    user_selected_result: bool,
    hovered_result_path: Option<PathBuf>,
    last_result_click: Option<(usize, Instant)>,
    results_scroll_y: f32,
    results_view_height: f32,
    search: Option<SearchHandle>,
    running: bool,
    status: String,
    settings: AppSettings,
    settings_form: SettingsForm,
    settings_tab: SettingsTab,
    history: SearchHistory,
}

impl DeeplensApp {
    fn new() -> (Self, Task<Message>) {
        let settings = settings::load();
        let settings_form = SettingsForm::from(&settings);
        let idle_height = settings.idle_height;
        let history = SearchHistory::load();
        let (main_window_id, open_main_window) = window::open(main_window_settings(&settings));

        (
            Self {
                window_id: Some(main_window_id),
                settings_window_id: None,
                window_height: idle_height,
                hidden: false,
                main_window_focused: false,
                macos_app_active: platform_app_is_active(),
                ignore_next_main_unfocus: false,
                global_shortcut: GlobalShortcut::register(&settings.global_shortcut).ok(),
                selected_folder: Some(default_scope()),
                query: String::new(),
                last_search_query: String::new(),
                active_search_text: String::new(),
                active_exact_phrase: None,
                pending_search: false,
                last_query_change: None,
                search_mode: SearchMode::All,
                results: Vec::new(),
                result_count: 0,
                selected_result: None,
                selected_result_path: None,
                user_selected_result: false,
                hovered_result_path: None,
                last_result_click: None,
                results_scroll_y: 0.0,
                results_view_height: 360.0,
                search: None,
                running: false,
                status: idle_status(settings.min_query_chars),
                settings,
                settings_form,
                settings_tab: SettingsTab::General,
                history,
            },
            open_main_window.map(Message::MainWindowOpened),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        let mut task = Task::none();

        match message {
            Message::MainWindowOpened(id) => {
                self.window_id = Some(id);
                task = operation::focus(search_input_id());
            }
            Message::SettingsWindowOpened(id) => {
                self.settings_window_id = Some(id);
            }
            Message::WindowClosed(id) => {
                if self.settings_window_id == Some(id) {
                    self.settings_window_id = None;
                    if let Some(main_id) = self.window_id {
                        task = window::set_level(main_id, window::Level::AlwaysOnTop);
                    }
                }
            }
            Message::StartDrag => {
                if let Some(id) = self.window_id {
                    task = window::drag(id);
                }
            }
            Message::StartSettingsDrag => {
                if let Some(id) = self.settings_window_id {
                    task = window::drag(id);
                }
            }
            Message::ChooseFolder => {
                self.ignore_next_main_unfocus = true;
                task = Task::perform(choose_folder(), Message::FolderChosen);
            }
            Message::FolderChosen(Some(path)) => {
                self.ignore_next_main_unfocus = false;
                self.selected_folder = Some(path);
                self.schedule_search();
            }
            Message::FolderChosen(None) => {
                self.ignore_next_main_unfocus = false;
            }
            Message::QueryChanged(query) => {
                self.query = query;
                self.schedule_search();
                task = operation::focus(search_input_id());
            }
            Message::Submit => {
                task = self.submit();
            }
            Message::Cancel => {
                self.cancel_search();
            }
            Message::Tick => {
                let app_active = platform_app_is_active();
                if self.hidden && app_active && !self.macos_app_active {
                    task = self.show_window();
                }
                self.macos_app_active = app_active;

                if self
                    .global_shortcut
                    .as_ref()
                    .is_some_and(GlobalShortcut::was_pressed)
                {
                    task = self.toggle_visibility();
                }

                self.drain_search_events();
                self.run_pending_search();
            }
            Message::SetSearchMode(mode) => {
                self.search_mode = mode;
                self.user_selected_result = false;
                self.hovered_result_path = None;
                self.schedule_search();
                task = operation::focus(search_input_id());
            }
            Message::SelectPrevious => {
                self.user_selected_result = true;
                self.hovered_result_path = None;
                task = self.select_previous();
            }
            Message::SelectNext => {
                self.user_selected_result = true;
                self.hovered_result_path = None;
                task = self.select_next();
            }
            Message::SelectResult(index) => {
                self.user_selected_result = true;
                self.select_or_open_result(index);
            }
            Message::HoverResult(index) => {
                self.hovered_result_path =
                    self.results.get(index).map(|result| result.path.clone());
            }
            Message::ClearHoveredResult(index) => {
                if self
                    .results
                    .get(index)
                    .is_some_and(|result| self.hovered_result_path.as_ref() == Some(&result.path))
                {
                    self.hovered_result_path = None;
                }
            }
            Message::AcceptGhostCompletion => {
                self.accept_ghost_completion();
            }
            Message::RevealSelected => {
                self.reveal_selected();
            }
            Message::OpenSelectedInTerminal => {
                self.open_selected_in_terminal();
            }
            Message::CopySelectedPath => {
                if let Some(path) = self.selected_path() {
                    self.status = String::from("Path copied");
                    task = clipboard::write(path.to_string_lossy().to_string());
                }
            }
            Message::CopyInstallCommand => {
                self.status = String::from("Install command copied");
                task = clipboard::write(String::from("brew install ripgrep-all"));
            }
            Message::RetrySearch => {
                self.pending_search = false;
                self.start_search();
            }
            Message::ResultsScrolled(viewport) => {
                self.results_scroll_y = viewport.absolute_offset().y;
                self.results_view_height = viewport.bounds().height;
            }
            Message::ShowSettings => {
                self.settings_form = SettingsForm::from(&self.settings);
                if let Some(id) = self.settings_window_id {
                    let mut tasks = vec![
                        window::set_level(id, window::Level::AlwaysOnTop),
                        window::gain_focus(id),
                    ];

                    if let Some(main_id) = self.window_id {
                        tasks.push(window::set_level(main_id, window::Level::Normal));
                    }

                    task = Task::batch(tasks);
                } else {
                    let (id, open_settings) = window::open(settings_window_settings());
                    self.settings_window_id = Some(id);
                    let mut tasks = vec![
                        open_settings.map(Message::SettingsWindowOpened),
                        window::gain_focus(id),
                    ];

                    if let Some(main_id) = self.window_id {
                        tasks.push(window::set_level(main_id, window::Level::Normal));
                    }

                    task = Task::batch(tasks);
                }
            }
            Message::CloseSettings => {
                if let Some(id) = self.settings_window_id.take() {
                    let mut tasks = vec![window::close(id)];

                    if let Some(main_id) = self.window_id {
                        tasks.push(window::set_level(main_id, window::Level::AlwaysOnTop));
                    }

                    task = Task::batch(tasks);
                }
            }
            Message::SaveSettings => {
                task = self.save_settings();
            }
            Message::ResetSettings => {
                self.settings = AppSettings::default();
                self.settings_form = SettingsForm::from(&self.settings);
                let shortcut_error = self.reload_global_shortcut().err();
                match settings::save(&self.settings) {
                    Ok(path) => self.status = format!("Settings saved to {}", path.display()),
                    Err(error) => self.status = format_error_status(error),
                }
                if let Some(error) = shortcut_error {
                    self.status = format_error_status(error);
                }
            }
            Message::SettingsChanged(field, value) => {
                self.update_settings_form(field, value);
            }
            Message::SetSettingsTab(tab) => {
                self.settings_tab = tab;
            }
            Message::FocusSearchInput => {
                if !self.hidden {
                    if let Some(id) = self.window_id {
                        task = Task::batch([
                            window::gain_focus(id),
                            operation::focus(search_input_id()),
                        ]);
                    } else {
                        task = operation::focus(search_input_id());
                    }
                }
            }
            Message::KeyboardEvent(event, status, window) => {
                if self.window_id != Some(window) {
                    return Task::none();
                }

                if matches!(event, Event::Window(window::Event::Focused)) {
                    self.main_window_focused = true;
                    return Task::none();
                }

                if matches!(event, Event::Window(window::Event::Unfocused)) {
                    if self.ignore_next_main_unfocus {
                        self.ignore_next_main_unfocus = false;
                    } else if self.main_window_focused
                        && !self.hidden
                        && self.settings_window_id.is_none()
                    {
                        task = self.hide_window();
                    }
                    self.main_window_focused = false;

                    return Task::batch([task, self.sync_window_size()]);
                }

                if let Some(message) =
                    shortcuts::keyboard_event(event, status, window, &self.settings)
                {
                    return self.update(message);
                }
            }
            Message::Escape => {
                if self.running {
                    self.cancel_search();
                } else if !self.query.is_empty() {
                    self.query.clear();
                    self.pending_search = false;
                    self.clear_results();
                    self.status = idle_status(self.settings.min_query_chars);
                } else {
                    task = self.hide_window();
                }
            }
        }

        Task::batch([task, self.sync_window_size()])
    }

    fn subscription(&self) -> Subscription<Message> {
        let keyboard = iced::event::listen_with(|event, status, window| {
            Some(Message::KeyboardEvent(event, status, window))
        });
        let windows = window::close_events().map(Message::WindowClosed);
        let hotkey_tick = time::every(Duration::from_millis(120)).map(|_| Message::Tick);

        if self.running || self.pending_search {
            Subscription::batch([
                time::every(Duration::from_millis(75)).map(|_| Message::Tick),
                keyboard,
                windows,
            ])
        } else {
            Subscription::batch([keyboard, windows, hotkey_tick])
        }
    }

    fn view(&self, window: window::Id) -> Element<'_, Message> {
        if self.settings_window_id == Some(window) {
            return self.view_settings_window();
        }

        let folder_label = self
            .selected_folder
            .as_ref()
            .map(|path| compact_path(path))
            .unwrap_or_else(|| String::from("/"));

        let visible_indices = self.visible_indices();
        let visible_count = visible_indices.len();
        let list = visible_indices
            .into_iter()
            .fold(column![].spacing(0), |list, index| {
                let result = &self.results[index];
                list.push(self.view_result_row(
                    index,
                    result,
                    self.is_selected_result(index, result),
                ))
            });

        let mut surface = column![self.view_search_panel(folder_label.clone())]
            .spacing(12)
            .padding(10);

        if self.status == "No results"
            && visible_count == 0
            && !self.running
            && !self.pending_search
        {
            surface = surface.push(
                container(view_no_results_state(&folder_label))
                    .height(Length::Fill)
                    .width(Length::Fill),
            );
        } else if !self.is_fresh_idle_screen() && !self.is_set_command_screen() {
            surface = surface.push(
                container(
                    scrollable(list)
                        .id(results_scroll_id())
                        .on_scroll(Message::ResultsScrolled)
                        .style(results_scrollable_style)
                        .height(Length::Fill)
                        .width(Length::Fill),
                )
                .height(Length::Fill)
                .padding([0, 2]),
            );
        }

        if self.rga_is_missing() {
            surface = surface.push(self.view_missing_rga_state());
        } else if self.status_is_error() {
            surface = surface.push(text(&self.status).size(13).color(ERROR));
        }

        let rounded_screen = mouse_area(
            container(surface)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(surface_style),
        )
        .on_press(Message::StartDrag);

        container(rounded_screen)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(8)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }

    fn view_search_panel(&self, folder_label: String) -> Element<'_, Message> {
        let search_input = row![text("⌕").size(38).color(MUTED), self.view_search_input(),]
            .spacing(10)
            .align_y(Alignment::Center);

        let mode_chips = SearchMode::ALL.iter().fold(row![].spacing(6), |row, mode| {
            row.push(self.view_mode_chip(*mode))
        });

        let mut controls = row![
            button(text("+").size(18))
                .padding([3, 9])
                .style(icon_button)
                .on_press(Message::ChooseFolder),
            scope_pill(folder_label),
            mode_chips,
            Space::new().width(Length::Fill),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        if self.running {
            controls = controls.push(
                button(text("Cancel").size(12))
                    .padding([6, 10])
                    .style(chip_button)
                    .on_press(Message::Cancel),
            );
        }

        container(
            column![search_input, controls, self.view_status_label()]
                .spacing(10)
                .padding([10, 14]),
        )
        .width(Length::Fill)
        .style(prompt_style)
        .into()
    }

    fn view_search_input(&self) -> Element<'_, Message> {
        let input: Element<'_, Message> = text_input("Ask anything", &self.query)
            .id(search_input_id())
            .on_input(Message::QueryChanged)
            .padding([10, 8])
            .size(22)
            .style(search_input_style)
            .width(Length::Fill)
            .into();

        let suffix: Element<'_, Message> = self
            .ghost_completion()
            .map(|suffix| text(suffix).size(22).color(FAINT).into())
            .unwrap_or_else(|| Space::new().width(Length::Fixed(0.0)).into());

        row![input, suffix]
            .spacing(0)
            .align_y(Alignment::Center)
            .width(Length::Fill)
            .into()
    }

    fn view_mode_chip(&self, mode: SearchMode) -> Element<'static, Message> {
        let selected = mode == self.search_mode;

        button(text(mode.label()).size(12))
            .padding([5, 9])
            .style(move |_, status| filter_button_style(status, selected))
            .on_press(Message::SetSearchMode(mode))
            .into()
    }

    fn view_settings_window(&self) -> Element<'_, Message> {
        let path = settings::settings_path()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| String::from("Settings path unavailable"));

        let tabs = SettingsTab::ALL
            .iter()
            .fold(row![].spacing(6), |tabs, tab| {
                tabs.push(self.view_settings_tab(*tab))
            });

        let header = row![
            mouse_area(
                container(text("Settings").size(16).color(TEXT))
                    .width(Length::Fill)
                    .padding([6, 0])
            )
            .on_press(Message::StartSettingsDrag),
            button(text("Reset").size(12))
                .padding([6, 10])
                .style(chip_button)
                .on_press(Message::ResetSettings),
            button(text("Save").size(12))
                .padding([6, 10])
                .style(chip_button)
                .on_press(Message::SaveSettings),
            button(text("Done").size(12))
                .padding([6, 10])
                .style(chip_button)
                .on_press(Message::CloseSettings),
        ]
        .spacing(8)
        .align_y(Alignment::Center);

        let content: Element<'_, Message> = container(
            column![
                header,
                text(path).size(11).color(FAINT),
                tabs,
                scrollable(self.view_settings_fields()).height(Length::Fill),
            ]
            .spacing(10),
        )
        .padding([10, 14])
        .width(Length::Fill)
        .into();

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(settings_window_padding())
            .style(settings_surface_style)
            .into()
    }

    fn view_settings_tab(&self, tab: SettingsTab) -> Element<'static, Message> {
        let selected = tab == self.settings_tab;

        button(text(tab.label()).size(12))
            .padding([6, 12])
            .style(move |_, status| filter_button_style(status, selected))
            .on_press(Message::SetSettingsTab(tab))
            .into()
    }

    fn view_settings_fields(&self) -> Element<'_, Message> {
        match self.settings_tab {
            SettingsTab::General => self.view_general_settings(),
            SettingsTab::Style => self.view_style_settings(),
            SettingsTab::Shortcuts => self.view_shortcut_settings(),
        }
    }

    fn view_general_settings(&self) -> Element<'_, Message> {
        column![
            settings_field(
                "Max results",
                &self.settings_form.max_displayed_results,
                "Caps the number of grouped files shown in the result list.",
                SettingsField::MaxDisplayedResults,
            ),
            settings_field(
                "Search events per tick",
                &self.settings_form.max_search_events_per_tick,
                "Maximum streamed result events the UI processes per update.",
                SettingsField::MaxSearchEventsPerTick,
            ),
            settings_field(
                "Search event limit",
                &self.settings_form.search_event_limit_multiplier,
                "Multiplier for raw streamed matches before a broad search is stopped.",
                SettingsField::SearchEventLimitMultiplier,
            ),
            settings_field(
                "History enabled",
                &self.settings_form.history_enabled,
                "Records opened results and boosts them in future searches.",
                SettingsField::HistoryEnabled,
            ),
            settings_field(
                "Max history items",
                &self.settings_form.max_history_items,
                "Maximum number of opened results stored in history.",
                SettingsField::MaxHistoryItems,
            ),
            settings_field(
                "History frequency boost",
                &self.settings_form.history_frequency_boost,
                "Ranking boost added for each previous open.",
                SettingsField::HistoryFrequencyBoost,
            ),
            settings_field(
                "History recency boost",
                &self.settings_form.history_recency_boost,
                "Maximum ranking boost for recently opened results.",
                SettingsField::HistoryRecencyBoost,
            ),
            settings_field(
                "History recency days",
                &self.settings_form.history_recency_days,
                "Number of days before the recency boost decays to zero.",
                SettingsField::HistoryRecencyDays,
            ),
            settings_field(
                "Debounce ms",
                &self.settings_form.search_debounce_ms,
                "Wait time after typing before a search starts.",
                SettingsField::SearchDebounceMs,
            ),
            settings_field(
                "Double-click ms",
                &self.settings_form.double_click_ms,
                "Maximum time between two clicks that counts as opening a result.",
                SettingsField::DoubleClickMs,
            ),
            settings_field(
                "Min query chars",
                &self.settings_form.min_query_chars,
                "Minimum number of typed characters before searching.",
                SettingsField::MinQueryChars,
            ),
            settings_field(
                "Window width",
                &self.settings_form.window_width,
                "Initial width of the main search window.",
                SettingsField::WindowWidth,
            ),
            settings_field(
                "Idle height",
                &self.settings_form.idle_height,
                "Height when the search box is empty.",
                SettingsField::IdleHeight,
            ),
            settings_field(
                "Compact height",
                &self.settings_form.compact_height,
                "Height for non-result states such as errors.",
                SettingsField::CompactHeight,
            ),
            settings_field(
                "Expanded height",
                &self.settings_form.expanded_height,
                "Height when search results are visible.",
                SettingsField::ExpandedHeight,
            ),
            settings_field(
                "Result row height",
                &self.settings_form.result_row_height,
                "Estimated row height used to keep keyboard selection visible.",
                SettingsField::ResultRowHeight,
            ),
            settings_field(
                "Terminal app",
                &self.settings_form.terminal_app,
                "macOS app name used by the terminal shortcut.",
                SettingsField::TerminalApp,
            ),
            settings_field(
                "Excluded folders",
                &self.settings_form.excluded_folders,
                "Comma-separated folders skipped by file and folder search.",
                SettingsField::ExcludedFolders,
            ),
        ]
        .spacing(8)
        .into()
    }

    fn view_style_settings(&self) -> Element<'_, Message> {
        column![
            settings_field(
                "Surface color",
                &self.settings_form.surface_color,
                "Main window background color as #RRGGBB.",
                SettingsField::SurfaceColor,
            ),
            settings_field(
                "Prompt color",
                &self.settings_form.prompt_color,
                "Search panel and input background color.",
                SettingsField::PromptColor,
            ),
            settings_field(
                "Text color",
                &self.settings_form.text_color,
                "Primary result and control text color.",
                SettingsField::TextColor,
            ),
            settings_field(
                "Muted color",
                &self.settings_form.muted_color,
                "Secondary labels, paths, and status text color.",
                SettingsField::MutedColor,
            ),
            settings_field(
                "Faint color",
                &self.settings_form.faint_color,
                "Low-emphasis helper and metadata text color.",
                SettingsField::FaintColor,
            ),
            settings_field(
                "Border color",
                &self.settings_form.border_color,
                "Borders around surfaces and selected chips.",
                SettingsField::BorderColor,
            ),
            settings_field(
                "Selected color",
                &self.settings_form.selected_color,
                "Highlighted result row background color.",
                SettingsField::SelectedColor,
            ),
            settings_field(
                "Chip color",
                &self.settings_form.chip_color,
                "Default chip and small button background color.",
                SettingsField::ChipColor,
            ),
            settings_field(
                "Accent color",
                &self.settings_form.accent_color,
                "Spinner, input selection, and accent UI color.",
                SettingsField::AccentColor,
            ),
            settings_field(
                "Highlight color",
                &self.settings_form.highlight_color,
                "Matched text highlight color inside snippets.",
                SettingsField::HighlightColor,
            ),
            settings_field(
                "Error color",
                &self.settings_form.error_color,
                "Error and failure status text color.",
                SettingsField::ErrorColor,
            ),
        ]
        .spacing(8)
        .into()
    }

    fn view_shortcut_settings(&self) -> Element<'_, Message> {
        column![
            settings_field(
                "Global shortcut",
                &self.settings_form.global_shortcut,
                "Shows or hides DeepLens from anywhere.",
                SettingsField::GlobalShortcut,
            ),
            settings_field(
                "Submit shortcut",
                &self.settings_form.submit_shortcut,
                "Opens the selected result or starts the current search.",
                SettingsField::SubmitShortcut,
            ),
            settings_field(
                "Previous result",
                &self.settings_form.previous_result_shortcut,
                "Moves the highlighted result upward. Repeats while held.",
                SettingsField::PreviousResultShortcut,
            ),
            settings_field(
                "Next result",
                &self.settings_form.next_result_shortcut,
                "Moves the highlighted result downward. Repeats while held.",
                SettingsField::NextResultShortcut,
            ),
            settings_field(
                "Accept completion",
                &self.settings_form.accept_completion_shortcuts,
                "Accepts ghost autocomplete; use commas for alternatives.",
                SettingsField::AcceptCompletionShortcuts,
            ),
            settings_field(
                "Cancel shortcut",
                &self.settings_form.cancel_shortcut,
                "Cancels search, clears input, or hides the window.",
                SettingsField::CancelShortcut,
            ),
            settings_field(
                "Settings shortcut",
                &self.settings_form.settings_shortcut,
                "Opens this settings window.",
                SettingsField::SettingsShortcut,
            ),
            settings_field(
                "Reveal shortcut",
                &self.settings_form.reveal_shortcut,
                "Reveals the selected result in Finder or the file manager.",
                SettingsField::RevealShortcut,
            ),
            settings_field(
                "Copy path shortcut",
                &self.settings_form.copy_path_shortcut,
                "Copies the selected result path.",
                SettingsField::CopyPathShortcut,
            ),
            settings_field(
                "Terminal shortcut",
                &self.settings_form.terminal_shortcut,
                "Opens the selected result folder in the configured terminal.",
                SettingsField::TerminalShortcut,
            ),
        ]
        .spacing(8)
        .into()
    }

    fn view_result_row(
        &self,
        index: usize,
        result: &GroupedSearchResult,
        selected: bool,
    ) -> Element<'_, Message> {
        let query = if self.active_search_text.is_empty() {
            self.parsed_query().search_text
        } else {
            self.active_search_text.clone()
        };

        view_result_row(index, result, selected, &query, self.result_row_height())
    }

    fn view_status_label(&self) -> Element<'_, Message> {
        view_status_label(
            &self.status,
            self.result_count,
            self.visible_result_count(),
            self.running || self.pending_search,
            self.settings.min_query_chars,
        )
    }

    fn view_missing_rga_state(&self) -> Element<'_, Message> {
        container(
            column![
                text("ripgrep-all (rga) is required.").size(14).color(TEXT),
                text("macOS install: brew install ripgrep-all")
                    .size(12)
                    .color(MUTED),
                row![
                    button(text("Copy install command").size(12))
                        .padding([6, 10])
                        .style(chip_button)
                        .on_press(Message::CopyInstallCommand),
                    button(text("I installed it, retry").size(12))
                        .padding([6, 10])
                        .style(chip_button)
                        .on_press(Message::RetrySearch),
                ]
                .spacing(8)
            ]
            .spacing(8),
        )
        .padding([2, 8])
        .into()
    }

    fn submit(&mut self) -> Task<Message> {
        if let Some(task) = self.commit_set_command() {
            return task;
        }

        if self.commit_mode_query() {
            return Task::none();
        }

        if self.commit_scope_query() {
            return Task::none();
        }

        if self.should_open_selected() {
            self.open_selected()
        } else {
            self.pending_search = false;
            self.start_search();
            Task::none()
        }
    }

    fn commit_mode_query(&mut self) -> bool {
        let parsed = self.parsed_query();

        if let Some(mode) = parsed.mode.filter(|_| parsed.search_text.trim().is_empty()) {
            self.search_mode = mode;
            self.query.clear();
            self.pending_search = false;
            self.stop_search();
            self.clear_results();
            self.status = format!("Mode set to {}", mode.label());
            return true;
        }

        false
    }

    fn commit_scope_query(&mut self) -> bool {
        let parsed = self.parsed_query();

        if parsed.search_text.trim().is_empty() {
            if let Some(error) = parsed.scope_error {
                self.status = format_error_status(error);
                return true;
            }

            if let Some(scope) = parsed.scope.filter(|scope| scope.exists()) {
                let scope = scope.canonicalize().unwrap_or(scope);
                self.selected_folder = Some(scope.clone());
                self.query.clear();
                self.pending_search = false;
                self.stop_search();
                self.clear_results();
                self.status = format!("Scope set to {}", compact_path(&scope));
                return true;
            }
        }

        false
    }

    fn should_open_selected(&self) -> bool {
        !self.pending_search
            && !self.results.is_empty()
            && (self.selected_result.is_some() || self.selected_result_path.is_some())
            && self.current_query_matches_active_search()
    }

    fn start_search(&mut self) {
        if self.running {
            self.stop_search();
        }

        let parsed = self.parsed_query();

        if let Some(error) = parsed.scope_error.clone() {
            self.clear_results();
            self.status = format_error_status(error);
            return;
        }

        if parsed.search_text.trim().is_empty() {
            self.clear_results();
            self.status = idle_status(self.settings.min_query_chars);
            return;
        }

        if !query_is_ready(&parsed.search_text, self.settings.min_query_chars) {
            self.clear_results();
            self.status = min_query_status(self.settings.min_query_chars);
            return;
        }

        let folder = self
            .selected_folder
            .clone()
            .or_else(|| parsed.scope.clone())
            .unwrap_or_default();

        if let Err(error) = search::validate_search(&self.query, &folder) {
            self.status = error.to_string();
            return;
        }

        let folder = folder.canonicalize().unwrap_or(folder);
        let file_folder_scope = file_folder_scope_for(&folder);

        self.clear_results();
        self.last_search_query = self.query.trim().to_owned();
        self.active_search_text = parsed.search_text.clone();
        self.active_exact_phrase = parsed.exact_phrase.clone();

        let search_filter = parsed.filter.unwrap_or(FileFilter::All);
        let search_mode = parsed.mode.unwrap_or(self.search_mode);

        match search::start_search(SearchOptions {
            query: parsed.search_text,
            file_folder_scope: file_folder_scope.clone(),
            exact: parsed.exact_phrase.is_some(),
            include_globs: search_filter.search_globs(),
            excluded_folders: self.settings.excluded_folder_patterns(),
            mode: search_mode,
        }) {
            Ok(handle) => {
                self.search = Some(handle);
                self.running = true;
                self.status = if folder == Path::new("/") && file_folder_scope != folder {
                    format!(
                        "Searching apps and {}…",
                        self.scope_label_for(&file_folder_scope)
                    )
                } else {
                    format!("Searching {}…", self.scope_label_for(&folder))
                };
            }
            Err(error) => {
                self.search = None;
                self.running = false;
                self.status = format_error_status(error.to_string());
            }
        }
    }

    fn cancel_search(&mut self) {
        self.stop_search();
        self.pending_search = false;
        self.status = String::from("Search cancelled");
    }

    fn stop_search(&mut self) {
        if let Some(search) = self.search.as_mut() {
            search.cancel();
        }

        self.search = None;
        self.running = false;
    }

    fn schedule_search(&mut self) {
        self.last_query_change = Some(Instant::now());

        if let Some(command) = parse_set_command(&self.query) {
            self.pending_search = false;
            self.stop_search();
            self.clear_results();
            self.status = set_command_status(command);
            return;
        }

        let parsed = self.parsed_query();
        self.apply_query_hints(&parsed);

        if let Some(error) = parsed.scope_error {
            self.pending_search = false;
            self.stop_search();
            self.clear_results();
            self.status = format_error_status(error);
            return;
        }

        if parsed.search_text.trim().is_empty() {
            self.pending_search = false;
            self.stop_search();
            self.clear_results();
            self.status = if let Some(mode) = parsed.mode {
                format!("Press Enter to use {}", mode.label())
            } else if let Some(scope) = parsed.scope {
                format!("Press Enter to use {}", compact_path(&scope))
            } else {
                idle_status(self.settings.min_query_chars)
            };
            return;
        }

        if !query_is_ready(&parsed.search_text, self.settings.min_query_chars) {
            self.pending_search = false;
            self.stop_search();
            self.clear_results();
            self.status = min_query_status(self.settings.min_query_chars);
            return;
        }

        self.pending_search = true;
        self.stop_search();
        self.status = String::from("Waiting…");
    }

    fn run_pending_search(&mut self) {
        if !self.pending_search {
            return;
        }

        let Some(last_change) = self.last_query_change else {
            return;
        };

        if last_change.elapsed() >= Duration::from_millis(self.settings.search_debounce_ms) {
            self.pending_search = false;
            self.start_search();
        }
    }

    fn clear_results(&mut self) {
        self.results.clear();
        self.result_count = 0;
        self.selected_result = None;
        self.selected_result_path = None;
        self.user_selected_result = false;
        self.hovered_result_path = None;
        self.last_result_click = None;
    }

    fn drain_search_events(&mut self) {
        if self.search.is_none() {
            return;
        }

        let mut outcome = None;
        let mut drained_events = 0;
        let search_event_limit = self
            .settings
            .max_displayed_results
            .saturating_mul(self.settings.search_event_limit_multiplier)
            .max(self.settings.max_displayed_results);

        while drained_events < self.settings.max_search_events_per_tick {
            let Some(event) = self.search.as_ref().and_then(SearchHandle::try_recv) else {
                break;
            };

            drained_events += 1;

            match event {
                SearchEvent::Result(result) => {
                    self.result_count += 1;

                    if self.results.len() < self.settings.max_displayed_results {
                        let history_boost = self.history.score_for(&result.path, &self.settings);
                        ranking::insert_result(
                            &mut self.results,
                            result,
                            &self.active_search_text,
                            self.active_exact_phrase.as_deref(),
                            history_boost,
                        );

                        if !self.user_selected_result {
                            let _ = self.select_first_visible();
                        } else if self.selected_result_path.is_some() {
                            self.reconcile_selected_result();
                        } else if self.selected_result.is_none() {
                            let _ = self.select_first_visible();
                        }
                    }

                    self.status = found_status(self.result_count, self.visible_result_count());

                    if self.results.len() >= self.settings.max_displayed_results {
                        outcome = Some(SearchOutcome::Finished);
                        break;
                    }

                    if self.result_count >= search_event_limit {
                        outcome = Some(SearchOutcome::Limited);
                        break;
                    }
                }
                SearchEvent::Finished => {
                    outcome = Some(SearchOutcome::Finished);
                }
                SearchEvent::Cancelled => {
                    outcome = Some(SearchOutcome::Cancelled);
                }
                SearchEvent::Error(error) => {
                    outcome = Some(SearchOutcome::Error(format_error_status(error)));
                }
            }
        }

        if let Some(outcome) = outcome {
            self.search = None;
            self.running = false;

            self.status = match outcome {
                SearchOutcome::Finished => {
                    if self.results.len() >= self.settings.max_displayed_results {
                        format!(
                            "Showing first {} files",
                            self.settings.max_displayed_results
                        )
                    } else if self.visible_result_count() == 0 {
                        String::from("No results")
                    } else {
                        found_status(self.result_count, self.visible_result_count())
                    }
                }
                SearchOutcome::Limited => {
                    if self.results.len() >= self.settings.max_displayed_results {
                        format!(
                            "Showing first {} files",
                            self.settings.max_displayed_results
                        )
                    } else {
                        format!("Showing first {} matches", self.result_count)
                    }
                }
                SearchOutcome::Cancelled => String::from("Search cancelled"),
                SearchOutcome::Error(error) => error,
            };
        }
    }

    fn select_previous(&mut self) -> Task<Message> {
        self.reconcile_selected_result();

        if self.selected_result.is_none() {
            return self.select_first_visible();
        };

        let visible = self.visible_indices();
        let Some(position) = self.selected_visible_position(&visible) else {
            return self.select_first_visible();
        };

        let next_position = position.saturating_sub(1);
        if let Some(index) = visible.get(next_position).copied() {
            self.select_result_index(index);
        }
        self.last_result_click = None;
        self.scroll_after_keyboard_navigation(position, next_position, NavigationDirection::Up)
    }

    fn select_next(&mut self) -> Task<Message> {
        self.reconcile_selected_result();

        let visible = self.visible_indices();

        if visible.is_empty() {
            self.selected_result = None;
            self.selected_result_path = None;
            self.last_result_click = None;
            return Task::none();
        }

        if self.selected_result.is_none() {
            return self.select_first_visible();
        }

        let Some(position) = self.selected_visible_position(&visible) else {
            return self.select_first_visible();
        };

        let next_position = (position + 1).min(visible.len() - 1);
        if let Some(index) = visible.get(next_position).copied() {
            self.select_result_index(index);
        }
        self.last_result_click = None;
        self.scroll_after_keyboard_navigation(position, next_position, NavigationDirection::Down)
    }

    fn select_or_open_result(&mut self, index: usize) {
        let now = Instant::now();
        let double_clicked = self
            .last_result_click
            .is_some_and(|(last_index, last_click)| {
                last_index == index
                    && now.duration_since(last_click)
                        <= Duration::from_millis(self.settings.double_click_ms)
            });

        self.select_result_index(index);

        if double_clicked {
            self.last_result_click = None;
            let _ = self.open_result(index, false);
        } else {
            self.last_result_click = Some((index, now));
        }
    }

    fn open_selected(&mut self) -> Task<Message> {
        self.reconcile_selected_result();

        if let Some(index) = self.selected_result {
            if self.running {
                self.stop_search();
            }
            return self.open_result(index, true);
        }

        Task::none()
    }

    fn accept_ghost_completion(&mut self) {
        let Some(query) = self.ghost_query() else {
            return;
        };

        let Some(target) = self.ghost_target(&query) else {
            return;
        };

        if ghost_completion(&query, &target).is_none() {
            return;
        }

        self.query = target;
        self.schedule_search();
    }

    fn open_result(&mut self, index: usize, hide_after_open: bool) -> Task<Message> {
        let Some(result) = self.results.get(index) else {
            return Task::none();
        };
        let path = result.path.clone();
        let kind = result.kind;

        if let Err(error) = open::that(&path) {
            self.status = format!("Failed to open file: {error}");
            Task::none()
        } else if self.result_count > 0 {
            let history_error = self.history.record_open(&path, kind, &self.settings).err();
            self.status = found_status(self.result_count, self.visible_result_count());
            if let Some(error) = history_error.filter(|_| !hide_after_open) {
                self.status = format_error_status(error);
            }
            if hide_after_open {
                self.hide_window()
            } else {
                Task::none()
            }
        } else if hide_after_open {
            let _ = self.history.record_open(&path, kind, &self.settings);
            self.hide_window()
        } else {
            if let Err(error) = self.history.record_open(&path, kind, &self.settings) {
                self.status = format_error_status(error);
            }

            Task::none()
        }
    }

    fn reveal_selected(&mut self) {
        let Some(path) = self.selected_path() else {
            return;
        };

        if let Err(error) = reveal_in_file_manager(&path) {
            self.status = format!("Failed to reveal file: {error}");
        }
    }

    fn open_selected_in_terminal(&mut self) {
        let Some(path) = self.selected_path() else {
            return;
        };

        let directory = terminal_directory_for(&path);

        if let Err(error) = open_terminal_at(&directory, &self.settings) {
            self.status = format!("Failed to open terminal: {error}");
        } else {
            self.status = String::from("Opened terminal");
        }
    }

    fn selected_path(&self) -> Option<PathBuf> {
        self.selected_result
            .and_then(|index| self.results.get(index))
            .map(|result| result.path.clone())
            .or_else(|| self.selected_result_path.clone())
    }

    fn ghost_completion(&self) -> Option<String> {
        let query = self.ghost_query()?;
        ghost_completion(&query, &self.ghost_target(&query)?)
    }

    fn ghost_query(&self) -> Option<String> {
        if self.pending_search || self.query.trim() != self.last_search_query {
            return None;
        }

        let query = self.parsed_query().search_text;
        (!query.trim().is_empty()).then_some(query)
    }

    fn ghost_target(&self, query: &str) -> Option<String> {
        let result = self.ghost_result()?;

        match result.kind {
            SearchResultKind::Application => result
                .path
                .file_name()
                .and_then(|name| name.to_str())
                .map(ToOwned::to_owned),
            SearchResultKind::FolderName => {
                path_completion_target(&self.relative_result_path(&result.path), query)
            }
            SearchResultKind::FileContent => {
                let relative_path = self.relative_result_path(&result.path);

                path_completion_target(&relative_path, query)
            }
        }
    }

    fn ghost_result(&self) -> Option<&GroupedSearchResult> {
        self.hovered_result_path
            .as_ref()
            .or(self.selected_result_path.as_ref())
            .and_then(|path| self.results.iter().find(|result| &result.path == path))
    }

    fn relative_result_path(&self, path: &Path) -> String {
        self.selected_folder
            .as_ref()
            .and_then(|scope| path.strip_prefix(scope).ok())
            .unwrap_or(path)
            .to_string_lossy()
            .trim_start_matches('/')
            .to_owned()
    }

    fn current_query_matches_active_search(&self) -> bool {
        let parsed = self.parsed_query();

        self.query.trim() == self.last_search_query
            || (!self.active_search_text.is_empty()
                && parsed.search_text.trim() == self.active_search_text)
    }

    fn save_settings(&mut self) -> Task<Message> {
        match self.settings_form.parse() {
            Ok(settings) => {
                self.settings = settings;
                self.history.trim(self.settings.max_history_items);
                let history_error = self.history.save().err();
                let shortcut_error = self.reload_global_shortcut().err();
                self.status = match settings::save(&self.settings) {
                    Ok(path) => format!("Settings saved to {}", path.display()),
                    Err(error) => format_error_status(error),
                };
                if let Some(error) = history_error {
                    self.status = format_error_status(error);
                }
                if let Some(error) = shortcut_error {
                    self.status = format_error_status(error);
                }
                self.sync_window_size()
            }
            Err(error) => {
                self.status = format_error_status(error);
                Task::none()
            }
        }
    }

    fn commit_set_command(&mut self) -> Option<Task<Message>> {
        let command = parse_set_command(&self.query)?;

        match command {
            SetCommand::Ready { field, value } => Some(self.apply_setting_value(field, &value)),
            other => {
                self.pending_search = false;
                self.stop_search();
                self.clear_results();
                self.status = set_command_status(other);
                Some(Task::none())
            }
        }
    }

    fn apply_setting_value(&mut self, field: SettingsField, value: &str) -> Task<Message> {
        let mut form = SettingsForm::from(&self.settings);
        update_settings_form_value(&mut form, field, value.to_owned());

        match form.parse() {
            Ok(settings) => {
                self.settings = settings;
                self.settings_form = SettingsForm::from(&self.settings);
                self.history.trim(self.settings.max_history_items);
                let history_error = self.history.save().err();
                let shortcut_error = self.reload_global_shortcut().err();
                self.query.clear();
                self.pending_search = false;
                self.stop_search();
                self.clear_results();

                self.status = match settings::save(&self.settings) {
                    Ok(_) => format!("Set {} to {}", field.canonical_name(), value.trim()),
                    Err(error) => format_error_status(error),
                };
                if let Some(error) = shortcut_error {
                    self.status = format_error_status(error);
                }
                if let Some(error) = history_error {
                    self.status = format_error_status(error);
                }

                self.sync_window_size()
            }
            Err(error) => {
                self.pending_search = false;
                self.stop_search();
                self.clear_results();
                self.status = format_error_status(error);
                Task::none()
            }
        }
    }

    fn update_settings_form(&mut self, field: SettingsField, value: String) {
        update_settings_form_value(&mut self.settings_form, field, value);
    }

    fn reload_global_shortcut(&mut self) -> Result<(), String> {
        self.global_shortcut = None;
        self.global_shortcut = Some(GlobalShortcut::register(&self.settings.global_shortcut)?);
        Ok(())
    }

    fn sync_window_size(&mut self) -> Task<Message> {
        let height = self.target_window_height();

        if (self.window_height - height).abs() < f32::EPSILON {
            return Task::none();
        }

        self.window_height = height;

        let Some(id) = self.window_id else {
            return Task::none();
        };

        window::resize(id, Size::new(self.settings.window_width, height))
    }

    fn target_window_height(&self) -> f32 {
        if self.visible_result_count() > 0 {
            self.settings.expanded_height
        } else if self.is_fresh_idle_screen() || self.is_set_command_screen() {
            self.settings.idle_height
        } else {
            self.settings.compact_height
        }
    }

    fn is_fresh_idle_screen(&self) -> bool {
        let showing_guidance_status = self.status == idle_status(self.settings.min_query_chars)
            || self.status == min_query_status(self.settings.min_query_chars)
            || self.status.starts_with("Scope set to ")
            || self.status.starts_with("Mode set to ")
            || self.status.starts_with("Press Enter to use ");

        self.results.is_empty() && !self.running && !self.pending_search && showing_guidance_status
    }

    fn is_set_command_screen(&self) -> bool {
        self.results.is_empty()
            && !self.running
            && !self.pending_search
            && parse_set_command(&self.query).is_some()
    }

    fn parsed_query(&self) -> ParsedQuery {
        query::parse_query(&self.query)
    }

    fn apply_query_hints(&mut self, parsed: &ParsedQuery) {
        if parsed.filter.is_some() {
            self.search_mode = SearchMode::Files;
        }

        if let Some(mode) = parsed.mode {
            self.search_mode = mode;
        }

        if !parsed.search_text.trim().is_empty() {
            if let Some(scope) = parsed.scope.clone().filter(|scope| scope.exists()) {
                self.selected_folder = Some(scope);
            }
        }
    }

    fn scope_label_for(&self, folder: &Path) -> String {
        scope_name(folder)
    }

    fn toggle_visibility(&mut self) -> Task<Message> {
        if self.hidden {
            self.hidden = false;
            self.show_window()
        } else {
            self.hidden = true;
            self.hide_window()
        }
    }

    fn show_window(&mut self) -> Task<Message> {
        let Some(id) = self.window_id else {
            return Task::none();
        };

        self.hidden = false;
        self.main_window_focused = false;
        self.macos_app_active = platform_app_is_active();

        window::set_mode(id, window::Mode::Windowed)
            .chain(window::gain_focus(id))
            .chain(operation::focus(search_input_id()))
            .chain(self.sync_window_size())
            .chain(delayed_search_focus())
    }

    fn hide_window(&mut self) -> Task<Message> {
        let Some(id) = self.window_id else {
            return Task::none();
        };

        self.hidden = true;
        self.main_window_focused = false;
        self.macos_app_active = platform_app_is_active();
        self.stop_search();
        self.pending_search = false;
        platform_hide_app();

        window::set_mode(id, window::Mode::Hidden)
    }

    fn select_first_visible(&mut self) -> Task<Message> {
        if let Some(index) = self.visible_indices().first().copied() {
            self.select_result_index(index);
        } else {
            self.selected_result = None;
            self.selected_result_path = None;
        }
        self.last_result_click = None;
        self.scroll_selected_into_view()
    }

    fn select_result_index(&mut self, index: usize) {
        self.selected_result = Some(index);
        self.selected_result_path = self.results.get(index).map(|result| result.path.clone());
    }

    fn reconcile_selected_result(&mut self) {
        if let Some(path) = self.selected_result_path.as_ref() {
            if let Some(index) = self.results.iter().position(|result| &result.path == path) {
                self.selected_result = Some(index);
                return;
            }
        }

        if let Some(index) = self
            .selected_result
            .filter(|index| *index < self.results.len())
        {
            self.selected_result_path = self.results.get(index).map(|result| result.path.clone());
        } else {
            self.selected_result = None;
            self.selected_result_path = None;
        }
    }

    fn selected_visible_position(&self, visible: &[usize]) -> Option<usize> {
        if let Some(path) = self.selected_result_path.as_ref() {
            if let Some(position) = visible.iter().position(|index| {
                self.results
                    .get(*index)
                    .is_some_and(|result| &result.path == path)
            }) {
                return Some(position);
            }
        }

        if let Some(selected) = self.selected_result {
            if let Some(position) = visible.iter().position(|index| *index == selected) {
                return Some(position);
            }

            return visible
                .iter()
                .enumerate()
                .min_by_key(|(_, index)| selected.abs_diff(**index))
                .map(|(position, _)| position);
        }

        None
    }

    fn is_selected_result(&self, index: usize, result: &GroupedSearchResult) -> bool {
        self.selected_result == Some(index)
            || self
                .selected_result_path
                .as_ref()
                .is_some_and(|path| path == &result.path)
    }

    fn scroll_selected_into_view(&mut self) -> Task<Message> {
        let Some(selected) = self.selected_result else {
            return Task::none();
        };

        let visible = self.visible_indices();
        let Some(position) = visible.iter().position(|index| *index == selected) else {
            return Task::none();
        };

        let row_height = self.result_row_height();
        let scroll_margin = row_height * SCROLL_KEEP_VISIBLE_MARGIN_ROWS;
        let row_top = position as f32 * row_height;
        let row_bottom = row_top + row_height;
        let viewport_top = self.results_scroll_y;
        let viewport_bottom = viewport_top + self.results_view_height;

        let target_y = if row_top < viewport_top + scroll_margin {
            Some((row_top - scroll_margin).max(0.0))
        } else if row_bottom > viewport_bottom - scroll_margin {
            Some((row_bottom + scroll_margin - self.results_view_height).max(0.0))
        } else {
            None
        };

        let Some(target_y) = target_y else {
            return Task::none();
        };

        let target_y = self.clamp_scroll_y(target_y, visible.len());
        self.results_scroll_y = target_y;

        operation::scroll_to(
            results_scroll_id(),
            AbsoluteOffset {
                x: None,
                y: Some(target_y),
            },
        )
    }

    fn scroll_after_keyboard_navigation(
        &mut self,
        previous_position: usize,
        next_position: usize,
        direction: NavigationDirection,
    ) -> Task<Message> {
        if previous_position == next_position {
            return Task::none();
        }

        let visible_count = self.visible_indices().len();
        if visible_count == 0 || self.results_view_height <= 0.0 {
            return Task::none();
        }

        let row_height = self.result_row_height();
        let previous_row_top = previous_position as f32 * row_height;
        let previous_screen_top = previous_row_top - self.results_scroll_y;
        let previous_screen_bottom = previous_screen_top + row_height;
        let edge_margin = row_height * 0.75;

        let should_anchor = match direction {
            NavigationDirection::Up => previous_screen_top <= edge_margin,
            NavigationDirection::Down => {
                previous_screen_bottom >= self.results_view_height - edge_margin
            }
        };

        if !should_anchor {
            return self.scroll_selected_into_view();
        }

        let target_y = next_position as f32 * row_height - previous_screen_top;
        let target_y = self.clamp_scroll_y(target_y, visible_count);
        self.results_scroll_y = target_y;

        operation::scroll_to(
            results_scroll_id(),
            AbsoluteOffset {
                x: None,
                y: Some(target_y),
            },
        )
    }

    fn clamp_scroll_y(&self, target_y: f32, visible_count: usize) -> f32 {
        let content_height = visible_count as f32 * self.result_row_height();
        let max_scroll = (content_height - self.results_view_height).max(0.0);

        target_y.clamp(0.0, max_scroll)
    }

    fn result_row_height(&self) -> f32 {
        self.settings.result_row_height.max(96.0)
    }

    fn visible_indices(&self) -> Vec<usize> {
        (0..self.results.len()).collect()
    }

    fn visible_result_count(&self) -> usize {
        self.results.len()
    }

    fn status_is_error(&self) -> bool {
        !(self.status == "Idle"
            || self.status == idle_status(self.settings.min_query_chars)
            || self.status == "Searching…"
            || self.status.starts_with("Searching ")
            || self.status == "Waiting…"
            || self.status == "Search cancelled"
            || self.status == "Path copied"
            || self.status == "Opened terminal"
            || self.status == "Install command copied"
            || self.status == "No results"
            || self.status.starts_with("Showing first ")
            || self.status == min_query_status(self.settings.min_query_chars)
            || self.status.contains(" found in ")
            || self.status.starts_with("Scope set to ")
            || self.status.starts_with("Mode set to ")
            || self.status.starts_with("Settings saved to ")
            || self.status.starts_with("Press Enter to use "))
    }

    fn rga_is_missing(&self) -> bool {
        self.status.contains("ripgrep-all (rga) was not found")
    }
}

enum SearchOutcome {
    Finished,
    Limited,
    Cancelled,
    Error(String),
}

async fn choose_folder() -> Option<PathBuf> {
    FileDialog::new().pick_folder()
}

fn default_scope() -> PathBuf {
    PathBuf::from("/")
}

fn file_folder_scope_for(scope: &Path) -> PathBuf {
    if scope == Path::new("/") {
        return std::env::var_os("HOME")
            .map(PathBuf::from)
            .filter(|home| home.is_dir())
            .unwrap_or_else(|| scope.to_path_buf());
    }

    scope.to_path_buf()
}

fn platform_app_is_active() -> bool {
    #[cfg(target_os = "macos")]
    unsafe {
        use objc::runtime::{Class, Object, Sel, BOOL, NO};
        use objc::Message;

        let Some(class) = Class::get("NSRunningApplication") else {
            return false;
        };
        let Ok(app) =
            class.send_message::<(), *mut Object>(Sel::register("currentApplication"), ())
        else {
            return false;
        };
        if app.is_null() {
            return false;
        }
        let Ok(active) = (&*app).send_message::<(), BOOL>(Sel::register("isActive"), ()) else {
            return false;
        };
        active != NO
    }

    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}

fn platform_hide_app() {
    #[cfg(target_os = "macos")]
    unsafe {
        use objc::runtime::{Class, Object, Sel};
        use objc::Message;
        use std::ptr;

        let Some(class) = Class::get("NSApplication") else {
            return;
        };
        let Ok(app) = class.send_message::<(), *mut Object>(Sel::register("sharedApplication"), ())
        else {
            return;
        };
        if app.is_null() {
            return;
        }
        let _ =
            (&*app).send_message::<(*mut Object,), ()>(Sel::register("hide:"), (ptr::null_mut(),));
    }
}

fn search_input_id() -> Id {
    Id::new("deeplens-search-input")
}

fn results_scroll_id() -> Id {
    Id::new("deeplens-results-scroll")
}

fn settings_field<'a>(
    label: &'static str,
    value: &'a str,
    help: &'static str,
    field: SettingsField,
) -> Element<'a, Message> {
    row![
        column![
            text(label).size(12).color(MUTED),
            text(help).size(11).color(FAINT),
        ]
        .spacing(2)
        .width(Length::Fill),
        text_input("", value)
            .on_input(move |value| Message::SettingsChanged(field, value))
            .padding([6, 8])
            .size(13)
            .style(search_input_style)
            .width(Length::Fixed(150.0)),
    ]
    .spacing(10)
    .align_y(Alignment::Center)
    .into()
}

fn settings_window_padding() -> Padding {
    #[cfg(target_os = "macos")]
    {
        Padding {
            top: 32.0,
            right: 12.0,
            bottom: 12.0,
            left: 12.0,
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        Padding::new(12.0)
    }
}

fn delayed_search_focus() -> Task<Message> {
    Task::perform(
        async {
            std::thread::sleep(Duration::from_millis(90));
        },
        |_| Message::FocusSearchInput,
    )
}

fn view_no_results_state(scope: &str) -> Element<'static, Message> {
    let hints = [
        "Try fewer words",
        "Choose a folder",
        "Use /project to switch scope with zoxide",
        "Try an exact phrase with quotes",
    ]
    .into_iter()
    .fold(column![].spacing(4), |column, hint| {
        column.push(text(format!("• {hint}")).size(12).color(MUTED))
    });

    container(
        column![
            text("No results found").size(15).color(TEXT),
            text(format!("No matches in {scope}."))
                .size(13)
                .color(MUTED),
            hints,
        ]
        .spacing(9)
        .align_x(Alignment::Center),
    )
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into()
}

fn view_result_row(
    index: usize,
    result: &GroupedSearchResult,
    selected: bool,
    query: &str,
    row_height: f32,
) -> Element<'static, Message> {
    let filename = result
        .path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("(unknown file)")
        .to_owned();

    let path = result_path_label(result);
    let snippet = result.snippet.trim();

    let title_font = Font {
        weight: font::Weight::Semibold,
        ..Font::DEFAULT
    };

    let more_matches: Element<'static, Message> = if result.match_count > 1 {
        text(format!("+{} more matches", result.match_count - 1))
            .size(12)
            .color(FAINT)
            .into()
    } else {
        Space::new().height(Length::Fixed(0.0)).into()
    };

    let body = container(
        row![
            file_icon(result.kind, &result.path, result.icon_path.as_deref()),
            column![
                text(filename).size(15).font(title_font).color(TEXT),
                highlighted_snippet(snippet, query),
                text(path).size(12).color(FAINT),
                more_matches,
            ]
            .spacing(3)
            .width(Length::Fill),
        ]
        .spacing(8)
        .padding([10, 12])
        .align_y(Alignment::Center)
        .width(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fixed(row_height))
    .style(move |_| result_row_style(selected));

    mouse_area(body)
        .on_enter(Message::HoverResult(index))
        .on_exit(Message::ClearHoveredResult(index))
        .on_press(Message::SelectResult(index))
        .into()
}

fn file_icon(
    result_kind: SearchResultKind,
    path: &Path,
    icon_path: Option<&Path>,
) -> Element<'static, Message> {
    if result_kind == SearchResultKind::Application {
        if let Some(icon_path) = icon_path {
            return container(
                image(image::Handle::from_path(icon_path.to_path_buf()))
                    .width(Length::Fixed(22.0))
                    .height(Length::Fixed(22.0)),
            )
            .width(Length::Fixed(24.0))
            .center_x(Length::Fixed(24.0))
            .into();
        }
    }

    let kind = ResultKind::from_result(result_kind, path);

    container(text(kind.symbol()).size(16))
        .width(Length::Fixed(18.0))
        .into()
}

fn result_path_label(result: &GroupedSearchResult) -> String {
    let path = compact_path(&result.path);

    match result.line_number {
        Some(line) => format!("{path}:{line}"),
        None => path,
    }
}

#[derive(Debug, Clone, Copy)]
enum ResultKind {
    Folder,
    App,
    Pdf,
    Doc,
    Text,
    Code,
    File,
}

impl ResultKind {
    fn from_result(result_kind: SearchResultKind, path: &Path) -> Self {
        if result_kind == SearchResultKind::FolderName {
            return Self::Folder;
        }

        if result_kind == SearchResultKind::Application {
            return Self::App;
        }

        match path
            .extension()
            .and_then(|extension| extension.to_str())
            .map(str::to_ascii_lowercase)
            .as_deref()
        {
            Some("pdf") => Self::Pdf,
            Some("doc" | "docx" | "rtf" | "odt" | "pages") => Self::Doc,
            Some("rs" | "toml" | "json" | "yaml" | "yml" | "js" | "ts" | "tsx" | "jsx") => {
                Self::Code
            }
            Some("txt" | "md" | "markdown") => Self::Text,
            _ => Self::File,
        }
    }

    fn symbol(self) -> &'static str {
        match self {
            Self::Folder => "📁",
            Self::App => "□",
            Self::Pdf | Self::Doc | Self::Text | Self::Code | Self::File => "📄",
        }
    }
}

fn highlighted_snippet(snippet: &str, query: &str) -> Element<'static, Message> {
    let snippet = if snippet.is_empty() {
        "(no preview available)"
    } else {
        snippet
    };

    let spans = highlighted_spans(snippet, query);

    iced::widget::text::Rich::<(), Message>::with_spans(spans)
        .size(13)
        .width(Length::Fill)
        .into()
}

fn highlighted_spans(
    snippet: &str,
    query: &str,
) -> Vec<iced::widget::text::Span<'static, (), Font>> {
    let terms: Vec<String> = query
        .split_whitespace()
        .map(str::to_ascii_lowercase)
        .filter(|term| !term.is_empty())
        .collect();

    if terms.is_empty() {
        return vec![iced::widget::text::Span::new(snippet.to_owned()).color(MUTED)];
    }

    let snippet_lower = snippet.to_ascii_lowercase();
    let mut spans = Vec::new();
    let mut cursor = 0;

    while let Some((start, end)) = next_highlight(&snippet_lower, &terms, cursor) {
        if start > cursor {
            spans.push(
                iced::widget::text::Span::new(snippet[cursor..start].to_owned()).color(MUTED),
            );
        }

        spans.push(
            iced::widget::text::Span::new(snippet[start..end].to_owned())
                .color(HIGHLIGHT)
                .font(Font {
                    weight: font::Weight::Semibold,
                    ..Font::DEFAULT
                }),
        );

        cursor = end;
    }

    if cursor < snippet.len() {
        spans.push(iced::widget::text::Span::new(snippet[cursor..].to_owned()).color(MUTED));
    }

    if spans.is_empty() {
        spans.push(iced::widget::text::Span::new(snippet.to_owned()).color(MUTED));
    }

    spans
}

fn next_highlight(snippet: &str, terms: &[String], cursor: usize) -> Option<(usize, usize)> {
    terms
        .iter()
        .filter_map(|term| {
            snippet[cursor..]
                .find(term)
                .map(|relative| (cursor + relative, cursor + relative + term.len()))
        })
        .min_by_key(|(start, _)| *start)
}

fn compact_path(path: &Path) -> String {
    let display = if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
        if let Ok(relative) = path.strip_prefix(&home) {
            format!("~/{}", relative.display())
        } else {
            path.display().to_string()
        }
    } else {
        path.display().to_string()
    };

    display
}

fn scope_name(path: &Path) -> String {
    compact_path(path)
}

fn scope_pill(label: String) -> Element<'static, Message> {
    container(text(label).size(12).color(MUTED))
        .padding([5, 9])
        .style(pill_style)
        .into()
}

fn query_is_ready(query: &str, min_query_chars: usize) -> bool {
    query.trim().chars().count() >= min_query_chars
}

fn path_completion_target(path: &str, query: &str) -> Option<String> {
    let query = query.trim();

    if query.is_empty() {
        return None;
    }

    let query = query.to_ascii_lowercase();
    let segments: Vec<&str> = path
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect();

    segments
        .iter()
        .position(|segment| segment.to_ascii_lowercase().starts_with(&query))
        .map(|position| segments[position..].join("/"))
}

fn min_query_status(min_query_chars: usize) -> String {
    format!("Type {min_query_chars}+ characters")
}

fn idle_status(min_query_chars: usize) -> String {
    format!("Type {min_query_chars}+ characters, or use /project to switch scope.")
}

fn format_error_status(error: String) -> String {
    if error.contains("ripgrep-all (rga)") || error.starts_with("Error:") {
        error
    } else {
        format!("Error: {error}")
    }
}

fn view_status_label(
    status: &str,
    occurrences: usize,
    file_count: usize,
    running: bool,
    min_query_chars: usize,
) -> Element<'static, Message> {
    let label = if running {
        status.to_owned()
    } else if status == "Waiting…" {
        String::from("Waiting…")
    } else if status == "Search cancelled" {
        String::from("Cancelled")
    } else if status == "Path copied" {
        String::from("Path copied")
    } else if status == "Opened terminal" {
        String::from("Opened terminal")
    } else if status == "Install command copied" {
        String::from("Copied")
    } else if status == min_query_status(min_query_chars) {
        min_query_status(min_query_chars)
    } else if status == idle_status(min_query_chars) {
        idle_status(min_query_chars)
    } else if status == "No results" {
        String::from("No results")
    } else if status.starts_with("Showing first ") {
        status.replace(" files", "")
    } else if status.contains(" found in ") {
        found_status(occurrences, file_count)
    } else if status.starts_with("Scope set to ")
        || status.starts_with("Mode set to ")
        || status.starts_with("Press Enter to use ")
    {
        status.to_owned()
    } else if is_set_command_status(status) {
        status.to_owned()
    } else if status.starts_with("Settings saved to ") {
        status.to_owned()
    } else {
        idle_status(min_query_chars)
    };

    let content: Element<'static, Message> = if running {
        row![
            spinner_dots(),
            text(label)
                .size(12)
                .color(status_color(status, min_query_chars)),
        ]
        .spacing(5)
        .align_y(Alignment::Center)
        .into()
    } else {
        text(label)
            .size(12)
            .color(status_color(status, min_query_chars))
            .into()
    };

    container(content).into()
}

fn spinner_dots() -> Element<'static, Message> {
    text("•••").size(12).color(ACCENT).into()
}

fn found_status(occurrences: usize, file_count: usize) -> String {
    format!(
        "{occurrences} found in {file_count} file{}",
        plural(file_count)
    )
}

fn plural(count: usize) -> &'static str {
    if count == 1 {
        ""
    } else {
        "s"
    }
}

fn status_color(status: &str, min_query_chars: usize) -> Color {
    if status == "Idle"
        || status == idle_status(min_query_chars)
        || status == "Searching…"
        || status == "Waiting…"
        || status == "Path copied"
        || status == "Opened terminal"
        || status == "Install command copied"
        || status == "No results"
        || status.starts_with("Showing first ")
        || status.contains(" found in ")
        || status.starts_with("Searching ")
        || status.starts_with("Scope set to ")
        || status.starts_with("Settings saved to ")
        || status.starts_with("Press Enter to use ")
        || is_set_command_status(status)
    {
        MUTED
    } else if status == "Search cancelled" {
        FAINT
    } else {
        ERROR
    }
}

fn is_set_command_status(status: &str) -> bool {
    status.starts_with("Use /set ")
        || status.starts_with("Press Enter to set ")
        || status.starts_with("Unknown setting ")
}

fn surface_style(_: &Theme) -> container_style::Style {
    container_style::Style {
        background: Some(Background::Color(SURFACE)),
        border: Border {
            color: Color::from_rgba(1.0, 1.0, 1.0, 0.72),
            width: 1.0,
            radius: border::Radius::from(24.0),
        },
        ..container_style::Style::default()
    }
}

fn settings_surface_style(_: &Theme) -> container_style::Style {
    container_style::Style {
        background: Some(Background::Color(SURFACE)),
        border: Border {
            color: SURFACE,
            width: 1.0,
            radius: border::Radius::from(0.0),
        },
        ..container_style::Style::default()
    }
}

fn results_scrollable_style(
    _: &Theme,
    status: scrollable_style::Status,
) -> scrollable_style::Style {
    let rail = scrollable_style::Rail {
        background: Some(Background::Color(Color::from_rgb(0.075, 0.078, 0.088))),
        border: border::rounded(7.0).width(0.0),
        scroller: scrollable_style::Scroller {
            background: Background::Color(match status {
                scrollable_style::Status::Dragged { .. } => Color::from_rgb(0.36, 0.38, 0.42),
                scrollable_style::Status::Hovered { .. } => Color::from_rgb(0.29, 0.31, 0.35),
                scrollable_style::Status::Active { .. } => Color::from_rgb(0.22, 0.235, 0.27),
            }),
            border: border::rounded(7.0).width(0.0),
        },
    };

    scrollable_style::Style {
        container: container_style::Style::default(),
        vertical_rail: rail,
        horizontal_rail: rail,
        gap: Some(Background::Color(SURFACE)),
        auto_scroll: scrollable_style::default(&Theme::Dark, status).auto_scroll,
    }
}

fn prompt_style(_: &Theme) -> container_style::Style {
    container_style::Style {
        background: Some(Background::Color(PROMPT)),
        border: border::rounded(20.0),
        ..container_style::Style::default()
    }
}

fn pill_style(_: &Theme) -> container_style::Style {
    container_style::Style {
        background: Some(Background::Color(Color::from_rgb(0.105, 0.110, 0.122))),
        border: border::rounded(14.0)
            .color(Color::from_rgb(0.18, 0.19, 0.21))
            .width(1.0),
        ..container_style::Style::default()
    }
}

fn search_input_style(_: &Theme, status: input_style::Status) -> input_style::Style {
    let border_width = match status {
        input_style::Status::Focused { .. } => 0.0,
        _ => 0.0,
    };

    input_style::Style {
        background: Background::Color(PROMPT),
        border: border::rounded(0.0).color(PROMPT).width(border_width),
        icon: MUTED,
        placeholder: FAINT,
        value: TEXT,
        selection: ACCENT,
    }
}

fn icon_button(_: &Theme, status: button_style::Status) -> button_style::Style {
    let background = match status {
        button_style::Status::Hovered => Color::from_rgb(0.18, 0.185, 0.20),
        button_style::Status::Pressed => Color::from_rgb(0.22, 0.225, 0.24),
        _ => Color::TRANSPARENT,
    };

    button_style::Style {
        background: Some(Background::Color(background)),
        text_color: TEXT,
        border: border::rounded(16.0),
        ..button_style::Style::default()
    }
}

fn chip_button(_: &Theme, status: button_style::Status) -> button_style::Style {
    let background = match status {
        button_style::Status::Hovered => Color::from_rgb(0.20, 0.205, 0.22),
        button_style::Status::Pressed => Color::from_rgb(0.24, 0.245, 0.26),
        _ => CHIP,
    };

    button_style::Style {
        background: Some(Background::Color(background)),
        text_color: TEXT,
        border: border::rounded(18.0),
        ..button_style::Style::default()
    }
}

fn filter_button_style(status: button_style::Status, selected: bool) -> button_style::Style {
    let background = if selected {
        Color::from_rgb(0.18, 0.205, 0.250)
    } else {
        match status {
            button_style::Status::Hovered => Color::from_rgb(0.18, 0.185, 0.20),
            button_style::Status::Pressed => Color::from_rgb(0.22, 0.225, 0.24),
            _ => Color::TRANSPARENT,
        }
    };

    button_style::Style {
        background: Some(Background::Color(background)),
        text_color: if selected { TEXT } else { MUTED },
        border: border::rounded(16.0)
            .color(if selected {
                Color::from_rgb(0.24, 0.29, 0.38)
            } else {
                BORDER
            })
            .width(if selected { 1.0 } else { 0.0 }),
        ..button_style::Style::default()
    }
}

fn result_row_style(selected: bool) -> container_style::Style {
    container_style::Style {
        background: Some(Background::Color(if selected {
            SELECTED
        } else {
            Color::TRANSPARENT
        })),
        border: border::rounded(16.0),
        ..container_style::Style::default()
    }
}

#[cfg(test)]
mod tests {
    use super::path_completion_target;

    #[test]
    fn detects_path_prefix_segments_for_ghost_completion() {
        assert_eq!(
            path_completion_target("tests/book.py", "test"),
            Some(String::from("tests/book.py"))
        );
        assert_eq!(
            path_completion_target("src/tests/book.py", "test"),
            Some(String::from("tests/book.py"))
        );
        assert_eq!(
            path_completion_target("backend/tests", "test"),
            Some(String::from("tests"))
        );
        assert_eq!(path_completion_target("src/book.py", "test"), None);
    }
}
