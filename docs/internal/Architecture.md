# 技術架構設計 — nl CLI

## 1. 架構總覽

`nl` CLI 採用分層架構，借鑑 Google Workspace CLI (`gws`) 的設計模式，針對 Newsleopard 已知且固定的 API 面（31 endpoints）做靜態化調整。

```
┌──────────────────────────────────────────────────────────┐
│                     main.rs                              │
│            (entry point, tracing init)                   │
└────────────────────────┬─────────────────────────────────┘
                         │
┌────────────────────────▼─────────────────────────────────┐
│                    cli/ (clap derive)                     │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐ │
│  │  edm/    │  │   sn/    │  │ config   │  │ helper   │ │
│  │ contacts │  │  email   │  │ init     │  │ campaign │ │
│  │ campaign │  │  sms     │  │ set/get  │  │ import   │ │
│  │ ab_test  │  │ webhook  │  │ profile  │  │ report   │ │
│  │ report   │  │sms-whook │  │          │  │ domain   │ │
│  │ template │  │ domain   │  │          │  │          │ │
│  │ automate │  │          │  │          │  │          │ │
│  │ account  │  │          │  │          │  │          │ │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘ │
└────────────────────────┬─────────────────────────────────┘
                         │
┌────────────────────────▼─────────────────────────────────┐
│                   executor/                              │
│         (CLI args → client call → format output)         │
└─────────┬──────────────┬─────────────────┬───────────────┘
          │              │                 │
┌─────────▼────┐  ┌──────▼───────┐  ┌─────▼──────────────┐
│   client/    │  │  formatter/  │  │     helpers/       │
│  ApiClient   │  │  json        │  │  campaign_send     │
│  EdmClient   │  │  table       │  │  import_wait       │
│  SnClient    │  │  yaml        │  │  report_export     │
│  rate_limiter│  │  csv_fmt     │  │  domain_setup      │
│  retry       │  │              │  │                    │
└─────────┬────┘  └──────────────┘  └────────────────────┘
          │
┌─────────▼────────────────────────────────────────────────┐
│                    共用層                                 │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐               │
│  │ config/  │  │  types/  │  │ error.rs │               │
│  │ TOML     │  │  edm.rs  │  │ NlError  │               │
│  │ profiles │  │  sn.rs   │  │ exit code│               │
│  │ env vars │  │          │  │          │               │
│  └──────────┘  └──────────┘  └──────────┘               │
└──────────────────────────────────────────────────────────┘
```

---

## 2. 專案結構

```
nl-cli/
├── Cargo.toml
├── src/
│   ├── main.rs                    # Entry point: parse args, init tracing, run executor
│   ├── cli/
│   │   ├── mod.rs                 # NlCli top-level struct, Command enum, OutputFormat, global flags
│   │   ├── edm/
│   │   │   ├── mod.rs             # EdmCommand enum
│   │   │   ├── contacts.rs        # create-group, list-groups, import-file, import-text, import-status, remove
│   │   │   ├── campaign.rs        # submit, submit-once, delete, pause, status
│   │   │   ├── ab_test.rs         # submit, submit-once
│   │   │   ├── report.rs          # list, metrics, export, download-link
│   │   │   ├── template.rs        # list, get
│   │   │   ├── automation.rs      # trigger
│   │   │   └── account.rs         # balance
│   │   └── sn/
│   │       ├── mod.rs             # SnCommand enum
│   │       ├── email.rs           # send, events
│   │       ├── sms.rs             # send, events, exclusive-number
│   │       ├── webhook.rs         # email webhook + SMS webhook CRUD
│   │       └── domain.rs          # create, verify, remove
│   ├── client/
│   │   ├── mod.rs                 # ApiClient struct (shared HTTP, rate limiter, dry-run)
│   │   ├── edm.rs                 # EdmClient — 20 endpoint methods
│   │   ├── surenotify.rs          # SurenotifyClient — 11+ endpoint methods
│   │   ├── rate_limiter.rs        # Token bucket (2 req/sec EDM, 1 req/10s report export)
│   │   └── retry.rs               # Exponential backoff for 429/5xx
│   ├── config/
│   │   └── mod.rs                 # Config load/save, env var override, TOML profiles
│   ├── executor/
│   │   └── mod.rs                 # CLI args → client call → format output
│   ├── formatter/
│   │   ├── mod.rs                 # OutputFormat dispatch
│   │   ├── json.rs                # Pretty JSON / compact NDJSON
│   │   ├── table.rs               # Auto-flatten nested JSON → tabled
│   │   ├── yaml.rs                # serde_yaml
│   │   └── csv_fmt.rs             # CSV with header detection
│   ├── error.rs                   # NlError enum, exit codes (0-5), JSON stderr output
│   ├── types/
│   │   ├── mod.rs
│   │   ├── edm.rs                 # EDM request/response structs
│   │   └── surenotify.rs          # SN request/response structs
│   └── helpers/
│       ├── mod.rs
│       ├── campaign_send.rs       # balance check → validate lists → submit → poll status → metrics
│       ├── import_wait.rs         # get upload URL → PUT file → poll import status
│       ├── report_export.rs       # export → poll → download
│       └── domain_setup.rs        # create → show DNS → verify
├── tests/
│   ├── edm_api_test.rs            # wiremock HTTP mock tests for EDM
│   ├── sn_api_test.rs             # wiremock HTTP mock tests for SN
│   └── cli_integration_test.rs    # assert_cmd end-to-end tests
└── .github/workflows/ci.yml       # fmt + clippy + test + cross-compile release
```

