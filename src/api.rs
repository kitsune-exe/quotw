use crate::model::{IndexQuote, Quote};
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Deserialize)]
struct TwseResponse {
    #[serde(rename = "msgArray")]
    msg_array: Vec<TwseQuote>,
}

#[derive(Debug, Deserialize)]
struct TwseQuote {
    c: String, // code (e.g., "2330")
    #[serde(default)]
    n: String, // name
    #[serde(default)]
    z: String, // price (often "-")
    #[serde(default)]
    v: String, // volume
    #[serde(default)]
    t: String, // time
    #[serde(default)]
    f: String, // change (漲跌), format: "val1_val2_..."
    #[serde(default)]
    g: String, // change percent (漲跌%), format: "val1_val2_..."
    #[serde(default)]
    y: String, // yesterday close
    #[serde(default)]
    h: String, // high
    #[serde(default)]
    l: String, // low
    #[serde(default)]
    o: String, // open
    #[serde(default, rename = "u")]
    limit_up: String, // 漲停價
    #[serde(default, rename = "w")]
    limit_down: String, // 跌停價
    #[serde(rename = "trade", default)]
    trade_info: Option<TradeInfo>,
}

#[derive(Debug, Deserialize)]
struct TradeInfo {
    z: String, // latest trade price (current price)
}

async fn fetch_from_market(
    client: &Client,
    market: &str,
    codes: &[String],
) -> Result<Vec<Quote>, ApiError> {
    if codes.is_empty() {
        return Ok(vec![]);
    }

    let ex_ch = codes
        .iter()
        .map(|c| format!("{}_{}.tw", market, c))
        .collect::<Vec<_>>()
        .join("|");

    let url = format!(
        "https://mis.twse.com.tw/stock/api/getStockInfo.jsp?ex_ch={}",
        ex_ch
    );

    let resp = client
        .get(&url)
        .header("User-Agent", "Mozilla/5.0 (compatible; quotw)")
        .header("Referer", "https://mis.twse.com.tw/")
        .send()
        .await?;

    let text = resp.text().await?;
    let twse_resp: TwseResponse = serde_json::from_str(&text)?;

    // Build a map for quick lookup
    let mut quote_map: HashMap<String, TwseQuote> = HashMap::new();
    for q in twse_resp.msg_array {
        quote_map.insert(q.c.clone(), q);
    }

    let mut results = Vec::with_capacity(codes.len());
    for code in codes {
        if let Some(q) = quote_map.get(code) {
            let price_str = q.trade_info.as_ref().map(|t| t.z.as_str()).unwrap_or(&q.z);
            let price = price_str.parse::<f64>().unwrap_or(f64::NAN);

            // Validate: must have name and valid price
            if q.n.is_empty() || !price.is_finite() {
                results.push(Quote::empty(code, code));
                continue;
            }

            let prev_close = q.y.parse::<f64>().unwrap_or(f64::NAN);
            let high = q.h.parse::<f64>().unwrap_or(f64::NAN);
            let low = q.l.parse::<f64>().unwrap_or(f64::NAN);
            let limit_up = q.limit_up.parse::<f64>().unwrap_or(f64::NAN);
            let limit_down = q.limit_down.parse::<f64>().unwrap_or(f64::NAN);

            let (change, pct) = if price.is_finite() && prev_close.is_finite() && prev_close != 0.0
            {
                let chg = price - prev_close;
                let pct_chg = (chg / prev_close) * 100.0;
                (chg, pct_chg)
            } else {
                (0.0, 0.0)
            };

            let volume_str = q.v.replace([',', ' '], "");
            // TWSE API returns volume in 張 (1000 shares) for both TSE and OTC
            let volume = volume_str.parse::<u64>().unwrap_or(0);

            results.push(Quote {
                code: code.clone(),
                name: q.n.clone(),
                price,
                change,
                pct,
                volume,
                time: q.t.clone(),
                prev_close,
                high,
                low,
                limit_up,
                limit_down,
            });
        } else {
            // Code not found in this market
            results.push(Quote::empty(code, code));
        }
    }

    Ok(results)
}

pub async fn fetch_quotes(codes: &[String]) -> Result<Vec<Quote>, ApiError> {
    if codes.is_empty() {
        return Ok(vec![]);
    }

    let client = Client::new();

    // First try TSE (上市)
    let tse_results = fetch_from_market(&client, "tse", codes).await?;

    // Find codes that weren't found in TSE (empty quotes)
    let not_found: Vec<String> = tse_results
        .iter()
        .filter(|q| !q.is_valid())
        .map(|q| q.code.clone())
        .collect();

    if not_found.is_empty() {
        // All found in TSE
        return Ok(tse_results);
    }

    // Try OTC (上櫃) for not found codes
    let otc_results = fetch_from_market(&client, "otc", &not_found).await?;

    // Merge results: use OTC results for codes not found in TSE
    let mut otc_map: HashMap<String, Quote> = HashMap::new();
    for q in otc_results {
        otc_map.insert(q.code.clone(), q);
    }

    let mut final_results = Vec::with_capacity(codes.len());
    for q in tse_results {
        if q.is_valid() {
            final_results.push(q);
        } else if let Some(otc_q) = otc_map.get(&q.code) {
            final_results.push(otc_q.clone());
        } else {
            final_results.push(q); // Still empty
        }
    }

    Ok(final_results)
}

pub async fn fetch_index() -> Result<IndexQuote, ApiError> {
    let client = Client::new();
    // TWSE index code is typically "0000" or "t00"
    let codes = ["t00".to_string()];

    let ex_ch = codes
        .iter()
        .map(|c| format!("tse_{}.tw", c))
        .collect::<Vec<_>>()
        .join("|");

    let url = format!(
        "https://mis.twse.com.tw/stock/api/getStockInfo.jsp?ex_ch={}",
        ex_ch
    );

    let resp = client
        .get(&url)
        .header("User-Agent", "Mozilla/5.0 (compatible; quotw)")
        .header("Referer", "https://mis.twse.com.tw/")
        .send()
        .await?;

    let text = resp.text().await?;
    let twse_resp: TwseResponse = serde_json::from_str(&text)?;

    if let Some(q) = twse_resp.msg_array.first() {
        let price_str = q.trade_info.as_ref().map(|t| t.z.as_str()).unwrap_or(&q.z);
        let price = price_str.parse::<f64>().unwrap_or(f64::NAN);

        let prev_close = q.y.parse::<f64>().unwrap_or(f64::NAN);

        let (change, pct) = if price.is_finite() && prev_close.is_finite() && prev_close != 0.0 {
            let chg = price - prev_close;
            let pct_chg = (chg / prev_close) * 100.0;
            (chg, pct_chg)
        } else {
            (0.0, 0.0)
        };

        Ok(IndexQuote {
            name: q.n.clone(),
            price,
            change,
            pct,
            time: q.t.clone(),
        })
    } else {
        Ok(IndexQuote {
            name: "加權指數".to_string(),
            price: f64::NAN,
            change: 0.0,
            pct: 0.0,
            time: "--:--:--".to_string(),
        })
    }
}
