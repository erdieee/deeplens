use crate::app::FileFilter;
use crate::types::SearchMode;

use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedQuery {
    pub original: String,
    pub search_text: String,
    pub exact_phrase: Option<String>,
    pub filter: Option<FileFilter>,
    pub mode: Option<SearchMode>,
    pub scope: Option<PathBuf>,
    pub scope_error: Option<String>,
}

pub fn parse_query(input: &str) -> ParsedQuery {
    let mut tokens = split_query(input);
    let mut filter = None;
    let mut mode = None;
    let mut scope = None;
    let mut scope_error = None;

    loop {
        let Some(first) = tokens.first().cloned() else {
            break;
        };

        let lower = first.to_ascii_lowercase();
        let mut consumed = false;

        if let Some(search_mode) = slash_search_mode(&lower) {
            mode = Some(search_mode);
            consumed = true;
        } else if let Some(slash_scope) = resolve_slash_scope(&first) {
            match slash_scope {
                Ok(path) => {
                    scope = Some(path);
                    consumed = true;
                }
                Err(error) => {
                    scope_error = Some(error);
                    consumed = true;
                }
            }
        } else if tokens.len() > 1 {
            match lower.as_str() {
                "pdf" => {
                    filter = Some(FileFilter::Pdf);
                    consumed = true;
                }
                "doc" | "docs" => {
                    filter = Some(FileFilter::Docs);
                    consumed = true;
                }
                "txt" | "text" => {
                    filter = Some(FileFilter::Text);
                    consumed = true;
                }
                "code" => {
                    filter = Some(FileFilter::Code);
                    consumed = true;
                }
                _ => {}
            }
        }

        if consumed {
            tokens.remove(0);
        } else {
            break;
        }
    }

    let exact_phrase = extract_single_quoted_phrase(&tokens);
    let search_text = exact_phrase
        .clone()
        .unwrap_or_else(|| tokens.join(" ").trim().to_owned());

    ParsedQuery {
        original: input.to_owned(),
        search_text,
        exact_phrase,
        filter,
        mode,
        scope,
        scope_error,
    }
}

fn slash_search_mode(token: &str) -> Option<SearchMode> {
    match token {
        "/all" => Some(SearchMode::All),
        "/app" | "/apps" => Some(SearchMode::Applications),
        "/folder" | "/folders" => Some(SearchMode::Folders),
        "/file" | "/files" => Some(SearchMode::Files),
        _ => None,
    }
}

fn resolve_slash_scope(token: &str) -> Option<Result<PathBuf, String>> {
    if !token.starts_with('/') {
        return None;
    }

    if token == "/" {
        return Some(Ok(PathBuf::from("/")));
    }

    let absolute_path = PathBuf::from(token);
    if absolute_path.exists() {
        return Some(Ok(absolute_path));
    }

    let zoxide_query = token.trim_start_matches('/');
    if zoxide_query.is_empty() {
        return Some(Ok(PathBuf::from("/")));
    }

    let output = Command::new("zoxide")
        .arg("query")
        .arg(zoxide_query)
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_owned();

            if path.is_empty() {
                Some(Err(format!("No zoxide match for /{zoxide_query}")))
            } else {
                Some(Ok(PathBuf::from(path)))
            }
        }
        Ok(_) => Some(Err(format!("No zoxide match for /{zoxide_query}"))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Some(Err(String::from("zoxide was not found on PATH")))
        }
        Err(error) => Some(Err(format!("Failed to query zoxide: {error}"))),
    }
}

fn extract_single_quoted_phrase(tokens: &[String]) -> Option<String> {
    if tokens.len() != 1 {
        return None;
    }

    let token = tokens[0].trim();

    token
        .strip_prefix('"')
        .and_then(|text| text.strip_suffix('"'))
        .filter(|text| !text.is_empty())
        .map(ToOwned::to_owned)
}

fn split_query(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for character in input.chars() {
        match character {
            '"' => {
                current.push(character);
                in_quotes = !in_quotes;
            }
            character if character.is_whitespace() && !in_quotes => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            character => current.push(character),
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::parse_query;
    use crate::app::FileFilter;

    #[test]
    fn parses_file_prefix() {
        let parsed = parse_query("pdf stripe invoice");

        assert_eq!(parsed.filter, Some(FileFilter::Pdf));
        assert_eq!(parsed.search_text, "stripe invoice");

        let parsed = parse_query("doc project notes");
        assert_eq!(parsed.filter, Some(FileFilter::Docs));
        assert_eq!(parsed.search_text, "project notes");

        let parsed = parse_query("txt api key");
        assert_eq!(parsed.filter, Some(FileFilter::Text));
        assert_eq!(parsed.search_text, "api key");
    }

    #[test]
    fn does_not_parse_incomplete_file_prefix_as_mode_switch() {
        let parsed = parse_query("doc");

        assert_eq!(parsed.filter, None);
        assert_eq!(parsed.search_text, "doc");

        let parsed = parse_query("docker");
        assert_eq!(parsed.filter, None);
        assert_eq!(parsed.search_text, "docker");
    }

    #[test]
    fn parses_exact_phrase() {
        let parsed = parse_query("\"stripe connect\"");

        assert_eq!(parsed.exact_phrase.as_deref(), Some("stripe connect"));
        assert_eq!(parsed.search_text, "stripe connect");
    }

    #[test]
    fn parses_root_slash_scope() {
        let parsed = parse_query("/ invoice");

        assert_eq!(parsed.scope, Some(std::path::PathBuf::from("/")));
        assert_eq!(parsed.search_text, "invoice");
    }

    #[test]
    fn parses_slash_search_modes() {
        let parsed = parse_query("/apps docker");

        assert_eq!(parsed.mode, Some(crate::types::SearchMode::Applications));
        assert_eq!(parsed.search_text, "docker");

        let parsed = parse_query("/folders tests");
        assert_eq!(parsed.mode, Some(crate::types::SearchMode::Folders));
        assert_eq!(parsed.search_text, "tests");
    }
}
