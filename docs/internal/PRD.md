# 產品需求文件 (PRD) — nl CLI

## 1. 產品概述

### 1.1 產品定義

`nl` 是一套 Rust 命令列工具，封裝 Newsleopard 平台的兩組 API：

- **EDM API** (`api.newsleopard.com`) — 20 個 endpoints，涵蓋批量行銷活動管理
- **Surenotify API** (`mail.surenotifyapi.com`) — 11+ 個 endpoints，涵蓋交易型 Email 與 SMS

### 1.2 目標使用者

| 角色 | 使用場景 |
|------|---------|
| **開發者** | 將 Newsleopard 整合進 CI/CD pipeline、自動化腳本 |
| **DevOps / SRE** | 監控帳戶餘額、自動匯出報告、批次管理聯絡人 |
| **行銷技術人員** | 快速發送測試活動、查詢活動指標、管理 Webhook |
| **系統管理員** | 管理寄件域名驗證、設定多環境 Profile |

### 1.3 產品目標

1. 提供 100% 的 EDM + Surenotify API 覆蓋，無需直接操作 HTTP
2. 降低整合門檻 — 從閱讀 API 文件到完成任務只需一行指令
3. 支援自動化 — 所有輸出可被 `jq`、`awk` 等工具 pipe 處理
4. 跨平台 — 支援 5 個編譯目標，單一靜態執行檔
5. 提供高階編排指令，將多步驟工作流程封裝為一鍵操作

---

## 2. API 覆蓋範圍

### 2.1 EDM API（20 Endpoints）

#### 聯絡人管理（6 Endpoints）

| # | 操作 | HTTP | Endpoint | CLI 指令 |
|---|------|------|----------|---------|
| 1 | 建立群組 | `POST` | `/v1/contacts/lists/insert` | `nl edm contacts create-group` |
| 2 | 查詢群組 | `GET` | `/v1/contacts/lists` | `nl edm contacts list-groups` |
| 3 | 檔案匯入聯絡人 | `POST` | `/v1/contacts/imports/{list_sn}/file` | `nl edm contacts import-file` |
| 4 | 文字匯入聯絡人 | `POST` | `/v1/contacts/imports/{list_sn}/text` | `nl edm contacts import-text` |
| 5 | 查詢匯入狀態 | `GET` | `/v1/contacts/imports/result/{import_sn}` | `nl edm contacts import-status` |
| 6 | 刪除聯絡人 | `DELETE` | `/v1/contacts/{list_sn}` | `nl edm contacts remove` |

#### 活動管理（5 Endpoints）

| # | 操作 | HTTP | Endpoint | CLI 指令 |
|---|------|------|----------|---------|
| 7 | 送出活動 | `POST` | `/v1/campaign/normal/submit` | `nl edm campaign submit` |
| 8 | 單次上傳活動 | `POST` | `/v1/campaign/normal/once` | `nl edm campaign submit-once` |
| 9 | 刪除活動 | `DELETE` | `/v1/campaign/normal` | `nl edm campaign delete` |
| 10 | 暫停活動 | `PATCH` | `/v1/campaign/normal/{campaign_sn}` | `nl edm campaign pause` |
| 11 | 查詢活動狀態 | `GET` | `/v1/campaign/normal/{campaign_sn}` | `nl edm campaign status` |

#### A/B 測試（2 Endpoints）

| # | 操作 | HTTP | Endpoint | CLI 指令 |
|---|------|------|----------|---------|
| 12 | 送出 A/B 測試 | `POST` | `/v1/campaign/testing/submit` | `nl edm ab-test submit` |
| 13 | 單次上傳 A/B 測試 | `POST` | `/v1/campaign/testing/once` | `nl edm ab-test submit-once` |

#### 報告（4 Endpoints）

| # | 操作 | HTTP | Endpoint | CLI 指令 |
|---|------|------|----------|---------|
| 14 | 依日期查詢活動碼 | `GET` | `/v1/report/campaigns` | `nl edm report list` |
| 15 | 活動績效指標 | `POST` | `/v1/report/campaigns/metrics` | `nl edm report metrics` |
| 16 | 匯出詳細報告 | `POST` | `/v1/report/{campaign_sn}/export` | `nl edm report export` |
| 17 | 取得報告下載連結 | `GET` | `/v1/report/{campaign_sn}/link` | `nl edm report download-link` |

