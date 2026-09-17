use crate::model::Quote;
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
    msgArray: Vec<TwseQuote>,
}

#[derive(Debug, Deserialize)]
struct TwseQuote {
    c: String,      // code (e.g., "2330")
    n: String,      // name
    z: String,      // price (often "-")
    v: String,      // volume
    t: String,      // time
    f: String,      // change (漲跌), format: "val1_val2_..."
    g: String,      // change percent (漲跌%), format: "val1_val2_..."
    y: String,      // yesterday close
    h: String,      // high
    l: String,      // low
    o: String,      // open
    #[serde(rename = "trade")]
    trade_info: Option<TradeInfo>,
}

#[derive(Debug, Deserialize)]
struct TradeInfo {
    z: String,  // latest trade price (current price)
}

pub async fn fetch_quotes(codes: &[String]) -> Result<Vec<Quote>, ApiError> {
    if codes.is_empty() {
        return Ok(vec![]);
    }

    let client = Client::new();
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
        .header("User-Agent", "Mozilla/5.0 (compatible; ratatui-quote)")
        .header("Referer", "https://mis.twse.com.tw/")
        .send()
        .await?;

    let text = resp.text().await?;
    let twse_resp: TwseResponse = serde_json::from_str(&text)?;

    // Build a map for quick lookup using code from 'c' field (e.g., "2330")
    let mut quote_map: HashMap<String, TwseQuote> = HashMap::new();
    for q in twse_resp.msgArray {
        quote_map.insert(q.c.clone(), q);
    }

    let mut results = Vec::with_capacity(codes.len());
    for code in codes {
        // TWSE API returns code in 'c' field without .tw suffix
        if let Some(q) = quote_map.get(code) {
            // Current price is in trade.z, fallback to q.z
            let price_str = q.trade_info.as_ref().map(|t| t.z.as_str()).unwrap_or(&q.z);
            let price = price_str.parse::<f64>().unwrap_or(f64::NAN);

            // f and g have multiple values separated by _, take the first one
            let change = q.f.split('_').next().unwrap_or("0").parse::<f64>().unwrap_or(0.0);
            let pct = q.g.split('_').next().unwrap_or("0").parse::<f64>().unwrap_or(0.0);
            let volume = q.v.parse::<u64>().unwrap_or(0);

            results.push(Quote {
                code: code.clone(),
                name: q.n.clone(),
                price,
                change,
                pct,
                volume,
                time: q.t.clone(),
            });
        } else {
            // Code not found in response
            results.push(Quote::empty(code, code));
        }
    }

    Ok(results)
}