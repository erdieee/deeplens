use global_hotkey::hotkey::{Code as GlobalCode, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use iced::{event, keyboard, window, Event};

pub struct GlobalShortcut {
    _manager: GlobalHotKeyManager,
    hotkey_id: u32,
}

impl GlobalShortcut {
    pub fn register(shortcut: &str) -> Result<Self, String> {
        let manager = GlobalHotKeyManager::new().map_err(|error| error.to_string())?;
        let hotkey = parse_global_hotkey(shortcut)?;
        let hotkey_id = hotkey.id();

        manager
            .register(hotkey)
            .map_err(|error| format!("Global shortcut unavailable: {error}"))?;

        Ok(Self {
            _manager: manager,
            hotkey_id,
        })
    }

    pub fn was_pressed(&self) -> bool {
        let mut pressed = false;

        while let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            if event.id == self.hotkey_id && event.state == HotKeyState::Pressed {
                pressed = true;
            }
        }

        pressed
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShortcutKey {
    Character(char),
    Enter,
    ArrowUp,
    ArrowDown,
    ArrowRight,
    Tab,
    Escape,
    Space,
    Comma,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AppShortcut {
    control: bool,
    alt: bool,
    shift: bool,
    logo: bool,
    key: ShortcutKey,
}

impl AppShortcut {
    fn matches(
        self,
        key: &keyboard::Key,
        physical_key: keyboard::key::Physical,
        modifiers: keyboard::Modifiers,
    ) -> bool {
        self.control == modifiers.control()
            && self.alt == modifiers.alt()
            && self.shift == modifiers.shift()
            && self.logo == modifiers.logo()
            && (shortcut_key_matches(self.key, key)
                || shortcut_physical_key_matches(self.key, physical_key))
    }
}

pub fn shortcut_matches(
    spec: &str,
    key: &keyboard::Key,
    physical_key: keyboard::key::Physical,
    modifiers: keyboard::Modifiers,
) -> bool {
    if let Ok(shortcut) = parse_app_shortcut(spec) {
        return shortcut.matches(key, physical_key, modifiers);
    }

    spec.split(',')
        .filter_map(|part| parse_app_shortcut(part).ok())
        .any(|shortcut| shortcut.matches(key, physical_key, modifiers))
}

pub fn validate_shortcut(value: &str, label: &str) -> Result<String, String> {
    let value = value.trim();
    parse_app_shortcut(value).map_err(|error| format!("{label}: {error}"))?;
    Ok(value.to_ascii_lowercase())
}

pub fn validate_shortcut_list(value: &str, label: &str) -> Result<String, String> {
    let value = value.trim();
    let parts: Vec<&str> = value
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect();

    if parts.is_empty() {
        return Err(format!("{label} cannot be empty."));
    }

    for part in &parts {
        parse_app_shortcut(part).map_err(|error| format!("{label}: {error}"))?;
    }

    Ok(parts.join(",").to_ascii_lowercase())
}

pub fn validate_global_shortcut(value: &str, label: &str) -> Result<String, String> {
    let value = value.trim();
    parse_global_hotkey(value).map_err(|error| format!("{label}: {error}"))?;
    Ok(value.to_ascii_lowercase())
}

fn parse_app_shortcut(value: &str) -> Result<AppShortcut, String> {
    let mut shortcut = AppShortcut {
        control: false,
        alt: false,
        shift: false,
        logo: false,
        key: ShortcutKey::Space,
    };
    let mut key = None;

    for token in shortcut_tokens(value) {
        match token.as_str() {
            "cmd" | "command" => set_command_modifier(&mut shortcut),
            "ctrl" | "control" => shortcut.control = true,
            "alt" | "option" => shortcut.alt = true,
            "shift" => shortcut.shift = true,
            "super" | "meta" | "logo" => shortcut.logo = true,
            _ => {
                if key.is_some() {
                    return Err(String::from("only one non-modifier key is supported"));
                }
                key = Some(parse_shortcut_key(&token)?);
            }
        }
    }

    shortcut.key = key.ok_or_else(|| String::from("missing key"))?;
    Ok(shortcut)
}

fn parse_global_hotkey(value: &str) -> Result<HotKey, String> {
    let mut modifiers = Modifiers::empty();
    let mut key = None;

    for token in shortcut_tokens(value) {
        match token.as_str() {
            "cmd" | "command" | "super" | "meta" | "logo" => modifiers |= Modifiers::SUPER,
            "ctrl" | "control" => modifiers |= Modifiers::CONTROL,
            "alt" | "option" => modifiers |= Modifiers::ALT,
            "shift" => modifiers |= Modifiers::SHIFT,
            _ => {
                if key.is_some() {
                    return Err(String::from("only one non-modifier key is supported"));
                }
                key = Some(parse_global_key(&token)?);
            }
        }
    }

    let key = key.ok_or_else(|| String::from("missing key"))?;
    Ok(HotKey::new(Some(modifiers), key))
}

fn shortcut_tokens(value: &str) -> impl Iterator<Item = String> + '_ {
    value
        .split('+')
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(|token| token.to_ascii_lowercase())
}

fn set_command_modifier(shortcut: &mut AppShortcut) {
    #[cfg(target_os = "macos")]
    {
        shortcut.logo = true;
    }

    #[cfg(not(target_os = "macos"))]
    {
        shortcut.control = true;
    }
}

fn parse_shortcut_key(token: &str) -> Result<ShortcutKey, String> {
    match token {
        "enter" | "return" => Ok(ShortcutKey::Enter),
        "up" | "arrowup" => Ok(ShortcutKey::ArrowUp),
        "down" | "arrowdown" => Ok(ShortcutKey::ArrowDown),
        "right" | "arrowright" => Ok(ShortcutKey::ArrowRight),
        "tab" => Ok(ShortcutKey::Tab),
        "esc" | "escape" => Ok(ShortcutKey::Escape),
        "space" => Ok(ShortcutKey::Space),
        "," | "comma" => Ok(ShortcutKey::Comma),
        value if value.chars().count() == 1 => {
            Ok(ShortcutKey::Character(value.chars().next().unwrap()))
        }
        _ => Err(format!("unsupported key {token}")),
    }
}

fn shortcut_key_matches(shortcut_key: ShortcutKey, key: &keyboard::Key) -> bool {
    match (shortcut_key, key.as_ref()) {
        (ShortcutKey::Enter, keyboard::Key::Named(keyboard::key::Named::Enter)) => true,
        (ShortcutKey::ArrowUp, keyboard::Key::Named(keyboard::key::Named::ArrowUp)) => true,
        (ShortcutKey::ArrowDown, keyboard::Key::Named(keyboard::key::Named::ArrowDown)) => true,
        (ShortcutKey::ArrowRight, keyboard::Key::Named(keyboard::key::Named::ArrowRight)) => true,
        (ShortcutKey::Tab, keyboard::Key::Named(keyboard::key::Named::Tab)) => true,
        (ShortcutKey::Escape, keyboard::Key::Named(keyboard::key::Named::Escape)) => true,
        (ShortcutKey::Space, keyboard::Key::Named(keyboard::key::Named::Space)) => true,
        (ShortcutKey::Comma, keyboard::Key::Character(",")) => true,
        (ShortcutKey::Character(expected), keyboard::Key::Character(actual)) => actual
            .chars()
            .next()
            .is_some_and(|actual| actual.eq_ignore_ascii_case(&expected)),
        _ => false,
    }
}

fn shortcut_physical_key_matches(
    shortcut_key: ShortcutKey,
    physical_key: keyboard::key::Physical,
) -> bool {
    use keyboard::key::Code;

    match shortcut_key {
        ShortcutKey::Enter => physical_key == Code::Enter,
        ShortcutKey::ArrowUp => physical_key == Code::ArrowUp,
        ShortcutKey::ArrowDown => physical_key == Code::ArrowDown,
        ShortcutKey::ArrowRight => physical_key == Code::ArrowRight,
        ShortcutKey::Tab => physical_key == Code::Tab,
        ShortcutKey::Escape => physical_key == Code::Escape,
        ShortcutKey::Space => physical_key == Code::Space,
        ShortcutKey::Comma => physical_key == Code::Comma,
        ShortcutKey::Character(character) => {
            physical_key
                == match character.to_ascii_lowercase() {
                    'a' => Code::KeyA,
                    'b' => Code::KeyB,
                    'c' => Code::KeyC,
                    'd' => Code::KeyD,
                    'e' => Code::KeyE,
                    'f' => Code::KeyF,
                    'g' => Code::KeyG,
                    'h' => Code::KeyH,
                    'i' => Code::KeyI,
                    'j' => Code::KeyJ,
                    'k' => Code::KeyK,
                    'l' => Code::KeyL,
                    'm' => Code::KeyM,
                    'n' => Code::KeyN,
                    'o' => Code::KeyO,
                    'p' => Code::KeyP,
                    'q' => Code::KeyQ,
                    'r' => Code::KeyR,
                    's' => Code::KeyS,
                    't' => Code::KeyT,
                    'u' => Code::KeyU,
                    'v' => Code::KeyV,
                    'w' => Code::KeyW,
                    'x' => Code::KeyX,
                    'y' => Code::KeyY,
                    'z' => Code::KeyZ,
                    '0' => Code::Digit0,
                    '1' => Code::Digit1,
                    '2' => Code::Digit2,
                    '3' => Code::Digit3,
                    '4' => Code::Digit4,
                    '5' => Code::Digit5,
                    '6' => Code::Digit6,
                    '7' => Code::Digit7,
                    '8' => Code::Digit8,
                    '9' => Code::Digit9,
                    ',' => Code::Comma,
                    _ => return false,
                }
        }
    }
}

fn parse_global_key(token: &str) -> Result<GlobalCode, String> {
    match token {
        "a" => Ok(GlobalCode::KeyA),
        "b" => Ok(GlobalCode::KeyB),
        "c" => Ok(GlobalCode::KeyC),
        "d" => Ok(GlobalCode::KeyD),
        "e" => Ok(GlobalCode::KeyE),
        "f" => Ok(GlobalCode::KeyF),
        "g" => Ok(GlobalCode::KeyG),
        "h" => Ok(GlobalCode::KeyH),
        "i" => Ok(GlobalCode::KeyI),
        "j" => Ok(GlobalCode::KeyJ),
        "k" => Ok(GlobalCode::KeyK),
        "l" => Ok(GlobalCode::KeyL),
        "m" => Ok(GlobalCode::KeyM),
        "n" => Ok(GlobalCode::KeyN),
        "o" => Ok(GlobalCode::KeyO),
        "p" => Ok(GlobalCode::KeyP),
        "q" => Ok(GlobalCode::KeyQ),
        "r" => Ok(GlobalCode::KeyR),
        "s" => Ok(GlobalCode::KeyS),
        "t" => Ok(GlobalCode::KeyT),
        "u" => Ok(GlobalCode::KeyU),
        "v" => Ok(GlobalCode::KeyV),
        "w" => Ok(GlobalCode::KeyW),
        "x" => Ok(GlobalCode::KeyX),
        "y" => Ok(GlobalCode::KeyY),
        "z" => Ok(GlobalCode::KeyZ),
        "0" => Ok(GlobalCode::Digit0),
        "1" => Ok(GlobalCode::Digit1),
        "2" => Ok(GlobalCode::Digit2),
        "3" => Ok(GlobalCode::Digit3),
        "4" => Ok(GlobalCode::Digit4),
        "5" => Ok(GlobalCode::Digit5),
        "6" => Ok(GlobalCode::Digit6),
        "7" => Ok(GlobalCode::Digit7),
        "8" => Ok(GlobalCode::Digit8),
        "9" => Ok(GlobalCode::Digit9),
        "space" => Ok(GlobalCode::Space),
        "enter" | "return" => Ok(GlobalCode::Enter),
        "tab" => Ok(GlobalCode::Tab),
        "esc" | "escape" => Ok(GlobalCode::Escape),
        "," | "comma" => Ok(GlobalCode::Comma),
        _ => Err(format!("unsupported global shortcut key {token}")),
    }
}

pub fn keyboard_event(
    event: Event,
    _: event::Status,
    _: window::Id,
    settings: &crate::settings::AppSettings,
) -> Option<crate::app::Message> {
    let Event::Keyboard(keyboard::Event::KeyPressed {
        key,
        physical_key,
        modifiers,
        repeat,
        ..
    }) = event
    else {
        return None;
    };

    let repeats_allowed = shortcut_matches(
        &settings.previous_result_shortcut,
        &key,
        physical_key,
        modifiers,
    ) || shortcut_matches(
        &settings.next_result_shortcut,
        &key,
        physical_key,
        modifiers,
    );

    if repeat && !repeats_allowed {
        return None;
    }

    if shortcut_matches(&settings.reveal_shortcut, &key, physical_key, modifiers) {
        Some(crate::app::Message::RevealSelected)
    } else if shortcut_matches(&settings.preview_shortcut, &key, physical_key, modifiers) {
        Some(crate::app::Message::PreviewSelected)
    } else if shortcut_matches(&settings.submit_shortcut, &key, physical_key, modifiers) {
        Some(crate::app::Message::Submit)
    } else if shortcut_matches(
        &settings.previous_result_shortcut,
        &key,
        physical_key,
        modifiers,
    ) {
        Some(crate::app::Message::SelectPrevious)
    } else if shortcut_matches(
        &settings.next_result_shortcut,
        &key,
        physical_key,
        modifiers,
    ) {
        Some(crate::app::Message::SelectNext)
    } else if shortcut_matches(
        &settings.accept_completion_shortcuts,
        &key,
        physical_key,
        modifiers,
    ) {
        Some(crate::app::Message::AcceptGhostCompletion)
    } else if shortcut_matches(&settings.cancel_shortcut, &key, physical_key, modifiers) {
        Some(crate::app::Message::Escape)
    } else if shortcut_matches(&settings.copy_path_shortcut, &key, physical_key, modifiers) {
        Some(crate::app::Message::CopySelectedPath)
    } else if shortcut_matches(&settings.terminal_shortcut, &key, physical_key, modifiers) {
        Some(crate::app::Message::OpenSelectedInTerminal)
    } else if shortcut_matches(&settings.settings_shortcut, &key, physical_key, modifiers) {
        Some(crate::app::Message::ShowSettings)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{
        parse_app_shortcut, parse_global_hotkey, shortcut_matches, validate_shortcut_list,
    };
    use iced::keyboard;

    #[test]
    fn parses_app_shortcuts() {
        assert!(parse_app_shortcut("cmd+t").is_ok());
        assert!(parse_app_shortcut("cmd+enter").is_ok());
        assert!(parse_app_shortcut("tab").is_ok());
        assert!(parse_app_shortcut("right").is_ok());
    }

    #[test]
    fn parses_shortcut_lists() {
        assert_eq!(
            validate_shortcut_list("tab, right", "Completion").unwrap(),
            "tab,right"
        );
    }

    #[test]
    fn parses_global_shortcuts() {
        assert!(parse_global_hotkey("cmd+shift+space").is_ok());
    }

    #[test]
    fn matches_settings_shortcut_by_physical_comma_key() {
        let modifiers = {
            #[cfg(target_os = "macos")]
            {
                keyboard::Modifiers::LOGO
            }

            #[cfg(not(target_os = "macos"))]
            {
                keyboard::Modifiers::CTRL
            }
        };

        assert!(shortcut_matches(
            "cmd+,",
            &keyboard::Key::Character("<".into()),
            keyboard::key::Physical::Code(keyboard::key::Code::Comma),
            modifiers,
        ));
    }
}
