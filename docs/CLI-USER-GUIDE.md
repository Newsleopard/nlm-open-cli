# CLI 使用手冊 — nl

## 1. 安裝與設定

### 1.1 安裝

```bash
# Homebrew (macOS / Linux)
brew install newsleopard/tap/nl

# Cargo
cargo install nl-cli

# 從原始碼
git clone https://github.com/Newsleopard/nlm-open-cli.git && cd nlm-open-cli
cargo build --release
```

### 1.2 初始設定

```bash
nl config init
```

互動式流程會提示你輸入：
1. EDM API Key（從 Newsleopard Dashboard → 設定 → API Key 取得）
2. Surenotify API Key（從 Surenotify Dashboard → 設定 → API Key 取得）
3. 預設輸出格式（json / table / yaml / csv）

設定檔將寫入 `~/.config/nl/config.toml`。

### 1.3 手動設定

```bash
# 設定單一值
nl config set edm_api_key "your-key-here"
nl config set sn_api_key "your-sn-key-here"
nl config set default_format table

# 查看設定
nl config get edm_api_key    # 顯示 ****...（遮蔽）
nl config list               # 列出所有設定
```

### 1.4 多環境 Profile

```bash
# 建立 staging profile
nl config profile create staging
nl config set edm_api_key "staging-key" --profile staging

# 使用 staging
nl edm account balance --profile staging

# 或透過環境變數
NL_PROFILE=staging nl edm account balance

# 列出所有 profiles
nl config profile list

# 刪除 profile
nl config profile delete staging
```

### 1.5 環境變數

| 變數 | 說明 | 對應 flag |
|------|------|---------|
| `NL_EDM_API_KEY` | EDM API Key | — |
| `NL_SN_API_KEY` | Surenotify API Key | — |
| `NL_PROFILE` | 使用的 Profile 名稱 | `--profile` |
| `NL_FORMAT` | 預設輸出格式 | `--format` |

**優先順序：** 環境變數 > CLI flag > Profile 設定 > `[default]` 設定

---

## 2. 全域參數

所有指令都支援以下全域參數：

```
--format <json|table|yaml|csv>   輸出格式（預設: json）
--profile <NAME>                 設定檔 Profile（預設: default）
--dry-run                        預覽 HTTP 請求而不執行
-v, --verbose                    顯示請求詳情（-v 摘要, -vv 完整）
-q, --quiet                      只輸出錯誤
```

### 2.1 輸出格式

#### JSON（預設）

```bash
nl edm account balance
```
```json
{
  "email": 10000,
  "sms": 500
}
```

在 pipe 中自動切換為 compact：

```bash
nl edm account balance | jq '.email'
# 10000
```

#### Table

```bash
nl edm contacts list-groups --format table
```
```
┌──────────┬─────────────┬───────────────┬────────────┬────────────┬──────────┐
│ sn       │ name        │ subscribedCnt │ openedRate │ clickedRate │ status   │
├──────────┼─────────────┼───────────────┼────────────┼────────────┼──────────┤
│ GRP-001  │ VIP 客戶    │         1,234 │     32.5%  │      8.2%  │ GENERAL  │
│ GRP-002  │ 新用戶      │           456 │     28.1%  │      5.7%  │ GENERAL  │
│ GRP-003  │ 匯入中      │             0 │      0.0%  │      0.0%  │ PROCESS… │
└──────────┴─────────────┴───────────────┴────────────┴────────────┴──────────┘
```

#### YAML

```bash
nl edm account balance --format yaml
```
```yaml
email: 10000
sms: 500
```

#### CSV

```bash
nl edm contacts list-groups --format csv > groups.csv
```
```csv
sn,name,subscribedCnt,openedRate,clickedRate,status
GRP-001,VIP 客戶,1234,32.5,8.2,GENERAL
GRP-002,新用戶,456,28.1,5.7,GENERAL
```

### 2.2 Dry-run