**預估規模：** ~5,000–7,000 LOC

---

## 3. 依賴清單

```toml
[package]
name = "nl-cli"
version = "0.1.0"
edition = "2021"
description = "Newsleopard EDM & Surenotify CLI"
rust-version = "1.75"

[[bin]]
name = "nlm"
path = "src/main.rs"

[dependencies]
clap = { version = "4.5", features = ["derive", "env", "color", "wrap_help"] }
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json", "rustls-tls"], default-features = false }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
csv = "1.3"
thiserror = "2"
anyhow = "1"
toml = "0.8"
dirs = "5"
governor = "0.7"
backoff = { version = "0.4", features = ["tokio"] }
tabled = "0.16"
colored = "2.1"
indicatif = "0.17"
chrono = { version = "0.4", features = ["serde"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }

[dev-dependencies]
wiremock = "0.6"
assert_cmd = "2"
predicates = "3"
tempfile = "3"
insta = { version = "1", features = ["json", "yaml"] }

[profile.release]
lto = "thin"
strip = true
```

### 依賴選型理由

| Crate | 用途 | 選型理由 |
|-------|------|---------|
| `clap` (derive) | CLI 解析 | 靜態型別指令樹，編譯期檢查；vs gws 的動態模式，因 NL API 面固定 |
| `reqwest` (rustls) | HTTP 客戶端 | 跨平台不依賴 OpenSSL；同 gws 選擇 |
| `tokio` | 非同步 runtime | reqwest 和 backoff 的底層需求 |
| `serde` + `serde_json` | 序列化 | Rust 生態標準 |
| `thiserror` | 錯誤型別 | 參考 gws 的 GwsError 模式 |
| `anyhow` | 頂層錯誤 | main 函數的 error propagation |
| `governor` | Rate limiting | Token bucket 實作；vs gws 手寫限流器，crate 更可靠 |
| `backoff` | 重試機制 | Exponential backoff with jitter |
| `tabled` | Table 格式 | 自動從 struct 產生表格 |
| `indicatif` | 進度條 | Helper 指令的輪詢進度顯示 |
| `toml` | 設定檔 | Rust 生態標準設定格式；vs gws 的 JSON |
| `dirs` | 路徑解析 | 跨平台 `~/.config` 路徑 |
| `colored` | 色彩輸出 | 終端色彩提示 |
| `chrono` | 時間處理 | ISO 8601 日期解析/格式化 |
| `tracing` | 結構化 log | 取代 env_logger，支援 JSON 輸出 |
| `wiremock` | HTTP mock | 測試用 mock server |
| `assert_cmd` | CLI 測試 | 端對端指令測試 |
| `insta` | Snapshot 測試 | Formatter 輸出穩定性驗證 |

