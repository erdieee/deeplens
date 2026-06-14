mod app;
mod app_search;
mod calculator;
mod clipboard_history;
mod custom_commands;
mod doctor;
mod history;
mod pins;
mod query;
mod ranking;
mod search;
mod settings;
mod shortcuts;
mod tools;
mod types;
mod web_search;

fn main() -> iced::Result {
    tools::install_runtime_path();

    if doctor::requested() {
        std::process::exit(doctor::run());
    }

    app::run()
}