預覽 HTTP 請求而不實際送出：

```bash
nl edm campaign submit \
  --name "Test" \
  --lists GRP-001 \
  --subject "Hello" \
  --from-name "Sender" \
  --from-address "test@example.com" \
  --html "<p>Test</p>" \
  --dry-run
```

輸出到 stderr：
```json
{
  "dry_run": {
    "method": "POST",
    "url": "https://api.newsleopard.com/v1/campaign/normal/submit",
    "headers": {
      "x-api-key": "****...",
      "content-type": "application/json"
    },
    "body": {
      "form": {
        "name": "Test",
        "selectedLists": ["GRP-001"],
        "excludeLists": []
      },
      "content": {
        "subject": "Hello",
        "fromName": "Sender",
        "fromAddress": "test@example.com",
        "htmlContent": "<p>Test</p>",
        "footerLang": 1
      },
      "config": {
        "schedule": { "type": 0 },
        "ga": { "enable": false, "ecommerceEnable": false }
      }
    }
  }
}
```

### 2.3 Verbose 模式

```bash
# -v: 顯示 HTTP 方法和 URL
nl edm account balance -v
# → GET https://api.newsleopard.com/v1/balance [200 OK] 42ms

# -vv: 顯示完整 headers 和 body
nl edm account balance -vv
# → GET https://api.newsleopard.com/v1/balance
# → Request Headers: { x-api-key: ****..., content-type: application/json }
# → Response Status: 200 OK (42ms)
# → Response Headers: { content-type: application/json, ... }
# → Response Body: { "email": 10000, "sms": 500 }
```

---

## 3. EDM API 指令

### 3.1 帳戶（`nl edm account`）

#### 查詢餘額

```bash
nl edm account balance
```

```json
{ "email": 10000, "sms": 500 }
```

---

### 3.2 聯絡人管理（`nl edm contacts`）

#### 建立群組

```bash
nl edm contacts create-group --name "VIP 客戶"
```

```json
{ "sn": "GRP-abc123" }
```

#### 列出群組

```bash
# 預設分頁
nl edm contacts list-groups

# 指定分頁
nl edm contacts list-groups --size 10 --page 2

# 取得所有頁面（NDJSON 串流）
nl edm contacts list-groups --page-all
```

#### 檔案匯入聯絡人

```bash
# 基本匯入
nl edm contacts import-file --list-sn GRP-abc123 --file contacts.csv

# 匯入並等待完成
nl edm contacts import-file --list-sn GRP-abc123 --file contacts.csv --wait

# 指定 webhook 回呼
nl edm contacts import-file --list-sn GRP-abc123 --file contacts.csv \
  --webhook-url https://api.example.com/import-callback

# 自訂輪詢間隔（秒）
nl edm contacts import-file --list-sn GRP-abc123 --file contacts.csv \
  --wait --poll-interval 10
```

**CSV 檔案格式要求：**
- 必須有 `EMAIL` 欄位
- 自訂欄位必須先在 Dashboard 建立
- 檔案大小限制：10 MB
- 支援格式：CSV、Excel

#### 文字匯入聯絡人

```bash
# 直接傳入 CSV 文字
nl edm contacts import-text --list-sn GRP-abc123 \
  --csv-text "EMAIL,NAME
user1@example.com,Alice
user2@example.com,Bob"

# 從檔案讀取 CSV 文字
nl edm contacts import-text --list-sn GRP-abc123 --csv-file contacts.csv
```

#### 查詢匯入狀態

```bash
nl edm contacts import-status --import-sn IMP-xyz789
```

```json
{
  "import_sn": "IMP-xyz789",
  "status": "COMPLETE",
  "fileCnt": 1000,
  "insertCnt": 985,
  "duplicateCnt": 12,
  "errCnt": 3,
  "errorDownloadLink": "https://..."
}
```