---

## 4. 核心設計模式

### 4.1 指令解析層（clap derive）

**設計決策：** 靜態 clap derive structs，非動態 Discovery 模式。

```rust
/// 頂層 CLI 結構
#[derive(Parser)]
#[command(name = "nl", about = "Newsleopard EDM & Surenotify CLI")]
pub struct NlCli {
    #[command(subcommand)]
    pub command: Command,

    /// 輸出格式
    #[arg(long, global = true, default_value = "json", env = "NL_FORMAT")]
    pub format: OutputFormat,

    /// 設定檔 Profile
    #[arg(long, global = true, default_value = "default", env = "NL_PROFILE")]
    pub profile: String,

    /// 預覽請求而不執行
    #[arg(long, global = true)]
    pub dry_run: bool,

    /// 顯示詳細資訊（可疊加 -vv）
    #[arg(short, long, global = true, action = ArgAction::Count)]
    pub verbose: u8,

    /// 靜默模式
    #[arg(short, long, global = true)]
    pub quiet: bool,
}

#[derive(Subcommand)]
pub enum Command {
    /// EDM API 指令
    Edm(EdmCommand),
    /// Surenotify API 指令
    Sn(SnCommand),
    /// 設定管理
    Config(ConfigCommand),
    /// 高階編排指令
    #[command(alias = "x")]
    Helper(HelperCommand),
}
```

**vs gws 的 Discovery 模式：**

| 面向 | gws | nl |
|------|-----|-----|
| API 數量 | 數百個 Google API | 31 個固定 endpoints |
| 指令生成 | 執行期從 Discovery doc 動態產生 | 編譯期靜態定義 |
| 型別檢查 | 無（runtime validation） | 有（compile-time） |
| 自動補全 | 有限 | 完整 shell completion |
| 更新頻率 | API 頻繁異動 | API 穩定 |

### 4.2 Config 系統

```rust
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(flatten)]
    pub profiles: HashMap<String, Profile>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Profile {
    pub edm_api_key: Option<String>,
    pub sn_api_key: Option<String>,
    pub default_format: Option<String>,
}
```

**設定檔路徑：** `~/.config/nl/config.toml`

**載入優先順序：**

```
環境變數 (NL_EDM_API_KEY, NL_SN_API_KEY)
    ↓ 覆蓋
--profile 指定的 Profile section
    ↓ 覆蓋
[default] section
```

**互動式初始化 (`nl config init`)：**

```
1. 提示輸入 EDM API Key（可選跳過）
2. 提示輸入 Surenotify API Key（可選跳過）
3. 選擇預設輸出格式（json/table/yaml/csv）
4. 寫入 ~/.config/nl/config.toml（權限 600）
```

### 4.3 HTTP Client 架構

```rust
pub struct ApiClient {
    http: reqwest::Client,
    edm_limiter: RateLimiter,       // 2 req/sec
    report_limiter: RateLimiter,    // 1 req/10sec
    dry_run: bool,
    verbose: u8,
}

pub struct EdmClient<'a> {
    client: &'a ApiClient,
    api_key: String,
    base_url: String,   // https://api.newsleopard.com
}

pub struct SurenotifyClient<'a> {
    client: &'a ApiClient,
    api_key: String,
    base_url: String,   // https://mail.surenotifyapi.com
}
```

**請求流程：**

```
EdmClient::submit_campaign(request)
    │
    ├─ dry_run? ──yes──→ return NlError::DryRun { method, url, body }
    │
    ├─ rate_limiter.until_ready().await
    │
    ├─ verbose ≥ 1? → tracing::info!(method, url)
    │
    ├─ reqwest::Client::post(url)
    │      .header("x-api-key", &self.api_key)
    │      .json(&request)
    │      .send().await
    │
    ├─ verbose ≥ 2? → tracing::debug!(status, headers, body)
    │
    ├─ status 429 or 5xx? ──yes──→ retry with backoff
    │
    ├─ status 4xx? ──yes──→ parse error code → NlError::Api
    │
    └─ deserialize response → Ok(CampaignSubmitResponse)
```

