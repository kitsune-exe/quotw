# quotw

## 專案簡介

本專案是一個TUI APP，可從台灣證交所(上市、上櫃)取得自選股及加權指數的報價，並定期自動更新(預設每60秒)。

主要功能:

- 以分組(分頁)管理自選股，可在TUI內新增、編輯、刪除股票及分組，變更會寫回`portfolio.json`。
- 定期自動更新報價，也可手動立即重抓。
- 內建三種色彩主題(catppuccin_mocha、monokai_classic、tokyo_night)，切換後會記憶至`config.json`。
- 單次報價模式(`--once`)，不進入TUI，直接將報價輸出至終端機。

## 專案技術棧

- **Rust** (Edition 2024)
- **TUI Framework**: [ratatui](https://github.com/ratatui/ratatui) 0.30.2
- **Terminal Handling**: [crossterm](https://github.com/crossterm-rs/crossterm) 0.29.0
- **Async Runtime**: [tokio](https://tokio.rs/) 1.53 (full features)
- **HTTP Client**: [reqwest](https://github.com/seanmonstar/reqwest) 0.13 (json, rustls)
- **Serialization**: [serde](https://serde.rs/) 1.0 / [serde_json](https://github.com/serde-rs/json) 1.0
- **Error Handling**: [color-eyre](https://github.com/yaahc/color-eyre) 0.6.5, [thiserror](https://github.com/dtolnay/thiserror) 2.0
- **Configuration**: [clap](https://github.com/clap-rs/clap) 4.6 (derive), [dirs](https://github.com/dirs-dev/dirs-rs) 7.0
- **Date/Time**: [chrono](https://github.com/chronotope/chrono) 0.4 (serde support)

## 專案路線

現在的進度是: 第三階段已完成

1. 第一階段: 基礎建設 ✅
    - 將TUI app框架搭建起來。
    - 嘗試呼叫台灣證交所的API並取得2330的報價(一次性)。

2. 第二階段: 功能填充 ✅
    - 讀取自選股的json檔案，並依序取得報價，接著每60秒更新一次。
    - 實作翻頁功能，以便分頁顯示標的。

3. 第三階段: 顯示效果調整 ✅
    - 製作色彩主題
    - 使用者可選擇主題，並記憶設定值。可利用json存檔。

額外完成:

- 在TUI內新增/編輯/刪除股票及分組。
- 顯示加權指數。
- 單次報價模式(`--once`)。
- `Quote`顯示相關函式的單元測試。

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

## 操作說明

| 按鍵 | 功能 |
| --- | --- |
| `←` / `→`、`Tab` / `Shift+Tab` | 切換分組(分頁) |
| `↑` / `↓` | 選取股票 |
| `a` | 在目前分組新增股票(輸入代號後會先查詢驗證) |
| `A` | 新增分組 |
| `e` | 編輯選取的股票 |
| `d` | 刪除選取的股票；分組為空時則刪除該分組 |
| `r` | 立即重抓報價 |
| `t` | 切換色彩主題 |
| `q` / `Esc` | 離開 |

彈出視窗中，`Enter`確認、`Esc`取消；刪除確認視窗以`y`/`n`回應。

## 設定檔

設定檔位於`~/.config/quotw/`。

### config.json

```json
{
  "theme": "catppuccin_mocha",
  "refresh_interval_secs": 60
}
```

- `theme`: `catppuccin_mocha`、`monokai_classic`或`tokyo_night`。
- `refresh_interval_secs`: 自動更新的間隔秒數。

### portfolio.json

```json
{
  "groups": [
    {
      "groupName": "半導體",
      "code": ["2330", "2454"]
    }
  ]
}
```

## 測試

    cargo test