**Status 值：**
| Status | 意義 |
|--------|------|
| `PROCESSING` | 匯入進行中 |
| `COMPLETE` | 匯入完成 |
| `DUPLICATE_HEADER` | CSV 有重複欄位名 |
| `ERROR` | 匯入失敗 |
| `MISSING_REQUIRED_DATA` | 缺少必要欄位 |

#### 刪除聯絡人

```bash
nl edm contacts remove \
  --list-sn GRP-abc123 \
  --field DOMAIN \
  --operator like \
  --value "example.com"
```

**可用 operators：** `eq`、`not-eq`、`like`、`not-like`

**可用 fields：** `NAME`、`MAIL_ADDRESS`、`DOMAIN`、`LISTSN`、或自訂欄位

---

### 3.3 活動管理（`nl edm campaign`）

#### 送出活動

```bash
nl edm campaign submit \
  --name "三月電子報" \
  --lists GRP-001,GRP-002 \
  --subject "三月份精選優惠 — 最高 5 折" \
  --from-name "品牌電子報" \
  --from-address "newsletter@example.com" \
  --html-file template.html \
  --footer-lang chinese \
  --preheader "限時優惠，即將截止" \
  --schedule immediate
```

完整參數：

| 參數 | 必要 | 說明 |
|------|------|------|
| `--name` | 是 | 活動名稱 |
| `--lists` | 是 | 目標群組 SN（逗號分隔） |
| `--subject` | 是 | 郵件主旨（最多 150 字） |
| `--from-name` | 是 | 寄件人名稱（最多 50 字） |
| `--from-address` | 是 | 寄件人地址（需在 Dashboard 驗證） |
| `--html` / `--html-file` | 是 | HTML 內容（內聯或檔案） |
| `--footer-lang` | 否 | 頁尾語言：`english` / `chinese`（預設 chinese） |
| `--preheader` | 否 | 預覽文字（最多 60 字） |
| `--exclude-lists` | 否 | 排除群組 SN |
| `--schedule` | 否 | `immediate`（預設）/ `scheduled` |
| `--schedule-date` | 否 | 排程時間（UTC，ISO 8601） |
| `--schedule-timezone` | 否 | 時區代碼 |
| `--ga` | 否 | 啟用 GA 追蹤 |
| `--ga-ecommerce` | 否 | 啟用 GA 電商分析 |
| `--utm-campaign` | 否 | utm_campaign 值 |
| `--utm-content` | 否 | utm_content 值 |

**排程發送範例：**

```bash
nl edm campaign submit \
  --name "排程活動" \
  --lists GRP-001 \
  --subject "排程測試" \
  --from-name "Test" \
  --from-address "test@example.com" \
  --html "<p>Hello</p>" \
  --schedule scheduled \
  --schedule-date "2026-04-01T08:00:00.000Z" \
  --schedule-timezone 21
```

**GA 追蹤範例：**

```bash
nl edm campaign submit \
  --name "GA 追蹤活動" \
  --lists GRP-001 \
  --subject "促銷活動" \
  --from-name "品牌" \
  --from-address "promo@example.com" \
  --html-file promo.html \
  --ga \
  --ga-ecommerce \
  --utm-campaign "spring-promo" \
  --utm-content "main-cta"
```

#### 單次上傳活動

不儲存聯絡人，發送後即丟棄：

```bash
nl edm campaign submit-once \
  --contacts-file one-time-list.csv \
  --name "一次性發送" \
  --subject "特別通知" \
  --from-name "系統通知" \
  --from-address "notify@example.com" \
  --html-file notification.html
```

#### 查詢活動狀態

```bash
nl edm campaign status --sn CAMP-abc123
```

```json
{
  "sn": "CAMP-abc123",
  "name": "三月電子報",
  "status": "SENDING",
  "sendTimeType": "NOW",
  "type": "REGULAR",
  "sentBeginDate": "2026-03-16T02:00:00Z"
}
```