### 4.4 Error 系統

```rust
#[derive(Error, Debug)]
pub enum NlError {
    /// API 回傳的錯誤（exit code 1）
    #[error("API error {status}: [{code}] {message}")]
    Api {
        status: u16,
        code: Option<u32>,
        message: String,
    },

    /// 參數驗證失敗（exit code 2）
    #[error("Validation error: {0}")]
    Validation(String),

    /// 認證失敗（exit code 3）
    #[error("Authentication error: {0}")]
    Auth(String),

    /// 設定錯誤（exit code 3）
    #[error("Config error: {0}")]
    Config(String),

    /// 網路錯誤（exit code 4）
    #[error("Network error: {0}")]
    Network(String),

    /// Rate limit 耗盡（exit code 4）
    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    /// I/O 錯誤（exit code 5）
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Dry-run 預覽（exit code 0）
    #[error("Dry run")]
    DryRun {
        method: String,
        url: String,
        body: Option<serde_json::Value>,
    },

    /// 204 No Content（exit code 0）
    #[error("No content")]
    NoContent,
}

impl NlError {
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::DryRun { .. } | Self::NoContent => 0,
            Self::Api { .. } => 1,
            Self::Validation(_) => 2,
            Self::Auth(_) | Self::Config(_) => 3,
            Self::Network(_) | Self::RateLimit(_) => 4,
            Self::Io(_) => 5,
        }
    }

    /// 輸出 JSON 格式到 stderr
    pub fn to_json_stderr(&self) {
        let json = serde_json::json!({
            "error": {
                "type": self.error_type(),
                "message": self.to_string(),
                "exit_code": self.exit_code(),
            }
        });
        eprintln!("{}", serde_json::to_string_pretty(&json).unwrap());
    }
}
```

### 4.5 Rate Limiter

```rust
use governor::{Quota, RateLimiter as GovernorLimiter};
use std::num::NonZeroU32;

pub struct RateLimiter {
    limiter: GovernorLimiter</* ... */>,
}

impl RateLimiter {
    /// EDM 一般限流: 2 req/sec
    pub fn edm_general() -> Self {
        let quota = Quota::per_second(NonZeroU32::new(2).unwrap());
        Self { limiter: GovernorLimiter::direct(quota) }
    }

    /// Report 匯出限流: 1 req/10sec
    pub fn report_export() -> Self {
        let quota = Quota::new(
            NonZeroU32::new(1).unwrap(),
            Duration::from_secs(10),
        ).unwrap();
        Self { limiter: GovernorLimiter::direct(quota) }
    }

    pub async fn until_ready(&self) {
        self.limiter.until_ready().await;
    }
}
```

### 4.6 Retry 機制

```rust
use backoff::ExponentialBackoffBuilder;

pub async fn with_retry<F, T>(f: F) -> Result<T, NlError>
where
    F: Fn() -> Future<Output = Result<T, NlError>>,
{
    let backoff = ExponentialBackoffBuilder::default()
        .with_initial_interval(Duration::from_millis(500))
        .with_max_interval(Duration::from_secs(30))
        .with_max_elapsed_time(Some(Duration::from_secs(120)))
        .build();

    backoff::future::retry(backoff, || async {
        match f().await {
            Ok(v) => Ok(v),
            Err(NlError::Network(_)) => Err(backoff::Error::transient(e)),
            Err(NlError::RateLimit(_)) => Err(backoff::Error::transient(e)),
            Err(NlError::Api { status, .. }) if status >= 500 => {
                Err(backoff::Error::transient(e))
            }
            Err(e) => Err(backoff::Error::permanent(e)),
        }
    }).await
}
```

### 4.7 Output Formatting

```rust
#[derive(Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    Json,
    Table,
    Yaml,
    Csv,
}

pub fn format_output<T: Serialize>(
    data: &T,
    format: OutputFormat,
    is_piped: bool,
) -> Result<String, NlError> {
    match format {
        OutputFormat::Json => {
            if is_piped {
                // Compact JSON for piping
                serde_json::to_string(data)
            } else {
                // Pretty JSON for terminal
                serde_json::to_string_pretty(data)
            }
        }
        OutputFormat::Table => {
            // Auto-flatten nested JSON → tabled
            let flat = flatten_json(&serde_json::to_value(data)?);
            render_table(&flat)
        }
        OutputFormat::Yaml => {
            serde_yaml::to_string(data)
        }
        OutputFormat::Csv => {
            render_csv(data)
        }
    }
}
```

