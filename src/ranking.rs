use crate::app_search::app_match_score;
use crate::types::{GroupedSearchResult, SearchResult, SearchResultKind};

use std::cmp::Reverse;
use std::fs;
use std::path::Path;

pub fn insert_result(
    groups: &mut Vec<GroupedSearchResult>,
    result: SearchResult,
    query: &str,
    exact_phrase: Option<&str>,
    history_boost: i64,
) {
    let mut should_sort = false;

    if let Some(existing) = groups.iter_mut().find(|group| group.path == result.path) {
        existing.match_count += 1;

        let candidate_score = score_result(&result, query, exact_phrase, history_boost);
        if candidate_score > existing.score {
            existing.title = result.title;
            existing.line_number = result.line_number;
            existing.snippet = result.snippet;
            existing.score = candidate_score;
            existing.icon_path = result.icon_path;
            should_sort = true;
        }
    } else {
        groups.push(GroupedSearchResult {
            score: score_result(&result, query, exact_phrase, history_boost),
            title: result.title,
            path: result.path,
            line_number: result.line_number,
            snippet: result.snippet,
            match_count: 1,
            kind: result.kind,
            icon_path: result.icon_path,
        });
        should_sort = true;
    }

    if should_sort {
        groups.sort_by_key(|group| Reverse(group.score));
    }
}

fn score_result(
    result: &SearchResult,
    query: &str,
    exact_phrase: Option<&str>,
    history_boost: i64,
) -> i64 {
    let filename = result
        .path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let path = result.path.to_string_lossy().to_ascii_lowercase();
    let snippet = result.snippet.to_ascii_lowercase();
    let query = query.to_ascii_lowercase();

    let mut score = 0;
    score += history_boost;

    if result.kind == SearchResultKind::Application {
        score += 900;
        score += app_match_score(&result.path, &query);
    }

    if result.kind == SearchResultKind::Calculator {
        score += 2_000;
    }

    if !query.is_empty() {
        if filename == query {
            score += 1_000;
        }

        if filename.contains(&query) {
            score += 700;
        }

        if path.contains(&query) {
            score += 250;
        }

        if snippet.contains(&query) {
            score += 180;
        }
    }

    if let Some(exact) = exact_phrase.map(str::to_ascii_lowercase) {
        if filename.contains(&exact) {
            score += 500;
        }

        if snippet.contains(&exact) {
            score += 350;
        }
    }

    score += recency_score(&result.path);
    score -= noisy_path_penalty(&path);
    score -= (path.matches('/').count() as i64).min(40) * 2;
    score -= (result.snippet.len() as i64 / 80).min(20);
    score -= result.line_number.unwrap_or(0).min(1_000) as i64 / 80;

    score
}

fn recency_score(path: &Path) -> i64 {
    let Ok(metadata) = fs::metadata(path) else {
        return 0;
    };

    let Ok(modified) = metadata.modified() else {
        return 0;
    };

    let Ok(age) = modified.elapsed() else {
        return 0;
    };

    let days = age.as_secs() / 86_400;

    match days {
        0..=7 => 120,
        8..=30 => 80,
        31..=180 => 40,
        181..=365 => 15,
        _ => 0,
    }
}

fn noisy_path_penalty(path: &str) -> i64 {
    ["library", "cache", "node_modules", "target", ".git"]
        .into_iter()
        .filter(|segment| path.contains(segment))
        .count() as i64
        * 150
}
