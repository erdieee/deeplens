use crate::app_search::{app_display_name, app_match_score};
use crate::types::{SearchEvent, SearchMode, SearchResult, SearchResultKind};

use anyhow::{Context, Result};
use crossbeam_channel::{unbounded, Receiver, Sender};
use serde_json::Value;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread::{self, JoinHandle};

#[cfg(unix)]
use std::os::unix::process::CommandExt;

const RGA_MISSING_MESSAGE: &str =
    "ripgrep-all (rga) was not found. Install with: brew install ripgrep-all";

#[derive(Debug, Clone)]
pub struct SearchOptions {
    pub query: String,
    pub file_folder_scope: PathBuf,
    pub exact: bool,
    pub include_globs: Vec<String>,
    pub excluded_folders: Vec<String>,
    pub mode: SearchMode,
}

pub struct SearchHandle {
    receiver: Receiver<SearchEvent>,
    children: Arc<Mutex<Vec<Child>>>,
    cancel_requested: Arc<AtomicBool>,
    _worker: JoinHandle<()>,
}

impl SearchHandle {
    pub fn cancel(&mut self) {
        self.cancel_requested.store(true, Ordering::SeqCst);

        if let Ok(mut children) = self.children.lock() {
            for process in children.iter_mut() {
                terminate_process_tree(process);
            }
        }
    }

    pub fn try_recv(&self) -> Option<SearchEvent> {
        self.receiver.try_recv().ok()
    }
}

impl Drop for SearchHandle {
    fn drop(&mut self) {
        self.cancel();
    }
}

pub fn start_search(options: SearchOptions) -> Result<SearchHandle> {
    let (sender, receiver) = unbounded();
    let children = Arc::new(Mutex::new(Vec::new()));
    let cancel_requested = Arc::new(AtomicBool::new(false));

    let worker_children = Arc::clone(&children);
    let worker_cancel_requested = Arc::clone(&cancel_requested);
    let worker = thread::spawn(move || {
        let read_result =
            run_search_commands(options, &sender, &worker_children, &worker_cancel_requested);

        let event = match read_result {
            Err(error) => SearchEvent::Error(error),
            Ok(()) if worker_cancel_requested.load(Ordering::SeqCst) => SearchEvent::Cancelled,
            Ok(()) => SearchEvent::Finished,
        };

        let _ = sender.send(event);
    });

    Ok(SearchHandle {
        receiver,
        children,
        cancel_requested,
        _worker: worker,
    })
}

fn run_search_commands(
    options: SearchOptions,
    sender: &Sender<SearchEvent>,
    children: &Arc<Mutex<Vec<Child>>>,
    cancel_requested: &Arc<AtomicBool>,
) -> Result<(), String> {
    if options.mode == SearchMode::All {
        return run_all_commands(options, sender, children, cancel_requested);
    }

    match options.mode {
        SearchMode::Files => run_rga(&options, sender, children, cancel_requested)?,
        SearchMode::Folders => run_fd(&options, sender, children, cancel_requested)?,
        SearchMode::Applications => run_application_search(&options, sender, cancel_requested)?,
        SearchMode::All => unreachable!(),
    }

    Ok(())
}

fn run_all_commands(
    options: SearchOptions,
    sender: &Sender<SearchEvent>,
    children: &Arc<Mutex<Vec<Child>>>,
    cancel_requested: &Arc<AtomicBool>,
) -> Result<(), String> {
    thread::scope(|scope| {
        let fd_options = options.clone();
        let app_options = options.clone();

        let fd_search =
            scope.spawn(move || run_fd(&fd_options, sender, children, cancel_requested));
        let app_search =
            scope.spawn(move || run_application_search(&app_options, sender, cancel_requested));
        let rga_search = scope.spawn(move || run_rga(&options, sender, children, cancel_requested));

        let fd_result = fd_search
            .join()
            .unwrap_or_else(|_| Err(String::from("Folder search failed unexpectedly")));
        let app_result = app_search
            .join()
            .unwrap_or_else(|_| Err(String::from("Application search failed unexpectedly")));
        let rga_result = rga_search
            .join()
            .unwrap_or_else(|_| Err(String::from("File search failed unexpectedly")));

        fd_result.and(app_result).and(rga_result)
    })
}