| Format | 實作 | 特殊行為 |
|--------|------|---------|
| JSON | `serde_json::to_string_pretty` | piped stdout 時用 compact；`--page-all` 用 NDJSON |
| Table | `tabled` + auto-flatten | terminal 寬度截斷 |
| YAML | `serde_yaml::to_string` | 分頁時用 `---` 分隔 |
| CSV | `csv` crate | 分頁時只第一頁有 header |

### 4.8 Executor 模式

```rust
pub async fn execute(cli: NlCli) -> Result<(), NlError> {
    let config = Config::load(&cli.profile)?;
    let client = ApiClient::new(cli.dry_run, cli.verbose);

    let result: serde_json::Value = match cli.command {
        Command::Edm(edm) => {
            let edm_client = EdmClient::new(&client, &config.edm_api_key()?);
            execute_edm(edm, &edm_client).await?
        }
        Command::Sn(sn) => {
            let sn_client = SurenotifyClient::new(&client, &config.sn_api_key()?);
            execute_sn(sn, &sn_client).await?
        }
        Command::Config(cfg) => execute_config(cfg).await?,
        Command::Helper(helper) => {
            let edm_client = EdmClient::new(&client, &config.edm_api_key()?);
            let sn_client = SurenotifyClient::new(&client, &config.sn_api_key()?);
            execute_helper(helper, &edm_client, &sn_client).await?
        }
    };

    let output = format_output(&result, cli.format, !atty::is(Stream::Stdout))?;
    if !cli.quiet {
        println!("{output}");
    }
    Ok(())
}
```

### 4.9 Helper 工作流程

Helper 指令編排多個 API 呼叫為一個工作流程：

#### campaign-send

```
                    ┌──────────────┐
                    │ balance check│
                    └──────┬───────┘
                           │ balance OK?
                    ┌──────▼───────┐
                    │validate lists│
                    └──────┬───────┘
                           │ lists valid?
                    ┌──────▼───────┐
                    │submit campaign│
                    └──────┬───────┘
                           │ --wait?
                    ┌──────▼───────┐
              yes ← │  wait flag?  │ → no ──→ return SN
                    └──────┬───────┘
                           │
                    ┌──────▼───────┐
                    │ poll status   │ ◄─── with progress bar
                    └──────┬───────┘
                           │ COMPLETE?
                    ┌──────▼───────┐
                    │ fetch metrics │
                    └──────┬───────┘
                           │
                    ┌──────▼───────┐
                    │ display report│
                    └──────────────┘
```

#### import-and-wait

```
┌─────────────┐     ┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│ POST import │ ──→ │ GET upload   │ ──→ │ PUT file to  │ ──→ │ poll import  │
│ /file       │     │ URL          │     │ pre-signed   │     │ status       │
└─────────────┘     └──────────────┘     └──────────────┘     └──────┬───────┘
                                                                     │
                                                              ┌──────▼───────┐
                                                              │ show summary │
                                                              │ + error CSV  │
                                                              └──────────────┘
```

#### report-download

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│ POST export  │ ──→ │ poll link    │ ──→ │ download CSV │
│ /{sn}/export │     │ GET /{sn}/   │     │ to --output  │
│              │     │ link         │     │              │
└──────────────┘     └──────────────┘     └──────────────┘
```

#### domain-setup

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│ POST domain  │ ──→ │ display DNS  │ ──→ │ wait N secs  │ ──→ │ PUT verify   │
│ create       │     │ records table│     │ (optional)   │     │ domain       │
└──────────────┘     └──────────────┘     └──────────────┘     └──────────────┘
```

---

## 5. 型別系統

### 5.1 EDM Types

