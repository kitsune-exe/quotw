use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    pub code: String,
    pub name: String,
    pub price: f64,
    pub change: f64,
    pub pct: f64,
    pub volume: u64,
    pub time: String,
}

impl Quote {
    pub fn empty(code: &str, name: &str) -> Self {
        Self {
            code: code.into(),
            name: name.into(),
            price: f64::NAN,
            change: 0.0,
            pct: 0.0,
            volume: 0,
            time: "--:--:--".into(),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.price.is_finite()
    }

    pub fn price_display(&self) -> String {
        if !self.is_valid() {
            return "--".into();
        }
        if self.price >= 1000.0 {
            format!("{:.0}", self.price)
        } else {
            format!("{:.2}", self.price)
        }
    }

    pub fn change_display(&self) -> String {
        if !self.is_valid() {
            return "--".into();
        }
        let sign = if self.change >= 0.0 { "+" } else { "" };
        format!("{}{:.2}", sign, self.change)
    }

    pub fn pct_display(&self) -> String {
        if !self.is_valid() {
            return "--".into();
        }
        let sign = if self.pct >= 0.0 { "+" } else { "" };
        format!("{}{:.2}%", sign, self.pct)
    }

    pub fn volume_display(&self) -> String {
        if !self.is_valid() {
            return "--".into();
        }
        let v = self.volume as f64;
        if v >= 1_000_000_000.0 {
            format!("{:.1}B", v / 1_000_000_000.0)
        } else if v >= 1_000_000.0 {
            format!("{:.1}M", v / 1_000_000.0)
        } else if v >= 1_000.0 {
            format!("{:.1}K", v / 1_000.0)
        } else {
            format!("{:.0}", v)
        }
    }

    pub fn change_color(&self) -> &'static str {
        if !self.is_valid() {
            return "none";
        }
        if self.change > 0.0 {
            "up"
        } else if self.change < 0.0 {
            "down"
        } else {
            "none"
        }
    }
}

#[derive(Debug, Clone)]
pub struct GroupView {
    pub group: String,
    pub items: Vec<Quote>,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub groups: Vec<GroupView>,
    pub current_tab: usize,
    pub theme_name: String,
    pub last_update: DateTime<Local>,
    pub loading: bool,
    pub refresh_interval_secs: u64,
}

impl AppState {
    pub fn new(groups: Vec<GroupView>, theme_name: String, refresh_interval_secs: u64) -> Self {
        Self {
            groups,
            current_tab: 0,
            theme_name,
            last_update: Local::now(),
            loading: false,
            refresh_interval_secs,
        }
    }

    pub fn current_group(&self) -> Option<&GroupView> {
        self.groups.get(self.current_tab)
    }

    pub fn current_group_mut(&mut self) -> Option<&mut GroupView> {
        self.groups.get_mut(self.current_tab)
    }

    pub fn next_tab(&mut self) {
        if !self.groups.is_empty() {
            self.current_tab = (self.current_tab + 1) % self.groups.len();
        }
    }

    pub fn prev_tab(&mut self) {
        if !self.groups.is_empty() {
            self.current_tab = (self.current_tab + self.groups.len() - 1) % self.groups.len();
        }
    }
}