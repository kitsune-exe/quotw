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

## 安裝

本專案沒有發布至crates.io，所以需要clone本repo後編譯安裝。

有提供Makefile的腳本，如果有安裝GNU make，可以執行以下命令來自動化一些作業。也可以用cargo原本的指令來做到以下所有動作。

### 編譯並安裝(自動清理中間產物)

    make
or:

    make install

### 解除安裝

    make uninstall

### 編譯(產生debug資訊)

    make build

### 編譯(release)

    make release

### 清除編譯的中間產物

    make clean

### 解釋make選項

    make help

## 快速開始

### 啟動程式

    quotw

啟動程式後會在檢查有無`~/.config/quotw`，若無則創建該資料夾並生成`config.json`以及`portfolio.json`。前者儲存設定值，後者儲存選股資訊。

### 不進入TUI，僅作單次報價(portfolio.json內容)

    quotw -o

or

    quotw --once

### 不進入TUI，僅針對輸入代號進行報價

    quotw -o <code1> <code2> ...

or

    quotw --once <code1> <code2> ...