fn run_application_search(
    options: &SearchOptions,
    sender: &Sender<SearchEvent>,
    cancel_requested: &Arc<AtomicBool>,
) -> Result<(), String> {
    let query = options.query.to_ascii_lowercase();

    for root in application_roots() {
        if cancel_requested.load(Ordering::SeqCst) {
            break;
        }

        collect_application_matches(&root, &query, sender, cancel_requested)?;
    }

    Ok(())
}

fn collect_application_matches(
    root: &Path,
    query: &str,
    sender: &Sender<SearchEvent>,
    cancel_requested: &Arc<AtomicBool>,
) -> Result<(), String> {
    if cancel_requested.load(Ordering::SeqCst) || !root.exists() {
        return Ok(());
    }

    let entries = match std::fs::read_dir(root) {
        Ok(entries) => entries,
        Err(_) => return Ok(()),
    };

    for entry in entries {
        if cancel_requested.load(Ordering::SeqCst) {
            break;
        }

        let Ok(entry) = entry else {
            continue;
        };

        let path = entry.path();
        if is_app_bundle(&path) {
            if app_match_score(&path, query) > 0
                && sender
                    .send(SearchEvent::Result(SearchResult {
                        icon_path: application_icon_png(&path),
                        snippet: format!("Application: {}", app_display_name(&path)),
                        path,
                        line_number: None,
                        kind: SearchResultKind::Application,
                    }))
                    .is_err()
            {
                break;
            }

            continue;
        }

        if path.is_dir() {
            collect_application_matches(&path, query, sender, cancel_requested)?;
        }
    }

    Ok(())
}

fn application_roots() -> Vec<PathBuf> {
    let mut roots = vec![
        PathBuf::from("/Applications"),
        PathBuf::from("/System/Applications"),
    ];

    if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
        roots.push(home.join("Applications"));
    }

    roots
}

fn is_app_bundle(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("app"))
}

fn application_icon_png(app_bundle: &Path) -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        let icon = app_icon_file(app_bundle)?;
        let cache_path = app_icon_cache_path(app_bundle);

        if cache_path.exists() {
            return Some(cache_path);
        }

        if let Some(parent) = cache_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let status = Command::new("/usr/bin/sips")
            .arg("-s")
            .arg("format")
            .arg("png")
            .arg(&icon)
            .arg("--out")
            .arg(&cache_path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .ok()?;

        status.success().then_some(cache_path)
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = app_bundle;
        None
    }
}

#[cfg(target_os = "macos")]
fn app_icon_file(app_bundle: &Path) -> Option<PathBuf> {
    let resources = app_bundle.join("Contents").join("Resources");
    let plist = app_bundle.join("Contents").join("Info.plist");

    if let Some(icon_name) = plist_icon_name(&plist) {
        let icon_name = if icon_name.ends_with(".icns") {
            icon_name
        } else {
            format!("{icon_name}.icns")
        };

        let icon = resources.join(icon_name);
        if icon.exists() {
            return Some(icon);
        }
    }

    std::fs::read_dir(resources)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| {
            path.extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("icns"))
        })
}

#[cfg(target_os = "macos")]
fn plist_icon_name(plist: &Path) -> Option<String> {
    let output = Command::new("/usr/bin/plutil")
        .arg("-extract")
        .arg("CFBundleIconFile")
        .arg("raw")
        .arg("-o")
        .arg("-")
        .arg(plist)
        .stderr(Stdio::null())
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let value = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    (!value.is_empty()).then_some(value)
}

#[cfg(target_os = "macos")]
fn app_icon_cache_path(app_bundle: &Path) -> PathBuf {
    let mut hasher = DefaultHasher::new();
    app_bundle.hash(&mut hasher);

    std::env::temp_dir()
        .join("deeplens-app-icons")
        .join(format!("{:x}.png", hasher.finish()))
}