**狀態值：** `DRAFT`、`COMPLETE`、`STOP`、`SENDING`、`PREPARE`、`PREPARE_TO_SENT`、`OVER_LIMIT`、`TESTING`、`EMPTY_RECIPIENT`、`INSUFFICIENT_RECIPIENT_FOR_TESTING`、`UNAUTHENTICATED_SENDER`

#### 暫停活動

```bash
nl edm campaign pause --sn CAMP-abc123
# 204 No Content → exit code 0
```

#### 刪除活動

```bash
nl edm campaign delete --sns CAMP-001,CAMP-002
```

```json
{
  "success": ["CAMP-001"],
  "sendingCampaign": ["CAMP-002"],
  "badCampaigns": []
}
```

---

### 3.4 A/B 測試（`nl edm ab-test`）

#### 送出 A/B 測試

```bash
# 主旨測試
nl edm ab-test submit \
  --name "主旨 A/B 測試" \
  --lists GRP-001 \
  --test-on subject \
  --proportion 20 \
  --test-duration 4 \
  --test-unit hours \
  --subject-a "限時優惠 — 全站 8 折" \
  --subject-b "VIP 專屬 — 獨享 8 折優惠" \
  --from-name "品牌名稱" \
  --from-address "promo@example.com" \
  --html-file promo.html

# 寄件人測試
nl edm ab-test submit \
  --name "寄件人 A/B 測試" \
  --lists GRP-001 \
  --test-on sender \
  --proportion 30 \
  --test-duration 1 \
  --test-unit days \
  --subject "促銷活動" \
  --from-name-a "品牌名稱" \
  --from-address-a "brand@example.com" \
  --from-name-b "CEO Name" \
  --from-address-b "ceo@example.com" \
  --html-file promo.html

# 內容測試
nl edm ab-test submit \
  --name "內容 A/B 測試" \
  --lists GRP-001 \
  --test-on content \
  --proportion 25 \
  --test-duration 6 \
  --test-unit hours \
  --subject "促銷活動" \
  --from-name "品牌" \
  --from-address "promo@example.com" \
  --html-content-a-file version-a.html \
  --html-content-b-file version-b.html
```

**A/B 測試參數：**

| 參數 | 說明 |
|------|------|
| `--test-on` | 測試維度：`subject` / `sender` / `content` |
| `--proportion` | 測試比例 0-100（%） |
| `--test-duration` | 測試時長 |
| `--test-unit` | 時間單位：`hours` / `days` |

#### 單次上傳 A/B 測試

```bash
nl edm ab-test submit-once \
  --contacts-file one-time-list.csv \
  --test-on subject \
  # ... 其餘參數同 submit
```

---

### 3.5 報告（`nl edm report`）

#### 列出活動碼

```bash
nl edm report list \
  --start-date "2026-03-01T00:00:00.00Z" \
  --end-date "2026-03-16T23:59:59.00Z"
```

#### 活動績效指標

```bash
nl edm report metrics --sns CAMP-001,CAMP-002

# Table 格式（方便閱讀）
nl edm report metrics --sns CAMP-001,CAMP-002 --format table
```

```json
[
  {
    "campaignSn": "CAMP-001",
    "name": "三月電子報",
    "channel": "MAIL",
    "recipientCnt": 5000,
    "delivered": 4850,
    "bounced": 150,
    "opened": 1520,
    "clicked": 380,
    "complained": 2,
    "unsubscribed": 15
  }
]
```

#### 匯出詳細報告

```bash
# 觸發匯出
nl edm report export --sn CAMP-001

# 匯出並等待完成後下載
nl edm report export --sn CAMP-001 --wait --output ./report.csv
```

**注意：** 匯出有 1 request/10 seconds 的限流。

#### 取得報告下載連結

```bash
nl edm report download-link --sn CAMP-001
```

---

### 3.6 範本（`nl edm template`）

#### 列出範本

```bash
nl edm template list
nl edm template list --format table
```

#### 取得範本內容