#### 範本（2 Endpoints）

| # | 操作 | HTTP | Endpoint | CLI 指令 |
|---|------|------|----------|---------|
| 18 | 列出範本 | `GET` | `/v1/templates` | `nl edm template list` |
| 19 | 取得範本內容 | `GET` | `/v1/templates/{id}` | `nl edm template get` |

#### 自動化（1 Endpoint）

| # | 操作 | HTTP | Endpoint | CLI 指令 |
|---|------|------|----------|---------|
| 20 | 觸發/停止自動化 | `POST` | `/v1/automation/event` | `nl edm automation trigger` |

#### 帳戶（1 Endpoint）

| # | 操作 | HTTP | Endpoint | CLI 指令 |
|---|------|------|----------|---------|
| — | 查詢餘額 | `GET` | `/v1/balance` | `nl edm account balance` |

### 2.2 Surenotify API（11+ Endpoints）

#### Email（2 Endpoints）

| # | 操作 | HTTP | Endpoint | CLI 指令 |
|---|------|------|----------|---------|
| 1 | 發送 Email | `POST` | `/v1/messages` | `nl sn email send` |
| 2 | 查詢 Email 事件 | `GET` | `/v1/events` | `nl sn email events` |

#### SMS（3 Endpoints）

| # | 操作 | HTTP | Endpoint | CLI 指令 |
|---|------|------|----------|---------|
| 3 | 發送簡訊 | `POST` | `/v1/sms/messages` | `nl sn sms send` |
| 4 | 查詢簡訊事件 | `GET` | `/v1/sms/events` | `nl sn sms events` |
| 5 | 查詢專屬號碼 | `GET` | `/v1/sms/exclusive-number` | `nl sn sms exclusive-number` |

#### Email Webhook（3 Endpoints）

| # | 操作 | HTTP | Endpoint | CLI 指令 |
|---|------|------|----------|---------|
| 6 | 建立/更新 Webhook | `POST` | `/v1/webhooks` | `nl sn webhook create` |
| 7 | 列出 Webhook | `GET` | `/v1/webhooks` | `nl sn webhook list` |
| 8 | 刪除 Webhook | `DELETE` | `/v1/webhooks` | `nl sn webhook delete` |

#### SMS Webhook（3 Endpoints）

| # | 操作 | HTTP | Endpoint | CLI 指令 |
|---|------|------|----------|---------|
| 9 | 建立/更新 SMS Webhook | `POST` | `/v1/sms/webhooks` | `nl sn sms-webhook create` |
| 10 | 列出 SMS Webhook | `GET` | `/v1/sms/webhooks` | `nl sn sms-webhook list` |
| 11 | 刪除 SMS Webhook | `DELETE` | `/v1/sms/webhooks` | `nl sn sms-webhook delete` |

#### 寄件域名驗證（3 Endpoints）

| # | 操作 | HTTP | Endpoint | CLI 指令 |
|---|------|------|----------|---------|
| 12 | 建立域名驗證記錄 | `POST` | `/v1/domains/{domain}` | `nl sn domain create` |
| 13 | 驗證 DNS 記錄 | `PUT` | `/v1/domains/{domain}` | `nl sn domain verify` |
| 14 | 移除域名驗證 | `DELETE` | `/v1/domains/{domain}` | `nl sn domain remove` |

---

## 3. 使用場景

### 3.1 自動化行銷工作流程

**場景：** 每週自動發送電子報

```bash
#!/bin/bash
# weekly-newsletter.sh — 由 cron 排程執行

# 1. 檢查餘額是否足夠
BALANCE=$(nl edm account balance --format json -q | jq '.email')
if [ "$BALANCE" -lt 1000 ]; then
  echo "餘額不足: $BALANCE" >&2
  exit 1
fi

# 2. 發送活動
nl edm campaign submit \
  --name "Weekly Newsletter $(date +%Y-%m-%d)" \
  --lists "$LIST_SN" \
  --subject "本週精選" \
  --from-name "品牌電子報" \
  --from-address "newsletter@example.com" \
  --html-file ./templates/weekly.html \
  --schedule immediate \
  --ga --utm-campaign "weekly-$(date +%Y%m%d)"
```

