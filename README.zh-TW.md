# nl-cli

**繁體中文** | [English](README.md)

**一個 CLI 搞定 Newsleopard EDM 與 Surenotify API — 為開發者與 AI Agent 打造。**

透過單一命令列工具管理電子報活動、交易型訊息、聯絡人、範本與報告。結構化 JSON 輸出、內建限流、Dry-run 安全預覽。

[![CI](https://github.com/Newsleopard/nlm-open-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/Newsleopard/nlm-open-cli/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/Newsleopard/nlm-open-cli)](https://github.com/Newsleopard/nlm-open-cli/releases)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![Rust: 1.75+](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)

---

## 目錄

- [前置需求](#前置需求)
- [安裝](#安裝)
- [快速開始](#快速開始)
- [為什麼選擇 nl？](#為什麼選擇-nl)
- [指令總覽](#指令總覽)
- [認證設定](#認證設定)
- [全域參數](#全域參數)
- [程式化使用](#程式化使用)
- [變數語法](#變數語法)
- [Rate Limits](#rate-limits)
- [文件](#文件)
- [貢獻](#貢獻)
- [授權](#授權)

## 前置需求

- [Rust](https://rustup.rs/) 1.75+（從原始碼編譯時需要）或直接下載預編譯執行檔
- 一個 [Newsleopard（電子豹）](https://www.newsleopard.com/) 帳戶，並取得 EDM 或 Surenotify API Key

## 安裝

### npm

```bash
npm install -g @newsleopard/nl-cli
```

### Cargo

```bash
cargo install nl-cli
```

### GitHub Releases

從 [Releases](https://github.com/Newsleopard/nlm-open-cli/releases) 頁面下載對應平台的預編譯執行檔。

支援平台：Linux (x86_64, arm64)、macOS (x86_64, arm64)、Windows (x86_64)。

### 從原始碼編譯

```bash
git clone https://github.com/Newsleopard/nlm-open-cli.git
cd nlm-open-cli
cargo build --release
# 執行檔位於 target/release/nl
```

## 快速開始

```bash
# 1. 設定 API Key
nl config init

# 2. 查詢帳戶餘額
nl edm account balance

# 3. 列出聯絡人群組
nl edm contacts list-groups
```

## 為什麼選擇 nl？

### 之前：直接呼叫 API

```bash
curl -s -H "x-api-key: $KEY" \
  "https://api.newsleopard.com/v1/contacts/groups?page=1&per_page=20" \
  | jq '.data'
```

### 之後：一行指令

```bash
nl edm contacts list-groups --format table
```

**核心優勢：**

- **31 個 API endpoints** 封裝為直覺式子指令 — 不需手動拼接 URL 與 Header
- **結構化輸出** — JSON、Table、YAML、CSV 四種格式；piped 時 JSON 自動切換為 compact
- **內建限流** — Token bucket 自動遵守 API 限制（EDM 2 req/s、Report 匯出 1 req/10s）
- **智慧重試** — 遇到 429 與 5xx 錯誤時自動 exponential backoff
- **Dry-run 模式** — 預覽 HTTP 請求而不實際送出（`--dry-run`）
- **高階編排指令** — 多步驟工作流程如 `campaign-send` 與 `import-and-wait`
- **多 Profile 設定** — 透過 `--profile` 在 staging 與 production 間切換

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
| `nl config` | 設定檔管理 | -- |
| `nl helper` | 高階編排指令 | -- |

## 認證設定

| 使用情境 | 方式 | 設定方法 |
|----------|------|----------|
| 互動式使用（本地開發） | 設定檔 | `nl config init` |
| CI/CD 或容器環境 | 環境變數 | `export NL_EDM_API_KEY="..."` |
| 多環境切換 | Profile | `nl config set edm_api_key "..." --profile staging` |

**認證優先順序：** 環境變數 > CLI flag > Profile 設定 > `[default]` section。

### 設定檔

位於 `~/.config/nl/config.toml`：

```toml
[default]
edm_api_key = "your-edm-key"
sn_api_key = "your-sn-key"
default_format = "json"

[staging]
edm_api_key = "staging-key"
sn_api_key = "staging-sn-key"
```

### 環境變數

| 變數 | 用途 |
|------|------|
| `NL_EDM_API_KEY` | EDM API Key |
| `NL_SN_API_KEY` | Surenotify API Key |
| `NL_PROFILE` | 啟用的 Profile 名稱（預設：`default`） |
| `NL_FORMAT` | 預設輸出格式（預設：`json`） |

## 全域參數

```
--format <json|table|yaml|csv>   輸出格式（預設: json，env: NL_FORMAT）
--profile <NAME>                 設定檔 Profile（預設: default，env: NL_PROFILE）
--dry-run                        預覽請求而不執行
--page-all                       串流分頁結果為 NDJSON（每行一個 JSON 物件）
-v, --verbose                    顯示請求/回應詳情（可疊加: -vv）
-q, --quiet                      只顯示錯誤輸出
```

> **Piping 行為：** `--format json` 在終端輸出 pretty-printed JSON，piped 時自動切換為 compact（單行）JSON。`--page-all` 以 NDJSON 串流輸出，適合 `jq` 逐行處理。

## 程式化使用

本 CLI 專為腳本與 AI Agent 整合設計。所有輸出皆為機器可解析格式，搭配結構化 exit codes 與 JSON 錯誤輸出至 stderr。

### Exit Codes

| 代碼 | 意義 | 觸發條件 |
|------|------|----------|
| 0 | 成功 | 正常回應、dry-run 預覽、204 No Content |
| 1 | API 錯誤 | HTTP 4xx/5xx（Newsleopard API 回傳） |
| 2 | 驗證錯誤 | CLI 參數驗證失敗 |
| 3 | 認證/設定錯誤 | API Key 無效、設定檔缺失或損毀 |
| 4 | 網路/限流錯誤 | 連線失敗、每日額度用盡 |
| 5 | I/O 錯誤 | 檔案讀寫失敗 |

### 錯誤輸出格式

所有錯誤皆以 JSON 輸出至 **stderr**，`type` 值為 `api`、`validation`、`auth`、`config`、`network`、`rate_limit`、`io` 之一：

```json
{
  "error": {
    "type": "api",
    "message": "API error 400: [40012] Insufficient balance",
    "exit_code": 1
  }
}
```

### 腳本範例

```bash
# 取得活動開信率
result=$(nl edm report metrics --campaign-sn "$SN" -q 2>/tmp/nl_err.json)
if [ $? -eq 0 ]; then
  echo "$result" | jq '.open_rate'
else
  echo "Failed: $(jq -r '.error.type' /tmp/nl_err.json)" >&2
fi
```

```bash
# 串流所有群組，篩選開信率 > 30%
nl edm contacts list-groups --page-all -q | jq 'select(.opened_rate > 0.3)'
```

```bash
# Dry-run 預覽活動送出請求
nl edm campaign submit --name "三月電子報" --dry-run
```

## 變數語法

EDM 與 Surenotify 使用**不同的變數語法**，混用會導致變數替換靜默失敗。

| API | 語法 | 範例 | 適用指令 |
|-----|------|------|----------|
| EDM | `${FIELD_NAME}` | `${NAME}`、`${ORDER_ID}` | `nl edm campaign`、`nl edm ab-test`、`nl edm automation` |
| Surenotify | `{{variable_name}}` | `{{name}}`、`{{order_id}}` | `nl sn email`、`nl sn sms` |

> CLI 會偵測並警告跨用情況（例如 EDM 內容中出現 `{{...}}`）。

## Rate Limits

內建 token bucket 限流器，自動遵守以下限制：

| 限制 | 值 | 影響指令 |
|------|------|----------|
| EDM 一般請求 | 2 req/s | 所有 `nl edm` 指令 |
| Report 匯出 | 1 req/10s | `nl edm report export` |
| SN 收件人上限 | 100 人/次 | `nl sn email send`、`nl sn sms send` |

HTTP 429 與 5xx 錯誤會自動以 exponential backoff 重試（500ms 起始、30s 上限、120s 總逾時）。

## 文件

- [產品需求文件 (PRD)](docs/PRD.md) — 完整 API 覆蓋範圍、使用場景、驗收標準
- [技術架構設計](docs/Architecture.md) — 模組結構、設計模式、依賴
- [CLI 使用手冊](docs/CLI-USER-GUIDE.md) — 完整指令樹與範例
- [Newsleopard API Agent Skill](https://github.com/Newsleopard/nlm-open-skills) — AI Agent 技能，協助 AI 程式助手產生 Newsleopard API 整合程式碼（支援 Claude Code、GitHub Copilot、Cursor）

## 貢獻

歡迎貢獻！請閱讀 [CONTRIBUTING.md](CONTRIBUTING.md) 了解開發環境設定、程式碼風格與 PR 流程。

本專案遵循 [Contributor Covenant 行為準則](CODE_OF_CONDUCT.md)。

## 授權

本專案採用以下任一授權條款：

- [MIT 授權](LICENSE-MIT)
- [Apache 授權 2.0 版](LICENSE-APACHE)

由您自行選擇。