```bash
# 輸出到 stdout
nl edm template get --id TPL-001

# 儲存到檔案
nl edm template get --id TPL-001 --output template.html
```

---

### 3.7 自動化（`nl edm automation`）

#### 觸發/停止自動化腳本

```bash
# 觸發
nl edm automation trigger \
  --workflow "auto-script-001" \
  --event trigger \
  --recipients '[{"name":"Alice","address":"alice@example.com","variables":{"ORDER_ID":"A123"}}]'

# 從檔案讀取收件者
nl edm automation trigger \
  --workflow "auto-script-001" \
  --event trigger \
  --recipients-file recipients.json

# 停止
nl edm automation trigger \
  --workflow "auto-script-001" \
  --event terminate \
  --recipients '[{"name":"Alice","address":"alice@example.com"}]'
```

**收件者上限：** 100 人/次

---

## 4. Surenotify API 指令

### 4.1 Email（`nl sn email`）

#### 發送 Email

```bash
# 基本發送
nl sn email send \
  --subject "訂單確認" \
  --from-address "noreply@example.com" \
  --html "<h1>您的訂單已確認</h1>" \
  --to alice@example.com,bob@example.com

# 含個人化變數
nl sn email send \
  --subject "訂單確認 {{order_id}}" \
  --from-address "noreply@example.com" \
  --from-name "Example Store" \
  --html "<h1>Hi {{name}}</h1><p>訂單 {{order_id}} 已確認。</p>" \
  --recipients '[
    {"name":"Alice","address":"alice@example.com","variables":{"name":"Alice","order_id":"ORD-001"}},
    {"name":"Bob","address":"bob@example.com","variables":{"name":"Bob","order_id":"ORD-002"}}
  ]'

# 從檔案讀取 HTML 和收件者
nl sn email send \
  --subject "週報" \
  --from-address "report@example.com" \
  --html-file weekly-report.html \
  --recipients-file team.json

# 含取消訂閱連結
nl sn email send \
  --subject "電子報" \
  --from-address "news@example.com" \
  --html-file newsletter.html \
  --to user@example.com \
  --unsubscribe-link "https://example.com/unsubscribe"
```

**注意：** Surenotify 變數語法為 `{{variable}}`，不同於 EDM 的 `${FIELD}`。

**收件者上限：** 100 人/次

#### 查詢 Email 事件

```bash
# 用 message ID 查詢
nl sn email events --id "msg-uuid-123"

# 用收件者地址查詢
nl sn email events --recipient alice@example.com

# 加上時間範圍和狀態篩選
nl sn email events \
  --recipient alice@example.com \
  --from "2026-03-01T00:00:00.00Z" \
  --to "2026-03-16T23:59:59.00Z" \
  --status delivery
```

**注意：** `--id` 和 `--recipient` 互斥，只能擇一使用。

**狀態值：** `accept`、`retry`、`delivery`、`open`、`click`、`bounce`、`complaint`

**限制：** 30 天歷史上限，每次查詢最多 50 筆

---

### 4.2 SMS（`nl sn sms`）

#### 發送簡訊

```bash
# 基本發送
nl sn sms send \
  --content "【Example Store】您的驗證碼是 {{code}}" \
  --phone 0912345678 \
  --country-code 886

# 批次發送
nl sn sms send \
  --content "【Example Store】親愛的 {{name}}，您的訂單 {{order_id}} 已出貨" \
  --recipients '[
    {"address":"0912345678","country_code":"886","variables":{"name":"Alice","order_id":"A001"}},
    {"address":"0923456789","country_code":"886","variables":{"name":"Bob","order_id":"A002"}}
  ]'

# 從檔案讀取收件者
nl sn sms send \
  --content "【品牌】促銷通知" \
  --recipients-file sms-recipients.json

# 指定專屬號碼和重試時間
nl sn sms send \
  --content "【Brand】Verification code: {{code}}" \
  --phone 0912345678 \
  --country-code 886 \
  --from 0900123456 \
  --alive-mins 30
```