### 3.2 CI/CD 整合 — 部署通知

**場景：** 部署完成後自動通知團隊

```bash
# 在 CI pipeline 中
nl sn email send \
  --subject "部署完成: {{service}} v{{version}}" \
  --from-address "deploy@example.com" \
  --html-file ./templates/deploy-notification.html \
  --recipients-file ./team-recipients.json
```

### 3.3 報告匯出自動化

**場景：** 每日自動匯出前一天的活動報告

```bash
# 取得昨天的活動清單
YESTERDAY=$(date -v-1d +%Y-%m-%dT00:00:00.00Z)
TODAY=$(date +%Y-%m-%dT00:00:00.00Z)

CAMPAIGNS=$(nl edm report list --start-date "$YESTERDAY" --end-date "$TODAY" -q)

# 逐一匯出並下載
echo "$CAMPAIGNS" | jq -r '.[].sn' | while read SN; do
  nl helper report-download --sn "$SN" --output "./reports/${SN}.csv"
done
```

### 3.4 域名驗證設定

**場景：** 設定新的寄件域名

```bash
# 一鍵設定（建立 → 顯示 DNS 記錄 → 等待 → 驗證）
nl helper domain-setup --domain mail.example.com --auto-verify-after 300

# 或手動分步
nl sn domain create --domain mail.example.com  # 取得 DNS 記錄
# ... 設定 DNS ...
nl sn domain verify --domain mail.example.com  # 驗證
```

### 3.5 聯絡人批次管理

**場景：** 匯入大量聯絡人並等待完成

```bash
# 匯入並等待（含 progress bar）
nl helper import-and-wait \
  --list-sn "$GROUP_SN" \
  --file customers-export.csv \
  --timeout 600

# 結果
# ✓ 匯入完成: 15,230 筆成功, 42 筆重複, 3 筆失敗
# 失敗明細已下載至: import-errors.csv
```

### 3.6 A/B 測試

**場景：** 測試不同主旨行的開信率

```bash
nl edm ab-test submit \
  --name "三月促銷 A/B 測試" \
  --lists "$LIST_SN" \
  --test-on subject \
  --proportion 20 \
  --test-duration 4 \
  --test-unit hours \
  --subject-a "限時優惠 — 全站 8 折" \
  --subject-b "VIP 專屬 — 獨享 8 折優惠" \
  --from-name "品牌名稱" \
  --from-address "promo@example.com" \
  --html-file promo-template.html
```

### 3.7 Webhook 管理

**場景：** 設定完整的事件追蹤 pipeline

```bash
# 設定所有 Email 事件 Webhook
for EVENT in delivery open click bounce complaint; do
  nl sn webhook create --event-type "$EVENT" --url "https://api.example.com/webhooks/email/$EVENT"
done

# 確認設定
nl sn webhook list --format table
```

### 3.8 跨環境切換

**場景：** 在 staging 和 production 之間切換

```bash
# 查看可用 Profile
nl config profile list

# 使用 staging Profile
nl edm account balance --profile staging

# 或透過環境變數
NL_PROFILE=staging nl edm account balance
```

---

## 4. 需求規格

### 4.1 功能需求

#### FR-01: 完整 API 覆蓋

- 所有 20 個 EDM API endpoints 必須有對應的 CLI 指令
- 所有 11+ 個 Surenotify API endpoints 必須有對應的 CLI 指令
- 每個 API endpoint 的所有 required 和 optional 參數必須可透過 CLI flags 傳入

#### FR-02: 認證與設定

- 支援 `x-api-key` header 認證（EDM 和 Surenotify 各自獨立的 API Key）
- 設定檔使用 TOML 格式，位於 `~/.config/nl/config.toml`
- 支援多 Profile（default、staging、production 等）
- 環境變數覆蓋設定檔值（`NL_EDM_API_KEY`、`NL_SN_API_KEY`、`NL_PROFILE`、`NL_FORMAT`）
- `nl config init` 提供互動式設定流程

