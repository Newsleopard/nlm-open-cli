# nl — NewsLeopard CLI

Rust CLI 工具，封裝 NewsLeopard EDM API 和 SureNotify API，提供完整的電子報行銷與交易型訊息管理能力。

## 特色

- **完整 API 覆蓋** — 31 個 API endpoints（EDM 20 + SureNotify 11）全數支援
- **靜態型別檢查** — 使用 clap derive structs，所有指令與參數在編譯期驗證
- **多格式輸出** — JSON、Table、YAML、CSV 四種輸出格式，piping 友善
- **智慧重試** — 內建 exponential backoff，自動處理 429 rate limit 與 5xx 錯誤
- **Rate Limiting** — Token bucket 限流器，遵守 EDM 2 req/s 與 report export 1 req/10s 限制
- **Dry-run 模式** — 預覽 HTTP 請求而不實際送出，安全除錯
- **Helper 編排指令** — 高階工作流程（campaign-send、import-and-wait、report-download、domain-setup）
- **Profile 管理** — 多環境設定檔，支援 staging / production 切換
- **跨平台** — 支援 Linux (x86_64/arm64)、macOS (x86_64/arm64)、Windows (x86_64)

## 安裝

### Homebrew (macOS / Linux)

```bash
brew install newsleopard/tap/nl
```

### Cargo

```bash
cargo install nl-cli
```

### GitHub Releases

從 [Releases](https://github.com/newsleopard/nl-cli/releases) 頁面下載對應平台的執行檔。

### 從原始碼編譯

```bash
git clone https://github.com/newsleopard/nl-cli.git
cd nl-cli
cargo build --release
# 執行檔位於 target/release/nl
```

## 快速開始

### 1. 設定 API Key

```bash
# 互動式設定
nl config init

# 或手動設定
nl config set edm_api_key "your-edm-api-key"
nl config set sn_api_key "your-surenotify-api-key"

# 也可透過環境變數
export NL_EDM_API_KEY="your-edm-api-key"
export NL_SN_API_KEY="your-surenotify-api-key"
```

### 2. 查詢帳戶餘額

```bash
nl edm account balance
```

```json
{
  "email": 10000,
  "sms": 500
}
```

### 3. 發送交易型 Email

```bash
nl sn email send \
  --subject "訂單確認 {{order_id}}" \
  --from-address "noreply@example.com" \
  --html "<h1>Hi {{name}}</h1><p>訂單 {{order_id}} 已確認</p>" \
  --recipients '[{"name":"Alice","address":"alice@example.com","variables":{"name":"Alice","order_id":"ORD-001"}}]'
```

### 4. 建立聯絡人群組並發送電子報

```bash
# 建立群組
nl edm contacts create-group --name "VIP 客戶"

# 匯入聯絡人（等待匯入完成）
nl helper import-and-wait --list-sn <GROUP_SN> --file contacts.csv

# 一鍵發送（含餘額檢查、驗證、發送、追蹤）
nl helper campaign-send \
  --name "三月電子報" \
  --lists <GROUP_SN> \
  --subject "三月份精選優惠" \
  --from-name "品牌名稱" \
  --from-address "newsletter@example.com" \
  --html-file template.html \
  --wait
```

## 指令總覽

| 指令群組 | 說明 | Endpoint 數量 |
|----------|------|---------------|
| `nl edm contacts` | 聯絡人群組管理（建立、列表、匯入、刪除） | 6 |
| `nl edm campaign` | 電子報活動管理（送出、狀態、暫停、刪除） | 5 |
| `nl edm ab-test` | A/B 測試活動 | 2 |
| `nl edm report` | 活動報告（列表、指標、匯出、下載） | 4 |
| `nl edm template` | 範本管理（列表、取得） | 2 |
| `nl edm automation` | 自動化腳本觸發 | 1 |
| `nl edm account` | 帳戶資訊（餘額查詢） | 1 |
| `nl sn email` | 交易型 Email（發送、事件查詢） | 2 |
| `nl sn sms` | 簡訊（發送、事件查詢、專屬號碼） | 3 |
| `nl sn webhook` | Email Webhook CRUD | 3 |
| `nl sn sms-webhook` | SMS Webhook CRUD | 3 |
| `nl sn domain` | 寄件域名驗證（建立、驗證、移除） | 3 |
| `nl config` | 設定檔管理 | — |
| `nl helper` | 高階編排指令 | — |

## 全域參數

```
--format <json|table|yaml|csv>   輸出格式（預設: json，env: NL_FORMAT）
--profile <NAME>                 設定檔 profile（預設: default，env: NL_PROFILE）
--dry-run                        預覽請求而不執行
-v, --verbose                    顯示請求/回應詳情（可疊加: -vv）
-q, --quiet                      只顯示錯誤輸出
```

## 設定檔

設定檔位於 `~/.config/nl/config.toml`：

```toml
[default]
edm_api_key = "your-edm-key"
sn_api_key = "your-sn-key"
default_format = "json"

[staging]
edm_api_key = "staging-key"
sn_api_key = "staging-sn-key"
```

**優先順序：** 環境變數 > `--profile` 指定的 profile > `[default]` section

## 文件

- [產品需求文件 (PRD)](docs/PRD.md) — 完整 API 覆蓋範圍、使用場景、需求規格
- [技術架構設計](docs/Architecture.md) — 模組結構、設計模式、依賴、CI/CD
- [CLI 使用手冊](docs/CLI-USER-GUIDE.md) — 完整指令樹、範例、設定說明

## 開發

```bash
# 安裝 Rust (1.75+)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 建置
cargo build

# 執行測試
cargo test

# Lint
cargo clippy -- -D warnings

# 格式化
cargo fmt

# 執行特定測試
cargo test edm_api_test
cargo test cli_integration_test
```

## 授權

Copyright (c) NewsLeopard. All rights reserved.
