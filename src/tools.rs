use std::collections::HashSet;
use std::env;
use std::path::PathBuf;

pub fn install_runtime_path() {
    let mut paths = Vec::new();

    paths.extend(app_resource_paths());
    paths.extend(common_tool_paths());

    if let Some(existing) = env::var_os("PATH") {
        paths.extend(env::split_paths(&existing));
    }

    let mut seen = HashSet::new();
    paths.retain(|path| seen.insert(path.clone()));

    if let Ok(path) = env::join_paths(paths) {
        env::set_var("PATH", path);
    }
}

fn app_resource_paths() -> Vec<PathBuf> {
    let Ok(executable) = env::current_exe() else {
        return Vec::new();
    };

    let Some(macos_dir) = executable.parent() else {
        return Vec::new();
    };

    let Some(contents_dir) = macos_dir.parent().filter(|path| {
        path.file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name == "Contents")
    }) else {
        return Vec::new();
    };

    let resources = contents_dir.join("Resources");
    vec![resources.join("bin"), resources]
}

fn common_tool_paths() -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = [
        "/opt/homebrew/bin",
        "/usr/local/bin",
        "/opt/local/bin",
        "/usr/bin",
        "/bin",
        "/usr/sbin",
        "/sbin",
    ]
    .into_iter()
    .map(PathBuf::from)
    .collect();

    if let Some(home) = env::var_os("HOME") {
        paths.push(PathBuf::from(home).join(".cargo").join("bin"));
    }

    paths
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn common_paths_include_homebrew_locations() {
        let paths = common_tool_paths();

        assert!(paths.contains(&PathBuf::from("/opt/homebrew/bin")));
        assert!(paths.contains(&PathBuf::from("/usr/local/bin")));
    }

    #[test]
    fn common_paths_include_cargo_bin_when_home_exists() {
        if let Some(home) = env::var_os("HOME") {
            let paths = common_tool_paths();
            assert!(paths.contains(&PathBuf::from(home).join(".cargo").join("bin")));
        }
    }
}
