use crate::settings;

use std::env;
use std::path::{Path, PathBuf};

const REQUIRED_TOOLS: &[ToolCheck] = &[
    ToolCheck {
        binary: "rga",
        name: "ripgrep-all",
        required: true,
        macos_install: "brew install ripgrep-all",
        purpose: "file content search",
    },
    ToolCheck {
        binary: "fd",
        name: "fd",
        required: true,
        macos_install: "brew install fd",
        purpose: "folder search",
    },
    ToolCheck {
        binary: "zoxide",
        name: "zoxide",
        required: true,
        macos_install: "brew install zoxide",
        purpose: "folder scope shortcuts",
    },
];

#[derive(Debug, Clone, Copy)]
struct ToolCheck {
    binary: &'static str,
    name: &'static str,
    required: bool,
    macos_install: &'static str,
    purpose: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ToolStatus {
    binary: String,
    name: String,
    required: bool,
    purpose: String,
    install_command: String,
    path: Option<PathBuf>,
}

impl ToolStatus {
    fn ok(&self) -> bool {
        self.path.is_some() || !self.required
    }
}

pub fn requested() -> bool {
    env::args()
        .skip(1)
        .any(|argument| matches!(argument.as_str(), "doctor" | "--doctor" | "-d"))
}

pub fn run() -> i32 {
    let settings = settings::load();
    let mut statuses: Vec<ToolStatus> = REQUIRED_TOOLS.iter().map(check_tool).collect();

    statuses.push(ToolStatus {
        binary: settings.calculator_command.clone(),
        name: String::from("numbat"),
        required: false,
        purpose: String::from("calculator and unit conversion results"),
        install_command: String::from("cargo install numbat-cli"),
        path: find_executable(settings.calculator_command.trim()),
    });

    print_report(&statuses);

    if statuses.iter().all(ToolStatus::ok) {
        0
    } else {
        1
    }
}

fn check_tool(tool: &ToolCheck) -> ToolStatus {
    ToolStatus {
        binary: tool.binary.to_owned(),
        name: tool.name.to_owned(),
        required: tool.required,
        purpose: tool.purpose.to_owned(),
        install_command: tool.macos_install.to_owned(),
        path: find_executable(tool.binary),
    }
}

fn print_report(statuses: &[ToolStatus]) {
    println!("DeepLens doctor");
    println!();
    println!("Runtime PATH:");
    println!(
        "  {}",
        env::var("PATH").unwrap_or_else(|_| String::from("(not set)"))
    );
    println!();
    println!("Tool checks:");

    for status in statuses {
        let requirement = if status.required {
            "required"
        } else {
            "optional"
        };
        match status.path.as_ref() {
            Some(path) => {
                println!(
                    "  ok      {:<10} {:<8} {}",
                    status.binary,
                    requirement,
                    path.display()
                );
            }
            None => {
                println!(
                    "  missing {:<10} {:<8} {}",
                    status.binary, requirement, status.purpose
                );
                println!("          install: {}", status.install_command);
            }
        }
    }

    println!();

    if statuses.iter().all(ToolStatus::ok) {
        println!("DeepLens is ready.");
    } else {
        println!("Install the missing required tools, then run `DeepLens --doctor` again.");
    }
}

fn find_executable(binary: &str) -> Option<PathBuf> {
    if binary.trim().is_empty() {
        return None;
    }

    let binary_path = Path::new(binary);
    if binary_path.components().count() > 1 {
        return is_executable(binary_path).then(|| binary_path.to_path_buf());
    }

    env::var_os("PATH").and_then(|path| {
        env::split_paths(&path)
            .map(|directory| directory.join(binary))
            .find(|candidate| is_executable(candidate))
    })
}

fn is_executable(path: &Path) -> bool {
    path.is_file()
}

#[cfg(test)]
mod tests {
    use super::{find_executable, ToolStatus};

    #[test]
    fn optional_missing_tool_is_ok() {
        let status = ToolStatus {
            binary: String::from("missing"),
            name: String::from("missing"),
            required: false,
            purpose: String::new(),
            install_command: String::new(),
            path: None,
        };

        assert!(status.ok());
    }

    #[test]
    fn required_missing_tool_is_not_ok() {
        let status = ToolStatus {
            binary: String::from("missing"),
            name: String::from("missing"),
            required: true,
            purpose: String::new(),
            install_command: String::new(),
            path: None,
        };

        assert!(!status.ok());
    }

    #[test]
    fn finds_absolute_executable_path() {
        assert!(find_executable("/bin/sh").is_some());
    }
}