#### FR-03: 輸出格式

- 支援 4 種輸出格式：JSON（預設）、Table、YAML、CSV
- JSON：piped stdout 時自動切換為 compact 格式；`--page-all` 使用 NDJSON
- Table：自動扁平化巢狀 JSON，根據 terminal 寬度截斷
- YAML：分頁時使用 `---` 分隔
- CSV：分頁時只在第一頁輸出 header

#### FR-04: 錯誤處理

- 定義 6 個 exit codes（0-5），對應不同錯誤類型
- 所有錯誤輸出 JSON 格式到 stderr
- API 錯誤包含 HTTP status code、Newsleopard error code、錯誤訊息
- 支援所有 EDM error codes（40001-40020）的解析

#### FR-05: Rate Limiting

- EDM API：Token bucket 限制 2 requests/second
- Report export：獨立限流器 1 request/10 seconds
- 超過限制時自動等待，不丟棄請求

#### FR-06: 重試機制

- 對 HTTP 429（Too Many Requests）和 5xx 錯誤自動重試
- Exponential backoff：初始 500ms，最大 30s，總超時 120s
- 可透過 `--verbose` 觀察重試過程

#### FR-07: Dry-run 模式

- `--dry-run` flag 在所有指令可用
- 輸出完整的 HTTP 請求預覽（method、URL、headers、body）到 stderr
- Dry-run 不消耗任何 API quota

#### FR-08: Helper 編排指令

| Helper | 功能 | 編排步驟 |
|--------|------|---------|
| `campaign-send` | 一鍵發送活動 | 餘額檢查 → 驗證名單 → 送出 → （選）輪詢狀態 → 顯示指標 |
| `import-and-wait` | 匯入並等待完成 | 取得上傳 URL → PUT 檔案 → 輪詢匯入狀態（含 progress bar） |
| `report-download` | 匯出並下載報告 | 匯出 → 輪詢下載連結 → 下載檔案 |
| `domain-setup` | 域名驗證設定 | 建立 DNS 記錄 → 顯示 table → （選）等待 → 驗證 |

#### FR-09: 分頁

- `list-groups` 等列表指令支援 `--page` 和 `--size` 參數
- `--page-all` 自動遍歷所有頁面，以 NDJSON 格式串流輸出

#### FR-10: 檔案輸入

- `--html-file` 從檔案讀取 HTML 內容（替代 `--html` 內聯字串）
- `--recipients-file` 從 JSON 檔案讀取收件者列表（替代 `--recipients` 內聯 JSON）
- `--csv-file` 從檔案讀取 CSV 文字內容（替代 `--csv-text`）

### 4.2 非功能需求

#### NFR-01: 效能

- 單一指令的 CLI 解析到 HTTP 請求送出 < 50ms（不含網路延遲）
- 靜態連結的單一執行檔，啟動時間 < 10ms

#### NFR-02: 跨平台

- 編譯目標：Linux x86_64、Linux arm64、macOS x86_64、macOS arm64、Windows x86_64
- 使用 rustls（不依賴 OpenSSL）確保跨平台 TLS
- Release build 使用 thin LTO + strip 減少執行檔大小

#### NFR-03: 安全性

- API Key 存儲在使用者家目錄的設定檔，檔案權限 600
- `nl config list` 顯示設定時遮蔽 API Key 值（`****...`）
- 永不將 API Key 輸出到 stdout 或 log

#### NFR-04: 可測試性

- 所有 HTTP 互動透過 `wiremock` mock 測試
- CLI 指令透過 `assert_cmd` 做端對端測試
- Formatter 輸出透過 `insta` 做 snapshot 測試
- 測試涵蓋：serialization、validation、config 載入、error handling

#### NFR-05: 可觀測性

- 支援 `tracing` + `tracing-subscriber` 結構化 log
- `-v` 顯示 HTTP 請求/回應摘要
- `-vv` 顯示完整 HTTP headers 和 body
- `RUST_LOG` 環境變數控制 log level

