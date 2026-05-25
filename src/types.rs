use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub title: Option<String>,
    pub path: PathBuf,
    pub line_number: Option<u64>,
    pub snippet: String,
    pub kind: SearchResultKind,
    pub icon_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct GroupedSearchResult {
    pub title: Option<String>,
    pub path: PathBuf,
    pub line_number: Option<u64>,
    pub snippet: String,
    pub match_count: usize,
    pub score: i64,
    pub kind: SearchResultKind,
    pub icon_path: Option<PathBuf>,
    pub pinned: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchResultKind {
    FileContent,
    FolderName,
    Application,
    Calculator,
    Clipboard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMode {
    Files,
    Folders,
    Applications,
    All,
}

#[derive(Debug, Clone)]
pub enum SearchEvent {
    Result(SearchResult),
    Finished,
    Cancelled,
    Error(String),
}