**重要：**
- 簡訊內容必須包含公司/品牌名稱（NCC 規定）
- 電話號碼必須為純數字（不可包含 `+`、`-`、空格）
- 簡訊中的 URL 需申請白名單

#### 查詢簡訊事件

```bash
# 用 message ID 查詢
nl sn sms events --id "msg-uuid-456"

# 用收件者查詢
nl sn sms events --recipient 0912345678 --country-code 886

# 加篩選條件
nl sn sms events \
  --recipient 0912345678 \
  --country-code 886 \
  --from "2026-03-01T00:00:00.00Z" \
  --status delivery
```

**狀態值：** `accept`、`delivery`、`bounce`

#### 查詢專屬號碼

```bash
nl sn sms exclusive-number
```

```json
{
  "phoneNumbers": [
    {
      "phoneNumber": "0900123456",
      "createDate": "2026-01-15T08:00:00Z",
      "updateDate": "2026-01-15T08:00:00Z"
    }
  ]
}
```

---

### 4.3 Email Webhook（`nl sn webhook`）

#### 建立/更新 Webhook

```bash
nl sn webhook create --event-type delivery --url "https://api.example.com/webhooks/delivery"
nl sn webhook create --event-type open --url "https://api.example.com/webhooks/open"
nl sn webhook create --event-type click --url "https://api.example.com/webhooks/click"
nl sn webhook create --event-type bounce --url "https://api.example.com/webhooks/bounce"
nl sn webhook create --event-type complaint --url "https://api.example.com/webhooks/complaint"
```

**事件類型：** `delivery`、`open`、`click`、`bounce`、`complaint`

#### 列出 Webhook

```bash
nl sn webhook list
nl sn webhook list --format table
```

#### 刪除 Webhook

```bash
nl sn webhook delete --event-type bounce
```

---

### 4.4 SMS Webhook（`nl sn sms-webhook`）

#### 建立/更新 SMS Webhook

```bash
nl sn sms-webhook create --event-type delivery --url "https://api.example.com/sms-webhooks/delivery"
nl sn sms-webhook create --event-type bounce --url "https://api.example.com/sms-webhooks/bounce"
```

**SMS 事件類型：** `delivery`、`bounce`

#### 列出 SMS Webhook

```bash
nl sn sms-webhook list
```

#### 刪除 SMS Webhook

```bash
nl sn sms-webhook delete --event-type delivery
```

---

### 4.5 域名驗證（`nl sn domain`）

#### 建立域名驗證記錄

```bash
nl sn domain create --domain mail.example.com
```

```json
[
  {
    "name": "mail.example.com",
    "value": "v=spf1 include:amazonses.com include:mailgun.org ?all",
    "record_type": 0,
    "valid": false
  },
  {
    "name": "selector._domainkey.mail.example.com",
    "value": "...",
    "record_type": 1,
    "valid": false
  }
]
```

**record_type：** `0` = TXT、`1` = CNAME

#### 驗證 DNS 記錄

```bash
nl sn domain verify --domain mail.example.com
```

回傳同樣的 DNS 記錄陣列，但 `valid` 欄位會更新。

#### 移除域名驗證

```bash
nl sn domain remove --domain mail.example.com
```

**設定流程：**
1. `nl sn domain create` — 取得需要設定的 DNS 記錄
2. 到域名註冊商設定 DNS 記錄
3. 等待 DNS 傳播（最長 48 小時）
4. `nl sn domain verify` — 驗證設定是否正確

---

## 5. Helper 編排指令（`nl helper` / `nl x`）

Helper 指令將多步驟工作流程封裝為一鍵操作。可用 `nl helper` 或簡寫 `nl x` 呼叫。

### 5.1 campaign-send — 一鍵發送活動

```bash
nl helper campaign-send \
  --name "三月電子報" \
  --lists GRP-001 \
  --subject "三月份精選優惠" \
  --from-name "品牌" \
  --from-address "newsletter@example.com" \
  --html-file template.html \
  --wait
```

