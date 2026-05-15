pub(super) fn ghost_completion(query: &str, selected_filename: &str) -> Option<String> {
    let query = query.trim();

    if query.is_empty() {
        return None;
    }

    let query_lower: Vec<char> = query.chars().flat_map(char::to_lowercase).collect();
    let filename_lower: Vec<char> = selected_filename
        .chars()
        .flat_map(char::to_lowercase)
        .collect();

    if query_lower.len() >= filename_lower.len() {
        return None;
    }

    if !filename_lower.starts_with(&query_lower) {
        return None;
    }

    let suffix: String = selected_filename
        .chars()
        .skip(query.chars().count())
        .collect();

    (!suffix.is_empty()).then_some(suffix)
}

#[cfg(test)]
mod tests {
    use super::ghost_completion;

    #[test]
    fn completes_ghost_suffix_case_insensitively() {
        assert_eq!(
            ghost_completion("docke", "Docker.app"),
            Some(String::from("r.app"))
        );

        assert_eq!(
            ghost_completion("test", "tests/book.py"),
            Some(String::from("s/book.py"))
        );
    }

    #[test]
    fn skips_invalid_ghost_completion() {
        assert_eq!(ghost_completion("", "Docker.app"), None);
        assert_eq!(ghost_completion("PDF", "project.pdf"), None);
        assert_eq!(ghost_completion("docker.app", "Docker.app"), None);
    }
}