---

## 5. 變數語法差異

兩套 API 使用不同的變數語法，CLI 必須在驗證層檢查避免混用：

| API | 變數語法 | 範例 |
|-----|---------|------|
| EDM | `${FIELD_NAME}` | `${NAME}`、`${ORDER_ID}` |
| Surenotify | `{{variable_name}}` | `{{name}}`、`{{order_id}}` |

CLI 應在送出前檢查：

- EDM 指令的 HTML/subject 中包含 `{{...}}` 時發出警告
- Surenotify 指令的 HTML/subject/content 中包含 `${...}` 時發出警告

---

## 6. Error Codes 對照

### EDM API Error Codes

| Code | 意義 | CLI 行為 |
|------|------|---------|
| `40001` | 欄位驗證失敗 | 顯示欄位驗證提示 |
| `40003` | Email 格式無效 | 提示檢查 fromAddress |
| `40004` | 域名不允許 | 提示使用已驗證域名 |
| `40007` | SN 無效 | 提示檢查 campaign/group SN |
| `40008` | 檔案格式不支援 | 提示使用 CSV 或 Excel |
| `40009` | 檔案內容為空 | 提示檢查檔案 |
| `40010` | 檔案大小超限 | 提示分批匯入 |
| `40011` | 寄件地址未驗證 | 提示到 Dashboard 驗證 |
| `40012` | 餘額不足 | 顯示當前餘額，提示加值 |
| `40013` | 名單無可發送聯絡人 | 提示檢查名單狀態 |
| `40014` | 活動內容無效 | 提示檢查 HTML |
| `40015` | 發送資訊無效 | 提示檢查排程/寄件人設定 |
| `40017` | A/B 測試餘額不足 | 提示兩版本都需要額度 |
| `40019` | 排程時間無效 | 提示時間必須在未來 |
| `40020` | 日期格式無效 | 提示使用 ISO 8601 格式 |

### Exit Codes

| Code | 意義 | 對應錯誤 |
|------|------|---------|
| 0 | 成功 | — (含 dry-run、no-content) |
| 1 | API 錯誤 | HTTP 4xx/5xx 回應 |
| 2 | 驗證錯誤 | 參數驗證失敗 |
| 3 | 認證/設定錯誤 | API Key 無效、設定檔錯誤 |
| 4 | 網路/限流錯誤 | 連線失敗、rate limit 耗盡 |
| 5 | I/O 錯誤 | 檔案讀寫失敗 |

---

## 7. Rate Limits

| 限制類型 | 值 | CLI 處理方式 |
|---------|---|-------------|
| EDM 一般限流 | 2 requests/second | Token bucket 自動節流 |
| EDM 每日上限 | 300,000 requests/day | 超限時回傳 exit code 4 |
| Report 匯出限流 | 1 request/10 seconds | 獨立 token bucket |

---

## 8. 安裝管道

| 管道 | 指令 | 備註 |
|------|------|------|
| Homebrew | `brew install newsleopard/tap/nl` | macOS + Linux |
| Cargo | `cargo install nlm-cli` | 需要 Rust toolchain |
| GitHub Releases | 直接下載二進位 | 5 平台 pre-built |

---

## 9. 驗收標準

### PoC 驗證

1. `nl edm account balance` — 正確回傳帳戶餘額
2. `nl sn email send` — 成功送出交易型 Email
3. `nl config init` — 互動式建立設定檔

### 格式驗證

- `nl edm account balance --format table` 輸出正確的表格
- `nl edm account balance --format csv` 輸出正確的 CSV
- `nl edm account balance --format yaml` 輸出正確的 YAML

### Dry-run 驗證

- `nl edm campaign submit --dry-run ...` 輸出正確的 HTTP 請求預覽到 stderr

### 錯誤驗證

- 模擬 403、429、40011-40020 各 error code 的處理

### 跨平台驗證

- `cross build --target aarch64-apple-darwin` 確認 macOS ARM64 編譯成功
- 5 個目標平台全部成功編譯

### Eval 對齊

- CLI 行為與 `nlm-open-skills/evals/evals.json` 的 test cases 規格一致
