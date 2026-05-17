use crate::settings::AppSettings;

use std::process::{Command, Stdio};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Calculation {
    pub expression: String,
}

pub fn parse_calculation(input: &str, settings: &AppSettings) -> Option<Calculation> {
    if !settings.calculator_enabled {
        return None;
    }

    let trimmed = input.trim();
    if trimmed.is_empty() || trimmed.starts_with('/') {
        return None;
    }

    if let Some(expression) = trimmed.strip_prefix("calc ") {
        return non_empty_calculation(expression);
    }

    if let Some(expression) = trimmed.strip_prefix('=') {
        return non_empty_calculation(expression);
    }

    if settings.calculator_requires_prefix {
        return None;
    }

    if looks_like_expression(trimmed) {
        return non_empty_calculation(trimmed);
    }

    None
}

pub async fn evaluate(expression: String, command: String) -> Result<String, String> {
    let output = Command::new(&command)
        .arg("-e")
        .arg(&expression)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null())
        .output()
        .map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                format!("numbat was not found. Install with: cargo install numbat-cli")
            } else {
                format!("Failed to start numbat: {error}")
            }
        })?;

    if !output.status.success() {
        let error = clean_output(&String::from_utf8_lossy(&output.stderr));
        return Err(if error.is_empty() {
            String::from("Numbat could not evaluate that expression.")
        } else {
            error
        });
    }

    let output = clean_output(&String::from_utf8_lossy(&output.stdout));
    last_result_line(&output).ok_or_else(|| String::from("Numbat returned no result."))
}

fn non_empty_calculation(expression: &str) -> Option<Calculation> {
    let expression = expression.trim();
    (!expression.is_empty()).then(|| Calculation {
        expression: expression.to_owned(),
    })
}

fn looks_like_expression(input: &str) -> bool {
    let has_digit = input.chars().any(|character| character.is_ascii_digit());
    let has_operator = input.contains("->")
        || input
            .chars()
            .any(|character| matches!(character, '+' | '-' | '*' | '/' | '^' | '=' | '(' | ')'));
    let has_unit_division = input
        .split_whitespace()
        .any(|token| token.contains('/') && token.chars().any(char::is_alphabetic));

    has_digit && (has_operator || has_unit_division)
}

fn last_result_line(output: &str) -> Option<String> {
    output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .last()
        .map(|line| line.trim_start_matches('=').trim().to_owned())
        .filter(|line| !line.is_empty())
}

fn clean_output(output: &str) -> String {
    strip_ansi(output).trim().to_owned()
}

fn strip_ansi(input: &str) -> String {
    let mut output = String::new();
    let mut chars = input.chars().peekable();

    while let Some(character) = chars.next() {
        if character == '\u{1b}' && chars.peek() == Some(&'[') {
            let _ = chars.next();
            for next in chars.by_ref() {
                if next.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            output.push(character);
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::{last_result_line, parse_calculation};
    use crate::settings::AppSettings;

    #[test]
    fn detects_calculation_inputs() {
        let settings = AppSettings::default();

        assert_eq!(
            parse_calculation("2 + 2", &settings).map(|calc| calc.expression),
            Some(String::from("2 + 2"))
        );
        assert_eq!(
            parse_calculation("=30 km/h -> mph", &settings).map(|calc| calc.expression),
            Some(String::from("30 km/h -> mph"))
        );
        assert_eq!(
            parse_calculation("calc 5 ft + 2 in -> cm", &settings).map(|calc| calc.expression),
            Some(String::from("5 ft + 2 in -> cm"))
        );
        assert!(parse_calculation("10 / 2", &settings).is_some());
        assert!(parse_calculation("10 - 2", &settings).is_some());
    }

    #[test]
    fn ignores_plain_search_inputs() {
        let settings = AppSettings::default();

        assert!(parse_calculation("docker", &settings).is_none());
        assert!(parse_calculation("2026 taxes", &settings).is_none());
        assert!(parse_calculation("/g rust", &settings).is_none());
    }

    #[test]
    fn supports_prefix_only_mode() {
        let settings = AppSettings {
            calculator_requires_prefix: true,
            ..AppSettings::default()
        };

        assert!(parse_calculation("2 + 2", &settings).is_none());
        assert!(parse_calculation("=2 + 2", &settings).is_some());
    }

    #[test]
    fn extracts_result_line() {
        assert_eq!(
            last_result_line("\n  = 7.5 km\n"),
            Some(String::from("7.5 km"))
        );
    }
}
