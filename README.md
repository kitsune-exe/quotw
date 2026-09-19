# quotw

## 專案簡介

本專案是一個TUI APP，可從台灣證交所取得指定標的的報價，並在交易時間內定期更新。

## 專案技術棧

- **Rust** (Edition 2024)
- **TUI Framework**: [ratatui](https://github.com/ratatui/ratatui) 0.30.2
- **Terminal Handling**: [crossterm](https://github.com/crossterm-rs/crossterm) 0.29.0
- **Async Runtime**: [tokio](https://tokio.rs/) 1.40 (full features)
- **HTTP Client**: [reqwest](https://github.com/seanmonstar/reqwest) 0.12 (json, rustls-tls)
- **Serialization**: [serde](https://serde.rs/) 1.0 / [serde_json](https://github.com/serde-rs/json) 1.0
- **Error Handling**: [color-eyre](https://github.com/yaahc/color-eyre) 0.6.5, [thiserror](https://github.com/dtolnay/thiserror) 2.0
- **Configuration**: [clap](https://github.com/clap-rs/clap) 4.5 (derive), [dirs](https://github.com/dirs-dev/dirs-rs) 5.0
- **Date/Time**: [chrono](https://github.com/chronotope/chrono) 0.4 (serde support)

## 專案路線

現在的進度是: 第一階段

1. 第一階段: 基礎建設
    - 將TUI app框架搭建起來。
    - 嘗試呼叫台灣證交所的API並取得2330的報價(一次性)。

2. 第二階段: 功能填充
    - 讀取自選股的json檔案，並依序取得報價，接著每60秒更新一次。
    - 實作翻頁功能，以便分頁顯示標的。

3. 第三階段: 顯示效果調整
    - 製作色彩主題
    - 使用者可選擇主題，並記憶設定值。可利用json存檔。