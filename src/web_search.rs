use crate::settings::AppSettings;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebSearchCommand {
    pub shortcut: String,
    pub typed_shortcut: String,
    pub query: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WebSearchParseError {
    EmptyShortcut,
    InvalidEntry(String),
    TemplateMissingQuery(String),
}

pub fn parse_web_search_command(
    input: &str,
    settings: &AppSettings,
) -> Result<Option<WebSearchCommand>, WebSearchParseError> {
    let trimmed = input.trim();
    let (command, slash_prefixed) = trimmed
        .strip_prefix('/')
        .map(|command| (command, true))
        .unwrap_or((trimmed, false));

    if command.is_empty() {
        return Ok(None);
    }

    let mut parts = command.splitn(2, char::is_whitespace);
    let shortcut = parts.next().unwrap_or_default().trim();
    let query = parts.next().unwrap_or_default().trim();

    if shortcut.is_empty() {
        return Err(WebSearchParseError::EmptyShortcut);
    }

    let Some(template) = web_shortcut_template(&settings.web_search_shortcuts, shortcut)? else {
        return Ok(None);
    };

    if query.is_empty() {
        return Ok(Some(WebSearchCommand {
            shortcut: shortcut.to_owned(),
            typed_shortcut: typed_shortcut(shortcut, slash_prefixed),
            query: String::new(),
            url: String::new(),
        }));
    }

    Ok(Some(WebSearchCommand {
        shortcut: shortcut.to_owned(),
        typed_shortcut: typed_shortcut(shortcut, slash_prefixed),
        query: query.to_owned(),
        url: template.replace("{query}", &percent_encode_query(query)),
    }))
}

pub fn validate_web_shortcuts(value: &str) -> Result<String, String> {
    for entry in shortcut_entries(value) {
        let (shortcut, template) = split_entry(entry).map_err(format_web_search_error)?;

        if shortcut.starts_with('/') {
            return Err(format!(
                "Web shortcut {shortcut} should not include leading /."
            ));
        }

        if shortcut.chars().any(char::is_whitespace) {
            return Err(format!("Web shortcut {shortcut} cannot contain spaces."));
        }

        if !template.starts_with("https://") && !template.starts_with("http://") {
            return Err(format!(
                "Web shortcut {shortcut} must use http:// or https://."
            ));
        }

        if !template.contains("{query}") {
            return Err(format!(
                "Web shortcut {shortcut} URL must contain {{query}}."
            ));
        }
    }

    Ok(value.trim().to_owned())
}

pub fn web_search_status(
    command: Result<Option<WebSearchCommand>, WebSearchParseError>,
) -> Option<String> {
    match command {
        Ok(Some(command)) if command.query.is_empty() => {
            Some(format!("Type a query after {}", command.typed_shortcut))
        }
        Ok(Some(command)) => Some(format!(
            "Press Enter to search {} for {}",
            command.typed_shortcut, command.query
        )),
        Ok(None) => None,
        Err(error) => Some(format_web_search_error(error)),
    }
}

fn typed_shortcut(shortcut: &str, slash_prefixed: bool) -> String {
    if slash_prefixed {
        format!("/{shortcut}")
    } else {
        shortcut.to_owned()
    }
}

fn web_shortcut_template(
    settings_value: &str,
    requested: &str,
) -> Result<Option<String>, WebSearchParseError> {
    for entry in shortcut_entries(settings_value) {
        let (shortcut, template) = split_entry(entry)?;

        if shortcut.eq_ignore_ascii_case(requested) {
            return Ok(Some(template.to_owned()));
        }
    }

    Ok(None)
}

fn shortcut_entries(value: &str) -> impl Iterator<Item = &str> {
    value
        .split([',', '\n'])
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
}

fn split_entry(entry: &str) -> Result<(&str, &str), WebSearchParseError> {
    let Some((shortcut, template)) = entry.split_once('=') else {
        return Err(WebSearchParseError::InvalidEntry(entry.to_owned()));
    };

    let shortcut = shortcut.trim();
    let template = template.trim();

    if shortcut.is_empty() {
        return Err(WebSearchParseError::EmptyShortcut);
    }

    if !template.contains("{query}") {
        return Err(WebSearchParseError::TemplateMissingQuery(
            shortcut.to_owned(),
        ));
    }

    Ok((shortcut, template))
}

fn format_web_search_error(error: WebSearchParseError) -> String {
    match error {
        WebSearchParseError::EmptyShortcut => String::from("Web shortcut name is empty."),
        WebSearchParseError::InvalidEntry(entry) => {
            format!("Invalid web shortcut entry: {entry}. Use name=https://...?q={{query}}")
        }
        WebSearchParseError::TemplateMissingQuery(shortcut) => {
            format!("Web shortcut {shortcut} URL must contain {{query}}.")
        }
    }
}

fn percent_encode_query(query: &str) -> String {
    let mut encoded = String::new();

    for byte in query.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char)
            }
            b' ' => encoded.push('+'),
            byte => encoded.push_str(&format!("%{byte:02X}")),
        }
    }

    encoded
}

#[cfg(test)]
mod tests {
    use super::{parse_web_search_command, validate_web_shortcuts};
    use crate::settings::AppSettings;

    #[test]
    fn parses_configured_web_search() {
        let settings = AppSettings::default();
        let command = parse_web_search_command("g rust iced", &settings)
            .expect("command should parse")
            .expect("shortcut should exist");

        assert_eq!(command.shortcut, "g");
        assert_eq!(command.typed_shortcut, "g");
        assert_eq!(command.query, "rust iced");
        assert_eq!(command.url, "https://www.google.com/search?q=rust+iced");
    }

    #[test]
    fn still_accepts_slash_prefixed_web_search() {
        let settings = AppSettings::default();
        let command = parse_web_search_command("/yt rust iced", &settings)
            .expect("command should parse")
            .expect("shortcut should exist");

        assert_eq!(command.shortcut, "yt");
        assert_eq!(command.typed_shortcut, "/yt");
        assert_eq!(
            command.url,
            "https://www.youtube.com/results?search_query=rust+iced"
        );
    }

    #[test]
    fn ignores_unknown_slash_commands() {
        let settings = AppSettings::default();

        assert_eq!(
            parse_web_search_command("/project invoice", &settings).unwrap(),
            None
        );
    }

    #[test]
    fn validates_shortcut_templates() {
        assert!(validate_web_shortcuts("g=https://example.com?q={query}").is_ok());
        assert!(validate_web_shortcuts("g=https://example.com").is_err());
        assert!(validate_web_shortcuts("g=not-a-url?q={query}").is_err());
    }
}