**流程：**
1. 檢查帳戶餘額是否足夠
2. 驗證目標名單是否有效
3. 送出活動
4. （`--wait`）輪詢活動狀態直到完成
5. （`--wait`）取得並顯示績效指標

**支援所有 `campaign submit` 的參數，額外增加 `--wait` flag。**

### 5.2 import-and-wait — 匯入並等待完成

```bash
nl helper import-and-wait \
  --list-sn GRP-001 \
  --file customers.csv \
  --timeout 600
```

**流程：**
1. 呼叫檔案匯入 API 取得上傳 URL
2. PUT 檔案到 pre-signed URL
3. 輪詢匯入狀態（含 progress bar）
4. 顯示匯入結果摘要
5. 若有錯誤，自動下載錯誤明細 CSV

**輸出範例：**
```
⠋ 匯入進行中... [=========>          ] 45%
✓ 匯入完成: 15,230 筆成功, 42 筆重複, 3 筆失敗
  失敗明細已下載至: import-errors-IMP-xyz789.csv
```

### 5.3 report-download — 匯出並下載報告

```bash
nl helper report-download --sn CAMP-001 --output ./report.csv
```

**流程：**
1. 觸發報告匯出（`POST /v1/report/{sn}/export`）
2. 輪詢下載連結（`GET /v1/report/{sn}/link`）
3. 下載 CSV 檔案到指定路徑

### 5.4 domain-setup — 域名驗證設定

```bash
# 手動驗證（顯示 DNS 記錄後等待使用者設定）
nl helper domain-setup --domain mail.example.com

# 自動等待後驗證
nl helper domain-setup --domain mail.example.com --auto-verify-after 300
```

**流程：**
1. 建立域名驗證記錄
2. 以 Table 格式顯示需要設定的 DNS 記錄
3. （`--auto-verify-after N`）等待 N 秒後自動驗證
4. 驗證結果

---

## 6. 設定管理指令（`nl config`）

### 6.1 互動式初始化

```bash
nl config init
```

### 6.2 設定值

```bash
nl config set <KEY> <VALUE>
nl config set edm_api_key "your-key" --profile staging
```

### 6.3 讀取值

```bash
nl config get <KEY>
nl config get edm_api_key
# → ****...abc (遮蔽前綴，顯示末 3 碼)
```

### 6.4 列出所有設定

```bash
nl config list
```

```
Profile: default
  edm_api_key: ****...abc
  sn_api_key:  ****...xyz
  default_format: json

Profile: staging
  edm_api_key: ****...def
  sn_api_key:  ****...uvw
```

### 6.5 Profile 管理

```bash
nl config profile create staging
nl config profile list
nl config profile delete staging
```

---

## 7. 錯誤處理

### 7.1 Exit Codes

| Code | 意義 | 說明 |
|------|------|------|
| 0 | 成功 | 含 dry-run 和 204 No Content |
| 1 | API 錯誤 | HTTP 4xx/5xx 回應 |
| 2 | 驗證錯誤 | CLI 參數驗證失敗 |
| 3 | 認證/設定錯誤 | API Key 無效或設定檔錯誤 |
| 4 | 網路/限流錯誤 | 連線失敗或 rate limit |
| 5 | I/O 錯誤 | 檔案讀寫失敗 |

### 7.2 錯誤輸出

所有錯誤以 JSON 格式輸出到 stderr：

```json
{
  "error": {
    "type": "api",
    "message": "API error 400: [40012] Insufficient balance",
    "exit_code": 1
  }
}
```

### 7.3 重試行為

對以下錯誤自動重試（exponential backoff，500ms → 30s，最大 120s）：
- HTTP 429 (Too Many Requests)
- HTTP 5xx (Server Error)
- 網路連線錯誤

### 7.4 常見錯誤排除