```rust
// === Campaign ===

#[derive(Serialize, Deserialize)]
pub struct CampaignSubmitRequest {
    pub form: CampaignForm,
    pub content: CampaignContent,
    pub config: CampaignConfig,
}

#[derive(Serialize, Deserialize)]
pub struct CampaignForm {
    pub name: String,
    #[serde(rename = "selectedLists")]
    pub selected_lists: Vec<String>,
    #[serde(rename = "excludeLists", skip_serializing_if = "Vec::is_empty")]
    pub exclude_lists: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct CampaignContent {
    pub subject: String,              // max 150 chars
    #[serde(rename = "fromName")]
    pub from_name: String,            // max 50 chars
    #[serde(rename = "fromAddress")]
    pub from_address: String,
    #[serde(rename = "htmlContent")]
    pub html_content: String,
    #[serde(rename = "footerLang")]
    pub footer_lang: u8,              // 0 = English, 1 = Chinese
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preheader: Option<String>,    // max 60 chars
}

#[derive(Serialize, Deserialize)]
pub struct CampaignConfig {
    pub schedule: ScheduleConfig,
    pub ga: GaConfig,
}

#[derive(Serialize, Deserialize)]
pub struct ScheduleConfig {
    #[serde(rename = "type")]
    pub schedule_type: u8,            // 0 = immediate, 1 = scheduled
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<u8>,
    #[serde(rename = "scheduleDate", skip_serializing_if = "Option::is_none")]
    pub schedule_date: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct GaConfig {
    pub enable: bool,
    #[serde(rename = "ecommerceEnable")]
    pub ecommerce_enable: bool,
    #[serde(rename = "utmCampaign", skip_serializing_if = "Option::is_none")]
    pub utm_campaign: Option<String>,
    #[serde(rename = "utmContent", skip_serializing_if = "Option::is_none")]
    pub utm_content: Option<String>,
}

// === A/B Test ===

#[derive(Serialize, Deserialize)]
pub struct AbTestContent {
    #[serde(rename = "testingOn")]
    pub testing_on: u8,               // 1=subject, 2=sender, 3=content
    pub testing: AbTestConfig,
    // Shared fields (depending on testing_on)
    // ... subject, fromName, fromAddress, htmlContent
    // A/B variant fields
    // ... subjectA/B, fromNameA/B, fromAddressA/B, htmlContentA/B
}

#[derive(Serialize, Deserialize)]
pub struct AbTestConfig {
    pub proportion: u8,               // 0-100
    pub time: u32,
    pub unit: u8,                     // 0 = hours, 1 = days
}

// === Contacts ===

#[derive(Serialize, Deserialize)]
pub struct ContactGroup {
    pub sn: String,
    pub name: String,
    #[serde(rename = "subscribedCnt")]
    pub subscribed_cnt: u64,
    #[serde(rename = "excludeCnt")]
    pub exclude_cnt: u64,
    #[serde(rename = "openedRate")]
    pub opened_rate: f64,
    #[serde(rename = "clickedRate")]
    pub clicked_rate: f64,
    pub status: String,               // GENERAL | PROCESSING
    #[serde(rename = "type")]
    pub group_type: u8,               // 0 = regular, 1 = auto-segment
}

// === Report ===

#[derive(Serialize, Deserialize)]
pub struct CampaignMetrics {
    #[serde(rename = "campaignSn")]
    pub campaign_sn: String,
    pub name: String,
    pub channel: String,
    pub subject: String,
    #[serde(rename = "recipientCnt")]
    pub recipient_cnt: u64,
    pub delivered: u64,
    pub bounced: u64,
    pub opened: u64,
    pub clicked: u64,
    #[serde(rename = "distinctClickCnt")]
    pub distinct_click_cnt: u64,
    pub complained: u64,
    pub unsubscribed: u64,
}
```

### 5.2 Surenotify Types