fn run_rga(
    options: &SearchOptions,
    sender: &Sender<SearchEvent>,
    children: &Arc<Mutex<Vec<Child>>>,
    cancel_requested: &Arc<AtomicBool>,
) -> Result<(), String> {
    let mut command = Command::new("rga");
    command
        .arg("--json")
        .arg("--line-number")
        .arg("--threads")
        .arg("2")
        .arg("--max-filesize")
        .arg("20M")
        .arg("--max-columns")
        .arg("600")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .stdin(Stdio::null());

    if options.exact {
        command.arg("--fixed-strings");
    }

    for glob in &options.include_globs {
        command.arg("--glob").arg(glob);
    }

    for folder in &options.excluded_folders {
        command
            .arg("--glob")
            .arg(format!("!{}/**", folder.trim_matches('/')));
    }

    command.arg(&options.query).arg(&options.file_folder_scope);

    run_command(command, children, cancel_requested, |stdout| {
        read_rga_stdout(stdout, sender)
    })
}

fn run_fd(
    options: &SearchOptions,
    sender: &Sender<SearchEvent>,
    children: &Arc<Mutex<Vec<Child>>>,
    cancel_requested: &Arc<AtomicBool>,
) -> Result<(), String> {
    let mut command = Command::new("fd");
    command
        .arg("--type")
        .arg("directory")
        .arg("--absolute-path")
        .arg("--hidden")
        .arg("--max-results")
        .arg("300")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .stdin(Stdio::null());

    for folder in &options.excluded_folders {
        command.arg("--exclude").arg(folder);
    }

    command.arg(&options.query).arg(&options.file_folder_scope);

    run_command(command, children, cancel_requested, |stdout| {
        read_fd_stdout(stdout, sender)
    })
}

fn run_command(
    mut command: Command,
    children: &Arc<Mutex<Vec<Child>>>,
    cancel_requested: &Arc<AtomicBool>,
    read_stdout: impl FnOnce(Box<dyn std::io::Read + Send>) -> Result<(), String>,
) -> Result<(), String> {
    #[cfg(unix)]
    unsafe {
        command.pre_exec(|| {
            if libc::setsid() == -1 {
                return Err(std::io::Error::last_os_error());
            }

            Ok(())
        });
    }

    let mut process = command.spawn().map_err(|error| {
        let program = command.get_program().to_string_lossy();

        if error.kind() == std::io::ErrorKind::NotFound && program == "rga" {
            RGA_MISSING_MESSAGE.to_owned()
        } else if error.kind() == std::io::ErrorKind::NotFound && program == "fd" {
            String::from("fd was not found. Install it with: brew install fd")
        } else {
            format!("Failed to start {program}: {error}")
        }
    })?;

    let stdout = process
        .stdout
        .take()
        .context("Failed to capture command stdout")
        .map_err(|error| error.to_string())?;
    let process_id = process.id();

    if let Ok(mut child_list) = children.lock() {
        child_list.push(process);
    }

    let read_result = read_stdout(Box::new(stdout));

    let process_to_wait = if let Ok(mut child_list) = children.lock() {
        child_list
            .iter()
            .position(|process| process.id() == process_id)
            .map(|position| child_list.swap_remove(position))
    } else {
        None
    };

    if let Some(mut process) = process_to_wait {
        if cancel_requested.load(Ordering::SeqCst) {
            terminate_process_tree(&mut process);
        }

        let _ = process.wait();
    }

    read_result
}

fn terminate_process_tree(process: &mut Child) {
    #[cfg(unix)]
    {
        let process_group_id = process.id() as libc::pid_t;

        unsafe {
            let _ = libc::kill(-process_group_id, libc::SIGKILL);
        }
    }

    #[cfg(not(unix))]
    {
        let _ = process.kill();
    }
}

