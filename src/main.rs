mod app;
mod query;
mod ranking;
mod search;
mod settings;
mod shortcuts;
mod tools;
mod types;

fn main() -> iced::Result {
    tools::install_runtime_path();
    app::run()
}
