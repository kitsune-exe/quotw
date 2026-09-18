use clap::Parser;
use dirs::config_dir;
use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchItem {
    pub code: String,
    pub name: String,
    pub group: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioGroup {
    #[serde(rename = "groupName")]
    pub group_name: String,
    pub code: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Portfolio {
    pub groups: Vec<PortfolioGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub watchlist: Vec<WatchItem>,
    pub theme: String,
    pub refresh_interval_secs: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            watchlist: vec![],
            theme: "dark".into(),
            refresh_interval_secs: 60,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemeColors {
    pub fg: Color,
    pub bg: Color,
    pub border: Color,
    pub selected: Color,
    pub up: Color,
    pub down: Color,
    pub header_fg: Color,
    pub header_bg: Color,
}

impl ThemeColors {
    pub fn default_theme() -> Self {
        Self {
            fg: Color::White,
            bg: Color::Black,
            border: Color::Gray,
            selected: Color::Yellow,
            up: Color::Red,
            down: Color::Green,
            header_fg: Color::Black,
            header_bg: Color::Cyan,
        }
    }

    pub fn dark_theme() -> Self {
        Self {
            fg: Color::Rgb(220, 220, 220),
            bg: Color::Rgb(30, 30, 30),
            border: Color::Rgb(80, 80, 80),
            selected: Color::Rgb(255, 215, 0),
            up: Color::Rgb(255, 85, 85),
            down: Color::Rgb(85, 255, 85),
            header_fg: Color::Rgb(30, 30, 30),
            header_bg: Color::Rgb(0, 180, 180),
        }
    }

    pub fn high_contrast_theme() -> Self {
        Self {
            fg: Color::Yellow,
            bg: Color::Black,
            border: Color::White,
            selected: Color::Black,
            up: Color::Red,
            down: Color::Green,
            header_fg: Color::Black,
            header_bg: Color::Yellow,
        }
    }

    pub fn from_name(name: &str) -> Self {
        match name {
            "default" => Self::default_theme(),
            "dark" => Self::dark_theme(),
            "high_contrast" => Self::high_contrast_theme(),
            _ => Self::dark_theme(),
        }
    }
}

#[derive(Parser, Debug)]
#[command(name = "ratatui-quote", version, about = "Taiwan Stock Quote TUI")]
struct CliArgs {
    #[arg(help = "Stock codes to watch (overrides config)")]
    codes: Vec<String>,
}

pub fn load_or_build() -> color_eyre::Result<(Config, ThemeColors)> {
    let cli = CliArgs::parse();

    let config_path = config_file_path()?;
    let mut config = if config_path.exists() {
        let content = fs::read_to_string(&config_path)?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Config::default()
    };

    if !cli.codes.is_empty() {
        config.watchlist = cli
            .codes
            .into_iter()
            .map(|c| WatchItem {
                code: c.clone(),
                name: c,
                group: "CLI".into(),
            })
            .collect();
    }

    let theme = ThemeColors::from_name(&config.theme);
    Ok((config, theme))
}

fn config_file_path() -> color_eyre::Result<PathBuf> {
    let mut path = config_dir().ok_or_else(|| color_eyre::eyre::eyre!("No config dir"))?;
    path.push("ratatui-quote");
    fs::create_dir_all(&path)?;
    path.push("config.json");
    Ok(path)
}

fn portfolio_file_path() -> color_eyre::Result<PathBuf> {
    let mut path = config_dir().ok_or_else(|| color_eyre::eyre::eyre!("No config dir"))?;
    path.push("ratatui-quote");
    fs::create_dir_all(&path)?;
    path.push("portfolio.json");
    Ok(path)
}

pub fn load_portfolio() -> color_eyre::Result<Option<Portfolio>> {
    // First check config directory
    let config_path = portfolio_file_path()?;
    if config_path.exists() {
        let content = fs::read_to_string(&config_path)?;
        let portfolio: Portfolio = serde_json::from_str(&content)?;
        return Ok(Some(portfolio));
    }

    // Fallback to current directory
    let local_path = PathBuf::from("portfolio.json");
    if local_path.exists() {
        let content = fs::read_to_string(&local_path)?;
        let portfolio: Portfolio = serde_json::from_str(&content)?;
        return Ok(Some(portfolio));
    }

    Ok(None)
}

pub fn save_config(config: &Config) -> color_eyre::Result<()> {
    let path = config_file_path()?;
    let content = serde_json::to_string_pretty(config)?;
    fs::write(path, content)?;
    Ok(())
}