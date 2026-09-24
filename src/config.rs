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
    pub theme: String,
    pub refresh_interval_secs: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: "catppuccin_mocha".into(),
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
    pub fn catppuccin_mocha() -> Self {
        Self {
            fg: Color::Rgb(205, 214, 244),
            bg: Color::Rgb(30, 30, 46),
            border: Color::Rgb(49, 50, 68),
            selected: Color::Rgb(249, 226, 175),
            up: Color::Rgb(243, 139, 168),
            down: Color::Rgb(166, 227, 161),
            header_fg: Color::Rgb(30, 30, 46),
            header_bg: Color::Rgb(137, 180, 250),
        }
    }

    pub fn monokai_classic() -> Self {
        Self {
            fg: Color::Rgb(248, 248, 242),
            bg: Color::Rgb(39, 40, 34),
            border: Color::Rgb(73, 72, 62),
            selected: Color::Rgb(230, 219, 116),
            up: Color::Rgb(249, 38, 114),
            down: Color::Rgb(166, 226, 46),
            header_fg: Color::Rgb(39, 40, 34),
            header_bg: Color::Rgb(102, 217, 239),
        }
    }

    pub fn tokyo_night() -> Self {
        Self {
            fg: Color::Rgb(169, 177, 214),
            bg: Color::Rgb(26, 27, 38),
            border: Color::Rgb(86, 95, 137),
            selected: Color::Rgb(224, 175, 104),
            up: Color::Rgb(247, 118, 142),
            down: Color::Rgb(158, 206, 106),
            header_fg: Color::Rgb(26, 27, 38),
            header_bg: Color::Rgb(122, 162, 247),
        }
    }

    pub fn from_name(name: &str) -> Self {
        match name {
            "catppuccin_mocha" => Self::catppuccin_mocha(),
            "monokai_classic" => Self::monokai_classic(),
            "tokyo_night" => Self::tokyo_night(),
            _ => Self::catppuccin_mocha(),
        }
    }
}

#[derive(Parser, Debug)]
#[command(name = "quotw", version, about = "Taiwan Stock Quote TUI")]
pub struct CliArgs {
    /// 一次性模式：抓取一次報價後輸出並結束
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    pub once: bool,

    /// 股票代碼 (僅在 --once 時使用，不指定則輸出 portfolio 所有群組)
    #[arg(num_args = 0.., value_name = "CODE")]
    pub codes: Vec<String>,
}

pub fn load_or_build() -> color_eyre::Result<(Config, ThemeColors, CliArgs)> {
    let cli = CliArgs::parse();

    let config_path = config_file_path()?;
    let config = if config_path.exists() {
        let content = fs::read_to_string(&config_path)?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        let default_config = Config::default();
        save_config(&default_config)?;
        default_config
    };

    let portfolio_path = portfolio_file_path()?;
    if !portfolio_path.exists() {
        let default_portfolio = Portfolio::default();
        save_portfolio(&default_portfolio)?;
    }

    let theme = ThemeColors::from_name(&config.theme);
    Ok((config, theme, cli))
}

fn config_file_path() -> color_eyre::Result<PathBuf> {
    let mut path = config_dir().ok_or_else(|| color_eyre::eyre::eyre!("No config dir"))?;
    path.push("quotw");
    fs::create_dir_all(&path)?;
    path.push("config.json");
    Ok(path)
}

fn portfolio_file_path() -> color_eyre::Result<PathBuf> {
    let mut path = config_dir().ok_or_else(|| color_eyre::eyre::eyre!("No config dir"))?;
    path.push("quotw");
    fs::create_dir_all(&path)?;
    path.push("portfolio.json");
    Ok(path)
}

pub fn load_portfolio() -> color_eyre::Result<Option<Portfolio>> {
    let config_path = portfolio_file_path()?;
    if config_path.exists() {
        let content = fs::read_to_string(&config_path)?;
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

pub fn save_portfolio(portfolio: &Portfolio) -> color_eyre::Result<()> {
    let path = portfolio_file_path()?;
    let content = serde_json::to_string_pretty(portfolio)?;
    fs::write(path, content)?;
    Ok(())
}