pub fn validate_search(query: &str, folder: &Path) -> Result<()> {
    if query.trim().is_empty() {
        anyhow::bail!("Enter a search query before starting a search.");
    }

    if folder.as_os_str().is_empty() {
        anyhow::bail!("Choose a folder before starting a search.");
    }

    if !folder.exists() {
        anyhow::bail!("The selected folder does not exist.");
    }

    if !folder.is_dir() {
        anyhow::bail!("The selected path is not a folder.");
    }

    Ok(())
}

fn read_rga_stdout(stdout: impl std::io::Read, sender: &Sender<SearchEvent>) -> Result<(), String> {
    let reader = BufReader::new(stdout);

    for line in reader.lines() {
        let line = line.map_err(|error| format!("Failed to read rga output: {error}"))?;

        let Some(result) = parse_match_line(&line) else {
            continue;
        };

        if sender.send(SearchEvent::Result(result)).is_err() {
            break;
        }
    }

    Ok(())
}

fn read_fd_stdout(stdout: impl std::io::Read, sender: &Sender<SearchEvent>) -> Result<(), String> {
    let reader = BufReader::new(stdout);

    for line in reader.lines() {
        let line = line.map_err(|error| format!("Failed to read fd output: {error}"))?;
        let path = line.trim();

        if path.is_empty() {
            continue;
        }

        if sender
            .send(SearchEvent::Result(SearchResult {
                path: PathBuf::from(path),
                line_number: None,
                snippet: String::from("Folder name match"),
                kind: SearchResultKind::FolderName,
                icon_path: None,
            }))
            .is_err()
        {
            break;
        }
    }

    Ok(())
}

fn parse_match_line(line: &str) -> Option<SearchResult> {
    let value: Value = serde_json::from_str(line).ok()?;

    if value.get("type")?.as_str()? != "match" {
        return None;
    }

    let data = value.get("data")?;
    let path = text_value(data.get("path")?)?;
    let line_number = data.get("line_number").and_then(Value::as_u64);
    let snippet = data
        .get("lines")
        .and_then(text_value)
        .unwrap_or_default()
        .trim_end_matches(['\r', '\n'])
        .to_owned();

    Some(SearchResult {
        path: PathBuf::from(path),
        line_number,
        snippet,
        kind: SearchResultKind::FileContent,
        icon_path: None,
    })
}

fn text_value(value: &Value) -> Option<String> {
    value
        .get("text")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .or_else(|| {
            value
                .get("bytes")
                .and_then(Value::as_str)
                .and_then(|bytes| String::from_utf8(base64_decode(bytes)?).ok())
        })
}

fn base64_decode(input: &str) -> Option<Vec<u8>> {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut output = Vec::new();
    let mut buffer = 0u32;
    let mut bits = 0u8;

    for byte in input.bytes() {
        if byte == b'=' {
            break;
        }

        let value = TABLE.iter().position(|candidate| *candidate == byte)? as u32;
        buffer = (buffer << 6) | value;
        bits += 6;

        if bits >= 8 {
            bits -= 8;
            output.push(((buffer >> bits) & 0xff) as u8);
        }
    }

    Some(output)
}

#[cfg(test)]
mod tests {
    use super::parse_match_line;

    #[test]
    fn parses_ripgrep_match_event() {
        let line = r#"{"type":"match","data":{"path":{"text":"/tmp/example.txt"},"lines":{"text":"hello world\n"},"line_number":42}}"#;

        let result = parse_match_line(line).expect("match event should parse");

        assert_eq!(result.path.to_string_lossy(), "/tmp/example.txt");
        assert_eq!(result.line_number, Some(42));
        assert_eq!(result.snippet, "hello world");
    }

    #[test]
    fn ignores_non_match_events() {
        let line = r#"{"type":"begin","data":{"path":{"text":"/tmp/example.txt"}}}"#;

        assert!(parse_match_line(line).is_none());
    }

    #[test]
    fn ignores_invalid_json() {
        assert!(parse_match_line("not json").is_none());
    }
}