```bash
# API Key 問題
nl edm account balance
# → {"error": {"type": "auth", "message": "Authentication error: Forbidden", ...}}
# 解法: nl config set edm_api_key "correct-key"

# 寄件人未驗證
nl edm campaign submit --from-address "unverified@example.com" ...
# → {"error": {"type": "api", "message": "API error 400: [40011] Unverified sender address", ...}}
# 解法: 到 Dashboard → 設定 → 驗證寄件人地址

# 餘額不足
nl edm campaign submit ...
# → {"error": {"type": "api", "message": "API error 400: [40012] Insufficient balance", ...}}
# 解法: nl edm account balance 查看餘額，聯繫客服加值

# 變數語法混用
nl edm campaign submit --html "<p>Hi {{name}}</p>" ...
# → Warning: EDM API 使用 ${FIELD} 變數語法，偵測到 {{...}} 格式（Surenotify 語法）
```

---

## 8. Scripting 最佳實踐

### 8.1 搭配 jq 使用

```bash
# 取得 email 餘額
EMAIL_BALANCE=$(nl edm account balance -q | jq '.email')

# 取得所有群組 SN
nl edm contacts list-groups --page-all -q | jq -r '.sn'

# 篩選開信率 > 30% 的群組
nl edm contacts list-groups --page-all -q | jq 'select(.openedRate > 30)'
```

### 8.2 搭配 xargs 批次處理

```bash
# 刪除所有 DRAFT 狀態的活動
nl edm report list --start-date "$START" --end-date "$END" -q \
  | jq -r '.[].sn' \
  | xargs -I{} nl edm campaign status --sn {} -q \
  | jq -r 'select(.status == "DRAFT") | .sn' \
  | paste -sd, - \
  | xargs -I{} nl edm campaign delete --sns {}
```

### 8.3 錯誤處理範本

```bash
#!/bin/bash
set -euo pipefail

# 發送並檢查結果
RESULT=$(nl sn email send \
  --subject "Test" \
  --from-address "test@example.com" \
  --html "<p>Test</p>" \
  --to user@example.com 2>&1) || {
    EXIT_CODE=$?
    echo "發送失敗 (exit $EXIT_CODE): $RESULT" >&2
    exit $EXIT_CODE
}

# 取得 message ID
MSG_ID=$(echo "$RESULT" | jq -r '.success[0].id')
echo "發送成功: $MSG_ID"

# 查詢送達狀態
sleep 30
nl sn email events --id "$MSG_ID" --status delivery
```

### 8.4 CSV 輸出到 spreadsheet

```bash
# 匯出活動指標為 CSV
nl edm report metrics --sns CAMP-001,CAMP-002,CAMP-003 --format csv > metrics.csv

# 匯出所有群組資訊
nl edm contacts list-groups --page-all --format csv > groups.csv
```

---

## 9. 變數語法速查

| API | 語法 | 範例 | CLI 指令群 |
|-----|------|------|-----------|
| EDM | `${FIELD_NAME}` | `${NAME}`, `${ORDER_ID}` | `nl edm campaign`, `nl edm ab-test` |
| Surenotify | `{{variable_name}}` | `{{name}}`, `{{order_id}}` | `nl sn email`, `nl sn sms` |

**注意：** 混用變數語法會導致變數無法替換。CLI 會在送出前檢查並發出警告。

---

## 10. Rate Limits 速查

| 限制 | 值 | 影響指令 |
|------|---|---------|
| EDM 一般限流 | 2 requests/second | 所有 `nl edm` 指令 |
| EDM 每日上限 | 300,000 requests/day | 所有 `nl edm` 指令 |
| Report 匯出限流 | 1 request/10 seconds | `nl edm report export` |
| SN 收件者上限 | 100 人/request | `nl sn email send`, `nl sn sms send` |
| SN 變數長度 | 100 字元/值 | 所有含 variables 的指令 |

CLI 內建 token bucket 限流器，自動遵守 rate limits。超過每日上限時會回傳 exit code 4。
