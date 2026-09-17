mod api;
mod app;
mod config;
mod model;
mod ui;

use color_eyre::Result;
use config::load_or_build;
use ratatui::init;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let (config, theme) = load_or_build()?;

    let mut terminal = init();
    let result = app::run(&mut terminal, config, theme).await;

    ratatui::restore();
    result
}