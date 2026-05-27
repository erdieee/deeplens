use crate::settings::AppSettings;

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct CommandBook {
    commands: Vec<CustomCommand>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CustomCommand {
    pub name: String,
    pub alias: String,
    pub command: String,
    pub args_template: String,
    pub working_dir: String,
    pub open_in_terminal: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedCommand {
    pub id: String,
    pub name: String,
    pub alias: String,
    pub command_line: String,
    pub working_dir: Option<PathBuf>,
    pub open_in_terminal: bool,
}

impl Default for CustomCommand {
    fn default() -> Self {
        Self {
            name: String::new(),
            alias: String::new(),
            command: String::new(),
            args_template: String::new(),
            working_dir: String::new(),
            open_in_terminal: false,
        }
    }
}

impl CommandBook {
    pub fn from_commands(commands: Vec<CustomCommand>) -> Self {
        Self { commands }
    }

    pub fn load() -> Self {
        let Some(path) = commands_path() else {
            return Self::default();
        };

        let Some(text) = fs::read_to_string(path).ok() else {
            return Self::default();
        };

        serde_json::from_str::<Vec<CustomCommand>>(&text)
            .map(|commands| Self { commands })
            .or_else(|_| serde_json::from_str::<Self>(&text))
            .unwrap_or_default()
    }

    pub fn search(&self, query: &str, settings: &AppSettings) -> Vec<ResolvedCommand> {
        if !settings.custom_commands_enabled {
            return Vec::new();
        }

        let query = query.trim().to_ascii_lowercase();
        let mut matches: Vec<(i64, ResolvedCommand)> = self
            .commands
            .iter()
            .filter(|command| command.is_valid())
            .filter_map(|command| {
                let score = command.search_score(&query)?;
                Some((score, command.resolve("")))
            })
            .collect();

        matches.sort_by_key(|(score, _)| std::cmp::Reverse(*score));
        matches
            .into_iter()
            .map(|(_, command)| command)
            .take(settings.max_custom_command_results)
            .collect()
    }

    pub fn direct_match(&self, input: &str, settings: &AppSettings) -> Option<ResolvedCommand> {
        if !settings.custom_commands_enabled {
            return None;
        }

        let trimmed = input.trim();
        let mut parts = trimmed.splitn(2, char::is_whitespace);
        let alias = parts.next()?.to_ascii_lowercase();
        let args = parts.next().map(str::trim).unwrap_or_default();

        self.commands
            .iter()
            .filter(|command| command.is_valid())
            .find(|command| command.alias.to_ascii_lowercase() == alias)
            .map(|command| command.resolve(args))
    }

    pub fn resolve_by_id(&self, id: &str, query: &str) -> Option<ResolvedCommand> {
        self.commands
            .iter()
            .filter(|command| command.is_valid())
            .find(|command| command.id() == id)
            .map(|command| command.resolve(query))
    }

    pub fn commands(&self) -> &[CustomCommand] {
        &self.commands
    }
}

impl CustomCommand {
    pub fn is_valid(&self) -> bool {
        !self.alias.trim().is_empty() && !self.command.trim().is_empty()
    }

    fn id(&self) -> String {
        self.alias.trim().to_owned()
    }

    pub fn resolve(&self, query: &str) -> ResolvedCommand {
        let args = self.args_template.replace("{query}", query.trim());
        let command_line = join_command_line(&self.command, &args);
        let working_dir =
            expand_home(self.working_dir.trim()).filter(|path| !path.as_os_str().is_empty());

        ResolvedCommand {
            id: self.id(),
            name: if self.name.trim().is_empty() {
                self.alias.trim().to_owned()
            } else {
                self.name.trim().to_owned()
            },
            alias: self.alias.trim().to_owned(),
            command_line,
            working_dir,
            open_in_terminal: self.open_in_terminal,
        }
    }

    fn search_score(&self, query: &str) -> Option<i64> {
        if query.is_empty() {
            return Some(0);
        }

        let alias = self.alias.to_ascii_lowercase();
        let name = self.name.to_ascii_lowercase();
        let command = self.command.to_ascii_lowercase();

        if alias == query {
            return Some(1_000);
        }

        if alias.starts_with(query) {
            return Some(900);
        }

        if name.starts_with(query) {
            return Some(800);
        }

        if alias.contains(query) {
            return Some(700);
        }

        if name.contains(query) {
            return Some(600);
        }

        command.contains(query).then_some(300)
    }
}

pub fn parse_command_query(input: &str) -> Option<String> {
    let trimmed = input.trim();
    let mut parts = trimmed.splitn(2, char::is_whitespace);
    let command = parts.next()?.to_ascii_lowercase();

    if command != "cmd" && command != "command" && command != "commands" {
        return None;
    }

    Some(parts.next().map(str::trim).unwrap_or_default().to_owned())
}

pub fn save_commands(commands: &[CustomCommand]) -> Result<PathBuf, String> {
    validate_commands(commands)?;

    let path = commands_path().ok_or_else(|| String::from("Could not locate commands folder."))?;
    let parent = path
        .parent()
        .ok_or_else(|| String::from("Could not locate commands folder."))?;

    fs::create_dir_all(parent).map_err(|error| error.to_string())?;

    let text = serde_json::to_string_pretty(commands).map_err(|error| error.to_string())?;
    fs::write(&path, text).map_err(|error| error.to_string())?;

    Ok(path)
}

pub fn validate_commands(commands: &[CustomCommand]) -> Result<(), String> {
    let mut aliases = Vec::new();

    for (index, command) in commands.iter().enumerate() {
        let row = index + 1;
        let alias = command.alias.trim();
        let executable = command.command.trim();

        if alias.is_empty() && executable.is_empty() && command.name.trim().is_empty() {
            continue;
        }

        if alias.is_empty() {
            return Err(format!("Command {row} needs an alias."));
        }

        if executable.is_empty() {
            return Err(format!("Command {row} needs a command."));
        }

        if alias.contains(char::is_whitespace) {
            return Err(format!("Command {row} alias cannot contain spaces."));
        }

        let normalized_alias = alias.to_ascii_lowercase();
        if aliases.contains(&normalized_alias) {
            return Err(format!("Command alias {alias} is duplicated."));
        }
        aliases.push(normalized_alias);
    }

    Ok(())
}

pub fn run_command(command: &ResolvedCommand, settings: &AppSettings) -> Result<(), String> {
    if command.open_in_terminal {
        return run_command_in_terminal(command, settings);
    }

    let mut process = Command::new("/bin/sh");
    process.arg("-lc").arg(&command.command_line);

    if let Some(working_dir) = command.working_dir.as_ref() {
        process.current_dir(working_dir);
    }

    process
        .spawn()
        .map(|_| ())
        .map_err(|error| error.to_string())
}

pub fn commands_path() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(".deeplens").join("commands.json"))
}

