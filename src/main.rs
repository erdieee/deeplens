mod app;
mod app_search;
mod history;
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
    app::run()
}
