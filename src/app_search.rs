use std::path::Path;

pub fn app_display_name(path: &Path) -> String {
    if path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("app"))
    {
        return path
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_owned();
    }

    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_owned()
}

pub fn app_match_score(path: &Path, query: &str) -> i64 {
    let query = query.trim();
    if query.is_empty() {
        return 0;
    }

    let name = app_display_name(path);
    let name_lower = name.to_ascii_lowercase();
    let query_lower = query.to_ascii_lowercase();
    let normalized_name = normalize(&name);
    let normalized_query = normalize(query);

    if normalized_query.is_empty() {
        return 0;
    }

    if name_lower == query_lower || normalized_name == normalized_query {
        return 1_600;
    }

    if app_aliases(&normalized_name)
        .into_iter()
        .any(|alias| alias == normalized_query)
    {
        return 1_500;
    }

    if name_lower.starts_with(&query_lower) || normalized_name.starts_with(&normalized_query) {
        return 1_300;
    }

    if name_lower.contains(&query_lower) || normalized_name.contains(&normalized_query) {
        return 1_000;
    }

    if query_matches_words(&name, query) {
        return 900;
    }

    if normalized_query.len() >= 3 && fuzzy_subsequence(&normalized_query, &normalized_name) {
        return 550;
    }

    0
}

fn normalize(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn app_aliases(normalized_name: &str) -> Vec<&'static str> {
    match normalized_name {
        "visualstudiocode" => vec!["vscode", "vsc", "code"],
        "googlechrome" => vec!["chrome", "gc"],
        "iterm" | "iterm2" => vec!["iterm", "terminal", "term"],
        "terminal" => vec!["term", "shell"],
        "systemsettings" => vec!["settings", "preferences", "prefs"],
        "activitymonitor" => vec!["activity", "monitor"],
        "finder" => vec!["files"],
        _ => Vec::new(),
    }
}

fn query_matches_words(name: &str, query: &str) -> bool {
    let words = words(name);
    if words.is_empty() {
        return false;
    }

    let acronym: String = words
        .iter()
        .filter_map(|word| word.chars().next())
        .collect();

    query.split_whitespace().all(|term| {
        let normalized_term = normalize(term);
        !normalized_term.is_empty()
            && (acronym.starts_with(&normalized_term)
                || words.iter().any(|word| word.starts_with(&normalized_term)))
    })
}

fn words(value: &str) -> Vec<String> {
    value
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter_map(|word| {
            let word = normalize(word);
            (!word.is_empty()).then_some(word)
        })
        .collect()
}

fn fuzzy_subsequence(query: &str, candidate: &str) -> bool {
    let mut candidate_chars = candidate.chars();

    query
        .chars()
        .all(|query_char| candidate_chars.any(|candidate_char| candidate_char == query_char))
}

#[cfg(test)]
mod tests {
    use super::app_match_score;
    use std::path::Path;

    #[test]
    fn matches_common_app_aliases() {
        assert!(app_match_score(Path::new("/Applications/Visual Studio Code.app"), "vs code") > 0);
        assert!(app_match_score(Path::new("/Applications/Visual Studio Code.app"), "vscode") > 0);
        assert!(app_match_score(Path::new("/Applications/Google Chrome.app"), "chrome") > 0);
    }

    #[test]
    fn matches_fuzzy_app_names() {
        assert!(app_match_score(Path::new("/Applications/Docker.app"), "dockr") > 0);
        assert!(app_match_score(Path::new("/Applications/Activity Monitor.app"), "act mon") > 0);
    }

    #[test]
    fn rejects_unrelated_apps() {
        assert_eq!(
            app_match_score(Path::new("/Applications/Docker.app"), "photos"),
            0
        );
    }
}