fn run_command_in_terminal(
    command: &ResolvedCommand,
    settings: &AppSettings,
) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let terminal_app = settings.terminal_app.trim();
        let terminal_app = if terminal_app.is_empty() {
            "Terminal"
        } else {
            terminal_app
        };
        let shell_line = terminal_shell_line(command);
        let script = if terminal_app.to_ascii_lowercase().contains("iterm") {
            format!(
                r#"tell application "{}"
activate
set newWindow to (create window with default profile)
tell current session of newWindow
write text "{}"
end tell
end tell"#,
                escape_apple_script(terminal_app),
                escape_apple_script(&shell_line)
            )
        } else {
            format!(
                r#"tell application "{}"
activate
do script "{}"
end tell"#,
                escape_apple_script(terminal_app),
                escape_apple_script(&shell_line)
            )
        };

        return Command::new("osascript")
            .arg("-e")
            .arg(script)
            .spawn()
            .map(|_| ())
            .map_err(|error| error.to_string());
    }

    #[cfg(not(target_os = "macos"))]
    {
        let mut process = Command::new("/bin/sh");
        process.arg("-lc").arg(terminal_shell_line(command));

        if let Some(working_dir) = command.working_dir.as_ref() {
            process.current_dir(working_dir);
        }

        process
            .spawn()
            .map(|_| ())
            .map_err(|error| error.to_string())
    }
}

