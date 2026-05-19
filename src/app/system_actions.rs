use crate::settings::AppSettings;

use std::path::{Path, PathBuf};

pub(super) fn reveal_in_file_manager(path: &Path) -> std::io::Result<()> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg("-R")
            .arg(path)
            .spawn()?;
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(format!("/select,{}", path.display()))
            .spawn()?;
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let target = if path.is_dir() {
            path.to_path_buf()
        } else {
            path.parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| path.to_path_buf())
        };

        std::process::Command::new("xdg-open").arg(target).spawn()?;
    }

    Ok(())
}

pub(super) fn terminal_directory_for(path: &Path) -> PathBuf {
    if path.is_dir() {
        path.to_path_buf()
    } else {
        path.parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| path.to_path_buf())
    }
}

pub(super) fn open_terminal_at(directory: &Path, settings: &AppSettings) -> std::io::Result<()> {
    if let Some(terminal) = std::env::var_os("DEEPLENS_TERMINAL") {
        return std::process::Command::new(terminal)
            .current_dir(directory)
            .spawn()
            .map(|_| ());
    }

    #[cfg(target_os = "macos")]
    {
        let terminal_app = settings.terminal_app.trim();
        let terminal_app = if terminal_app.is_empty() {
            "Terminal"
        } else {
            terminal_app
        };

        return std::process::Command::new("open")
            .arg("-a")
            .arg(terminal_app)
            .arg(directory)
            .spawn()
            .map(|_| ());
    }

    #[cfg(target_os = "windows")]
    {
        return std::process::Command::new("wt")
            .arg("-d")
            .arg(directory)
            .spawn()
            .or_else(|_| {
                std::process::Command::new("cmd")
                    .current_dir(directory)
                    .spawn()
            })
            .map(|_| ());
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        for terminal in [
            "x-terminal-emulator",
            "gnome-terminal",
            "konsole",
            "xfce4-terminal",
        ] {
            if std::process::Command::new(terminal)
                .current_dir(directory)
                .spawn()
                .is_ok()
            {
                return Ok(());
            }
        }

        std::process::Command::new("xterm")
            .current_dir(directory)
            .spawn()
            .map(|_| ())
    }
}

pub(super) fn preview_with_quick_look(path: &Path) -> std::io::Result<()> {
    #[cfg(target_os = "macos")]
    {
        return std::process::Command::new("qlmanage")
            .arg("-p")
            .arg(path)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map(|_| ());
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = path;
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Quick Look preview is currently only supported on macOS",
        ))
    }
}
