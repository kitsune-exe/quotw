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
    pub prev_close: f64,
    pub limit_up: f64,
    pub limit_down: f64,
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
            prev_close: f64::NAN,
            limit_up: f64::NAN,
            limit_down: f64::NAN,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.price.is_finite()
    }

    /// 是否為漲停（目前價格等於交易所提供的漲停價）
    pub fn is_limit_up(&self) -> bool {
        if !self.is_valid() || !self.limit_up.is_finite() {
            return false;
        }
        (self.price - self.limit_up).abs() < f64::EPSILON
    }

    /// 是否為跌停（目前價格等於交易所提供的跌停價）
    pub fn is_limit_down(&self) -> bool {
        if !self.is_valid() || !self.limit_down.is_finite() {
            return false;
        }
        (self.price - self.limit_down).abs() < f64::EPSILON
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
        // Volume is already in 張 from API
        let v = self.volume as f64;
        if v >= 1_000_000.0 {
            format!("{:.1}M", v / 1_000_000.0)
        } else if v >= 1_000.0 {
            format!("{:.1}K", v / 1_000.0)
        } else {
            format!("{:.0}", v)
        }
    }
}

#[derive(Debug, Clone)]
pub struct IndexQuote {
    pub name: String,
    pub price: f64,
    pub change: f64,
    pub pct: f64,
    pub time: String,
}

impl IndexQuote {
    pub fn empty() -> Self {
        Self {
            name: "加權指數".to_string(),
            price: f64::NAN,
            change: 0.0,
            pct: 0.0,
            time: "--:--:--".to_string(),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.price.is_finite()
    }

    pub fn price_display(&self) -> String {
        if !self.is_valid() {
            return "--".into();
        }
        format!("{:.2}", self.price)
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
}

#[derive(Debug, Clone)]
pub struct GroupView {
    pub group: String,
    pub items: Vec<Quote>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PopupState {
    None,
    AddStock { group: String, code: String, name: String, field: usize, error: Option<String> },
    AddGroup { name: String },
    EditStock { group: String, index: usize, code: String, name: String, field: usize },
    DeleteConfirm { item_type: DeleteType, name: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeleteType {
    Stock { group: String, index: usize },
    Group { group: String },
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub groups: Vec<GroupView>,
    pub current_tab: usize,
    pub theme_name: String,
    pub last_update: DateTime<Local>,
    pub loading: bool,
    pub refresh_interval_secs: u64,
    pub popup: PopupState,
    pub selected_index: usize,
    pub index_quote: IndexQuote,
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
            popup: PopupState::None,
            selected_index: 0,
            index_quote: IndexQuote::empty(),
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
            self.selected_index = 0;
        }
    }

    pub fn prev_tab(&mut self) {
        if !self.groups.is_empty() {
            self.current_tab = (self.current_tab + self.groups.len() - 1) % self.groups.len();
            self.selected_index = 0;
        }
    }

    pub fn select_next(&mut self) {
        if let Some(group) = self.current_group() {
            if !group.items.is_empty() {
                self.selected_index = (self.selected_index + 1) % group.items.len();
            }
        }
    }

    pub fn select_prev(&mut self) {
        if let Some(group) = self.current_group() {
            if !group.items.is_empty() {
                self.selected_index = (self.selected_index + group.items.len() - 1) % group.items.len();
            }
        }
    }

    /// 確保 selected_index 在當前群組的有效範圍內
    pub fn clamp_selected_index(&mut self) {
        if let Some(group) = self.current_group() {
            if group.items.is_empty() {
                self.selected_index = 0;
            } else if self.selected_index >= group.items.len() {
                self.selected_index = group.items.len() - 1;
            }
        } else {
            self.selected_index = 0;
        }
    }

    pub fn selected_stock(&self) -> Option<&Quote> {
        self.current_group()?.items.get(self.selected_index)
    }

    pub fn selected_stock_mut(&mut self) -> Option<&mut Quote> {
        let idx = self.selected_index;
        self.current_group_mut()?.items.get_mut(idx)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    fn quote(price: f64, change: f64, pct: f64, volume: u64) -> Quote {
        Quote {
            price,
            change,
            pct,
            volume,
            ..Quote::empty("2330", "台積電")
        }
    }

    #[test]
    fn empty_quote_displays_placeholders() {
        let q = Quote::empty("2330", "台積電");
        assert!(!q.is_valid());
        assert_eq!(q.price_display(), "--");
        assert_eq!(q.change_display(), "--");
        assert_eq!(q.pct_display(), "--");
        assert_eq!(q.volume_display(), "--");
    }

    #[test]
    fn price_display_drops_decimals_at_or_above_1000() {
        assert_eq!(quote(999.5, 0.0, 0.0, 0).price_display(), "999.50");
        assert_eq!(quote(1085.0, 0.0, 0.0, 0).price_display(), "1085");
    }

    #[test]
    fn change_and_pct_display_include_sign() {
        let up = quote(100.0, 1.5, 1.52, 0);
        assert_eq!(up.change_display(), "+1.50");
        assert_eq!(up.pct_display(), "+1.52%");

        let down = quote(100.0, -2.0, -1.96, 0);
        assert_eq!(down.change_display(), "-2.00");
        assert_eq!(down.pct_display(), "-1.96%");
    }

    #[test]
    fn volume_display_uses_k_and_m_suffixes() {
        assert_eq!(quote(100.0, 0.0, 0.0, 999).volume_display(), "999");
        assert_eq!(quote(100.0, 0.0, 0.0, 12_345).volume_display(), "12.3K");
        assert_eq!(quote(100.0, 0.0, 0.0, 2_500_000).volume_display(), "2.5M");
    }

    #[test]
    fn limit_up_and_down_detection() {
        let mut q = quote(110.0, 10.0, 10.0, 0);
        q.limit_up = 110.0;
        q.limit_down = 90.0;
        assert!(q.is_limit_up());
        assert!(!q.is_limit_down());

        q.limit_up = f64::NAN;
        assert!(!q.is_limit_up());
    }
}