fn terminal_shell_line(command: &ResolvedCommand) -> String {
    if let Some(working_dir) = command.working_dir.as_ref() {
        format!(
            "cd {} && {}",
            shell_quote(&working_dir.to_string_lossy()),
            command.command_line
        )
    } else {
        command.command_line.clone()
    }
}

fn join_command_line(command: &str, args: &str) -> String {
    let command = command.trim();
    let args = args.trim();

    if args.is_empty() {
        command.to_owned()
    } else {
        format!("{command} {args}")
    }
}

fn expand_home(path: &str) -> Option<PathBuf> {
    if path.is_empty() {
        return None;
    }

    if path == "~" {
        return std::env::var_os("HOME").map(PathBuf::from);
    }

    if let Some(rest) = path.strip_prefix("~/") {
        return std::env::var_os("HOME").map(|home| Path::new(&home).join(rest));
    }

    Some(PathBuf::from(path))
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn escape_apple_script(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::{parse_command_query, CommandBook, CustomCommand};
    use crate::settings::AppSettings;

    #[test]
    fn parses_command_queries() {
        assert_eq!(
            parse_command_query("cmd deploy"),
            Some(String::from("deploy"))
        );
        assert_eq!(
            parse_command_query("commands deploy"),
            Some(String::from("deploy"))
        );
        assert_eq!(parse_command_query("cmd"), Some(String::new()));
        assert_eq!(parse_command_query("cm deploy"), None);
    }

    #[test]
    fn resolves_direct_alias_with_query_template() {
        let book = CommandBook {
            commands: vec![CustomCommand {
                name: String::from("Deploy"),
                alias: String::from("deploy"),
                command: String::from("deploy"),
                args_template: String::from("--service {query}"),
                working_dir: String::from("~/code/app"),
                open_in_terminal: true,
            }],
        };

        let command = book
            .direct_match("deploy api", &AppSettings::default())
            .unwrap();
        assert_eq!(command.name, "Deploy");
        assert_eq!(command.command_line, "deploy --service api");
        assert!(command.working_dir.is_some());
        assert!(command.open_in_terminal);
    }

    #[test]
    fn searches_commands_by_alias_name_and_command() {
        let book = CommandBook {
            commands: vec![
                CustomCommand {
                    name: String::from("Open repo"),
                    alias: String::from("repo"),
                    command: String::from("code"),
                    ..CustomCommand::default()
                },
                CustomCommand {
                    name: String::from("Deploy API"),
                    alias: String::from("deploy"),
                    command: String::from("deploy"),
                    ..CustomCommand::default()
                },
            ],
        };

        let matches = book.search("dep", &AppSettings::default());
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].alias, "deploy");
    }

    #[test]
    fn deserializes_array_command_file_shape() {
        let text = r#"[{"name":"Open","alias":"open","command":"code"}]"#;
        let commands: Vec<CustomCommand> = serde_json::from_str(text).unwrap();
        let book = CommandBook { commands };

        let matches = book.search("open", &AppSettings::default());
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].command_line, "code");
    }

    #[test]
    fn validates_aliases_and_required_fields() {
        let duplicate = vec![
            CustomCommand {
                alias: String::from("open"),
                command: String::from("code"),
                ..CustomCommand::default()
            },
            CustomCommand {
                alias: String::from("OPEN"),
                command: String::from("open"),
                ..CustomCommand::default()
            },
        ];
        assert!(super::validate_commands(&duplicate).is_err());

        let missing_command = vec![CustomCommand {
            alias: String::from("open"),
            ..CustomCommand::default()
        }];
        assert!(super::validate_commands(&missing_command).is_err());
    }
}
