mod api;
mod app;
mod config;
mod model;
mod ui;

use color_eyre::Result;
use config::load_or_build;
use crate::config::Config;
use ratatui::init;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let (config, theme, cli) = load_or_build()?;

    if cli.once {
        return run_once(&config, cli.codes).await;
    }

    let mut terminal = init();
    let result = app::run(&mut terminal, config, theme).await;

    ratatui::restore();
    result
}

async fn run_once(config: &Config, codes: Vec<String>) -> Result<()> {
    use crate::api::{fetch_index, fetch_quotes};
    use crate::config::load_portfolio;

    let codes_to_fetch: Vec<String> = if codes.is_empty() {
        // No codes specified, use portfolio
        let portfolio = load_portfolio()?.unwrap_or_default();
        portfolio.groups.iter().flat_map(|g| g.code.clone()).collect()
    } else {
        // Use provided codes
        codes.iter().map(|c| c.to_ascii_uppercase()).collect()
    };

    if codes_to_fetch.is_empty() {
        println!("投資組合為空，請先加入股票代碼或指定股票代碼");
        return Ok(());
    }

    let quotes = fetch_quotes(&codes_to_fetch).await?;
    let index = fetch_index().await.unwrap_or_else(|_| crate::model::IndexQuote::empty());

    // Print index
    if index.is_valid() {
        println!("{} {:.2} {:+.2} ({:+.2}%)", index.name, index.price, index.change, index.pct);
    }

    println!();

    if codes.is_empty() {
        // Portfolio mode - grouped output
        let portfolio = load_portfolio()?.unwrap_or_default();
        for group in &portfolio.groups {
            if group.code.is_empty() {
                continue;
            }
            println!("=== {} ===", group.group_name);
            for code in &group.code {
                if let Some(q) = quotes.iter().find(|q| q.code == *code) {
                    if q.is_valid() {
                        println!("{} {} {:.2} {:+.2} ({:+.2}%) vol:{}", 
                            q.code, q.name, q.price, q.change, q.pct, q.volume);
                    } else {
                        println!("{} {} --", q.code, q.name);
                    }
                }
            }
            println!();
        }
    } else {
        // Specific codes mode - flat output
        for code in &codes_to_fetch {
            if let Some(q) = quotes.iter().find(|q| q.code == *code) {
                if q.is_valid() {
                    println!("{} {} {:.2} {:+.2} ({:+.2}%) vol:{}", 
                        q.code, q.name, q.price, q.change, q.pct, q.volume);
                } else {
                    println!("{} {} --", q.code, q.name);
                }
            } else {
                println!("{} -- (查無資料)", code);
            }
        }
    }

    Ok(())
}