```rust
// === Email ===

#[derive(Serialize, Deserialize)]
pub struct EmailSendRequest {
    pub subject: String,
    #[serde(rename = "fromAddress")]
    pub from_address: String,
    pub content: String,
    pub recipients: Vec<EmailRecipient>,
    #[serde(rename = "fromName", skip_serializing_if = "Option::is_none")]
    pub from_name: Option<String>,
    #[serde(rename = "unsubscribedLink", skip_serializing_if = "Option::is_none")]
    pub unsubscribed_link: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct EmailRecipient {
    pub name: String,
    pub address: String,
    /// 個人化變數 — 必須放在 variables 物件中，不可作為 top-level 欄位
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize)]
pub struct EmailSendResponse {
    pub id: String,
    pub success: Vec<EmailSuccess>,
    #[serde(default)]
    pub failure: HashMap<String, String>,
}

// === SMS ===

#[derive(Serialize, Deserialize)]
pub struct SmsSendRequest {
    pub content: String,
    pub recipients: Vec<SmsRecipient>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    #[serde(rename = "alive_mins", skip_serializing_if = "Option::is_none")]
    pub alive_mins: Option<u16>,     // 5-480
}

#[derive(Serialize, Deserialize)]
pub struct SmsRecipient {
    pub address: String,             // 數字格式，無 + 或 -
    pub country_code: String,        // e.g. "886"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<HashMap<String, String>>,
}

// === Webhook ===

#[derive(Serialize, Deserialize)]
pub struct WebhookRequest {
    #[serde(rename = "type")]
    pub event_type: u8,              // 3=delivery, 4=open, 5=click, 6=bounce, 7=complaint
    pub url: String,
}

// === Domain ===

#[derive(Serialize, Deserialize)]
pub struct DnsRecord {
    pub name: String,
    pub value: String,
    pub record_type: u8,             // 0=TXT, 1=CNAME
    pub valid: bool,
}
```

### 5.3 變數語法驗證

```rust
/// 檢查 EDM 內容是否誤用 Surenotify 變數語法
pub fn validate_edm_variables(content: &str) -> Vec<String> {
    let mut warnings = Vec::new();
    let sn_pattern = regex::Regex::new(r"\{\{[^}]+\}\}").unwrap();
    if sn_pattern.is_match(content) {
        warnings.push(
            "EDM API 使用 ${FIELD} 變數語法，偵測到 {{...}} 格式（Surenotify 語法）".to_string()
        );
    }
    warnings
}

/// 檢查 Surenotify 內容是否誤用 EDM 變數語法
pub fn validate_sn_variables(content: &str) -> Vec<String> {
    let mut warnings = Vec::new();
    let edm_pattern = regex::Regex::new(r"\$\{[^}]+\}").unwrap();
    if edm_pattern.is_match(content) {
        warnings.push(
            "Surenotify API 使用 {{variable}} 變數語法，偵測到 ${...} 格式（EDM 語法）".to_string()
        );
    }
    warnings
}
```

---

## 6. 測試策略

### 6.1 測試分層

| 層級 | 工具 | 涵蓋範圍 |
|------|------|---------|
| Unit | `#[cfg(test)]` | types serialization、config parsing、variable validation |
| HTTP Mock | `wiremock` | 31 endpoints 的 request/response、error codes、rate limit |
| CLI E2E | `assert_cmd` | 指令解析、exit codes、dry-run、output formats |
| Snapshot | `insta` | Formatter 輸出格式穩定性 |

### 6.2 wiremock 範例

```rust
#[tokio::test]
async fn test_edm_balance() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/balance"))
        .and(header("x-api-key", "test-key"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(json!({"email": 10000, "sms": 500})))
        .mount(&mock_server)
        .await;

    let client = EdmClient::new_with_base_url(
        &ApiClient::new(false, 0),
        "test-key",
        &mock_server.uri(),
    );

    let balance = client.get_balance().await.unwrap();
    assert_eq!(balance.email, 10000);
    assert_eq!(balance.sms, 500);
}
```

### 6.3 assert_cmd 範例

```rust
#[test]
fn test_dry_run_output() {
    Command::cargo_bin("nl")
        .unwrap()
        .args(["edm", "campaign", "submit",
               "--name", "Test",
               "--lists", "SN1",
               "--subject", "Hello",
               "--from-name", "Sender",
               "--from-address", "test@example.com",
               "--html", "<p>Test</p>",
               "--dry-run"])
        .assert()
        .success()
        .stderr(predicates::str::contains("POST"))
        .stderr(predicates::str::contains("/v1/campaign/normal/submit"));
}
```

---

## 7. CI/CD 流程

### 7.1 GitHub Actions

```yaml
name: CI

on: [push, pull_request]

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - run: cargo fmt -- --check
      - run: cargo clippy -- -D warnings
      - run: cargo test

  release:
    if: startsWith(github.ref, 'refs/tags/v')
    needs: check
    strategy:
      matrix:
        include:
          - target: x86_64-unknown-linux-gnu
            os: ubuntu-latest
          - target: aarch64-unknown-linux-gnu
            os: ubuntu-latest
          - target: x86_64-apple-darwin
            os: macos-latest
          - target: aarch64-apple-darwin
            os: macos-latest
          - target: x86_64-pc-windows-msvc
            os: windows-latest
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - run: cargo build --release --target ${{ matrix.target }}
      - uses: softprops/action-gh-release@v2
        with:
          files: target/${{ matrix.target }}/release/nlm*
```

### 7.2 Release 管道

| 管道 | 觸發條件 | 產出 |
|------|---------|------|
| CI | 每次 push / PR | fmt + clippy + test |
| Release | Git tag `v*` | 5 平台 binary → GitHub Release |
| Homebrew | Release 後手動 | 更新 `newsleopard/homebrew-tap` |
| Crates.io | Release 後手動 | `cargo publish` |

---

## 8. 實作順序

### Phase 1: 骨架 + 核心

1. `Cargo.toml` + 專案結構
2. `cli/mod.rs` — 完整 clap derive 指令樹
3. `config/mod.rs` — TOML config + env var + `nl config init`
4. `error.rs` — NlError enum + exit codes
5. `client/mod.rs` — ApiClient + rate limiter
6. `formatter/json.rs` — JSON 輸出
7. `main.rs` — entry point + executor skeleton

### Phase 2: EDM API

8. `types/edm.rs` — 所有 request/response structs
9. `client/edm.rs` — 20 endpoint methods (with validation)
10. `cli/edm/*.rs` — 子指令 args
11. `executor/mod.rs` — EDM 指令分發

### Phase 3: Surenotify API

12. `types/surenotify.rs` — 所有 SN types
13. `client/surenotify.rs` — 11+ endpoint methods
14. `cli/sn/*.rs` — SN 子指令 args

### Phase 4: Formatters + Helpers

15. `formatter/table.rs` + `yaml.rs` + `csv_fmt.rs`
16. `helpers/*.rs` — 4 個 workflow
17. `client/retry.rs` — retry 邏輯

### Phase 5: 測試 + CI

18. Unit tests (serialization, validation, config)
19. `wiremock` HTTP mock tests (all 31 endpoints + error scenarios)
20. `assert_cmd` CLI integration tests (exit codes, dry-run, formats)
21. `insta` snapshot tests (formatter output)
22. GitHub Actions: fmt + clippy + test + 5-platform cross-compile release

---

## 9. 設計決策摘要

| 決策 | 選擇 | 理由 (vs GWS 做法) |
|------|------|-------------------|
| 靜態 vs 動態指令 | **靜態 clap derive** | API 面固定 31 endpoints（gws 用動態因為 Google 有數百個 API） |
| TLS | **rustls** | 跨平台編譯不需 OpenSSL（同 gws） |
| Config 格式 | **TOML** | Rust 生態標準（gws 用 JSON，但 TOML 更適合設定檔） |
| 認證 | **x-api-key header** | 比 gws 的 OAuth 簡單得多，不需 keyring 加密 |
| Rate limiter | **governor** | Token bucket（同 gws 概念，但用 crate 而非手寫） |
| Output | **4 格式 + NDJSON 分頁** | 完全參考 gws 的 json/table/yaml/csv 模式 |
| Error | **thiserror enum + exit codes** | 直接參考 gws 的 GwsError 模式 |
| Helper | **orchestration commands** | 參考 gws 的 Helper trait，但簡化為直接函數 |
| Retry | **backoff crate** | 標準 exponential backoff with jitter |
| 進度顯示 | **indicatif** | Helper 指令的輪詢進度條 |
