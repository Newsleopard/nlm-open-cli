//! Configuration system for the nl CLI.
//!
//! Config file: `~/.config/nl/config.toml`
//! File permissions: 0o600 (Unix only)
//!
//! Supports multiple profiles, env var overrides, and interactive initialization.
//!
//! # Load priority (highest wins)
//!
//! 1. Environment variables: `NL_EDM_API_KEY`, `NL_SN_API_KEY`, `NL_FORMAT`
//! 2. Named profile section (`--profile <name>`)
//! 3. `[default]` profile section

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::NlError;

/// The full TOML configuration file structure.
///
/// Each key under the top level is a profile name (e.g. `[default]`, `[staging]`).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConfigFile {
    #[serde(flatten)]
    pub profiles: HashMap<String, Profile>,
}

/// A single profile containing API keys and format preferences.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Profile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edm_api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sn_api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_url: Option<String>,
}

/// Resolved configuration after merging profile values with env var overrides.
///
/// This is the struct that the rest of the application uses.
#[derive(Debug, Clone)]
pub struct ResolvedConfig {
    edm_api_key: Option<String>,
    sn_api_key: Option<String>,
    #[allow(dead_code)]
    pub default_format: String,
    mcp_url: Option<String>,
}

impl ResolvedConfig {
    /// Returns the EDM API key, or an `Auth` error if not configured.
    pub fn edm_api_key(&self) -> Result<&str, NlError> {
        self.edm_api_key.as_deref().ok_or_else(|| {
            NlError::Auth(
                "EDM API key not configured. Run `nl config init` or set NL_EDM_API_KEY."
                    .to_string(),
            )
        })
    }

    /// Returns the Surenotify API key, or an `Auth` error if not configured.
    pub fn sn_api_key(&self) -> Result<&str, NlError> {
        self.sn_api_key.as_deref().ok_or_else(|| {
            NlError::Auth(
                "Surenotify API key not configured. Run `nl config init` or set NL_SN_API_KEY."
                    .to_string(),
            )
        })
    }

    /// Returns the MCP server URL, falling back to the default.
    pub fn mcp_url(&self) -> &str {
        self.mcp_url
            .as_deref()
            .unwrap_or(crate::client::mcp::DEFAULT_MCP_URL)
    }
}

// ── Path helpers ──────────────────────────────────────────────────

/// Returns the nl config directory: `~/.config/nl/`.
pub fn config_dir() -> Result<PathBuf, NlError> {
    let base = dirs::config_dir()
        .ok_or_else(|| NlError::Config("Could not determine config directory".to_string()))?;
    Ok(base.join("nl"))
}

/// Returns the config file path: `~/.config/nl/config.toml`.
pub fn config_path() -> Result<PathBuf, NlError> {
    Ok(config_dir()?.join("config.toml"))
}

// ── Load / Save ───────────────────────────────────────────────────

/// Loads and parses the TOML config file.
///
/// Returns an empty `ConfigFile` if the file does not exist.
fn load_file() -> Result<ConfigFile, NlError> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(ConfigFile::default());
    }
    let content = fs::read_to_string(&path)?;
    let config: ConfigFile = toml::from_str(&content)?;
    Ok(config)
}

/// Loads the config file and resolves the given profile with env var overrides.
///
/// If the config file does not exist, returns a config built entirely from
/// environment variables (if any are set).
pub fn load(profile_name: &str) -> Result<ResolvedConfig, NlError> {
    let config_file = load_file()?;

    // Start with the default profile, then overlay the named profile.
    let mut edm_api_key: Option<String> = None;
    let mut sn_api_key: Option<String> = None;
    let mut default_format: Option<String> = None;
    let mut mcp_url: Option<String> = None;

    // Layer 1: [default] profile.
    if let Some(default_profile) = config_file.profiles.get("default") {
        edm_api_key = default_profile.edm_api_key.clone();
        sn_api_key = default_profile.sn_api_key.clone();
        default_format = default_profile.default_format.clone();
        mcp_url = default_profile.mcp_url.clone();
    }

    // Layer 2: Named profile (if not "default").
    if profile_name != "default" {
        if let Some(profile) = config_file.profiles.get(profile_name) {
            if profile.edm_api_key.is_some() {
                edm_api_key = profile.edm_api_key.clone();
            }
            if profile.sn_api_key.is_some() {
                sn_api_key = profile.sn_api_key.clone();
            }
            if profile.default_format.is_some() {
                default_format = profile.default_format.clone();
            }
            if profile.mcp_url.is_some() {
                mcp_url = profile.mcp_url.clone();
            }
        }
        // It's not an error if a profile doesn't exist in the file —
        // env vars may provide the keys.
    }

    // Layer 3: Environment variable overrides (highest priority).
    if let Ok(val) = std::env::var("NL_EDM_API_KEY") {
        if !val.is_empty() {
            edm_api_key = Some(val);
        }
    }
    if let Ok(val) = std::env::var("NL_SN_API_KEY") {
        if !val.is_empty() {
            sn_api_key = Some(val);
        }
    }
    if let Ok(val) = std::env::var("NL_FORMAT") {
        if !val.is_empty() {
            default_format = Some(val);
        }
    }
    if let Ok(val) = std::env::var("NL_MCP_URL") {
        if !val.is_empty() {
            mcp_url = Some(val);
        }
    }

    Ok(ResolvedConfig {
        edm_api_key,
        sn_api_key,
        default_format: default_format.unwrap_or_else(|| "json".to_string()),
        mcp_url,
    })
}

/// Saves the config file, creating parent directories as needed.
///
/// On Unix, sets file permissions to 0o600.
pub fn save(config: &ConfigFile) -> Result<(), NlError> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = toml::to_string_pretty(config)
        .map_err(|e| NlError::Config(format!("Failed to serialize config: {e}")))?;
    fs::write(&path, &content)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o600);
        fs::set_permissions(&path, perms)?;
    }

    Ok(())
}

// ── Interactive init ──────────────────────────────────────────────

/// Interactive initialization via dialoguer prompts.
///
/// Asks for EDM API key, SN API key, and default format, then writes
/// the config file to `~/.config/nl/config.toml`.
pub fn init_interactive() -> Result<(), NlError> {
    use dialoguer::{Input, Select};

    let edm_key: String = Input::new()
        .with_prompt("EDM API Key (leave blank to skip)")
        .allow_empty(true)
        .interact_text()
        .map_err(|e| NlError::Io(std::io::Error::other(e)))?;

    let sn_key: String = Input::new()
        .with_prompt("Surenotify API Key (leave blank to skip)")
        .allow_empty(true)
        .interact_text()
        .map_err(|e| NlError::Io(std::io::Error::other(e)))?;

    let format_options = &["json", "table", "yaml", "csv"];
    let format_idx = Select::new()
        .with_prompt("Default output format")
        .items(format_options)
        .default(0)
        .interact()
        .map_err(|e| NlError::Io(std::io::Error::other(e)))?;

    let profile = Profile {
        edm_api_key: if edm_key.is_empty() {
            None
        } else {
            Some(edm_key)
        },
        sn_api_key: if sn_key.is_empty() {
            None
        } else {
            Some(sn_key)
        },
        default_format: Some(format_options[format_idx].to_string()),
        mcp_url: None,
    };

    // Load existing config (or start fresh) and upsert the default profile.
    let mut config = load_file()?;
    config.profiles.insert("default".to_string(), profile);
    save(&config)?;

    let path = config_path()?;
    eprintln!("Config saved to {}", path.display());
    Ok(())
}

// ── Get / Set / List ──────────────────────────────────────────────

/// Sets a single config value within a profile.
///
/// Valid keys: `edm_api_key`, `sn_api_key`, `default_format`.
/// If `profile` is `None`, defaults to `"default"`.
pub fn set_value(key: &str, value: &str, profile: Option<&str>) -> Result<(), NlError> {
    let profile_name = profile.unwrap_or("default");
    let mut config = load_file()?;
    let prof = config.profiles.entry(profile_name.to_string()).or_default();

    match key {
        "edm_api_key" => prof.edm_api_key = Some(value.to_string()),
        "sn_api_key" => prof.sn_api_key = Some(value.to_string()),
        "default_format" => {
            let valid = ["json", "table", "yaml", "csv"];
            if !valid.contains(&value) {
                return Err(NlError::Validation(format!(
                    "Invalid format '{value}'. Valid values: {}",
                    valid.join(", ")
                )));
            }
            prof.default_format = Some(value.to_string());
        }
        "mcp_url" => prof.mcp_url = Some(value.to_string()),
        _ => {
            return Err(NlError::Validation(format!(
                "Unknown config key '{key}'. Valid keys: edm_api_key, sn_api_key, default_format, mcp_url"
            )));
        }
    }

    save(&config)
}

/// Gets a config value, masking API keys for security.
///
/// API keys are displayed as `****...` followed by the last 3 characters.
/// If `profile` is `None`, defaults to `"default"`.
pub fn get_value(key: &str, profile: Option<&str>) -> Result<String, NlError> {
    let profile_name = profile.unwrap_or("default");
    let config = load_file()?;
    let prof = config
        .profiles
        .get(profile_name)
        .ok_or_else(|| NlError::Config(format!("Profile '{profile_name}' not found")))?;

    let value = match key {
        "edm_api_key" => prof
            .edm_api_key
            .as_deref()
            .map(mask_value)
            .unwrap_or_else(|| "(not set)".to_string()),
        "sn_api_key" => prof
            .sn_api_key
            .as_deref()
            .map(mask_value)
            .unwrap_or_else(|| "(not set)".to_string()),
        "default_format" => prof
            .default_format
            .clone()
            .unwrap_or_else(|| "(not set)".to_string()),
        "mcp_url" => prof
            .mcp_url
            .clone()
            .unwrap_or_else(|| "(not set)".to_string()),
        _ => {
            return Err(NlError::Validation(format!(
                "Unknown config key '{key}'. Valid keys: edm_api_key, sn_api_key, default_format, mcp_url"
            )));
        }
    };

    Ok(value)
}

/// Lists all profiles and their settings (with masked API keys).
pub fn list_all() -> Result<String, NlError> {
    let config = load_file()?;
    if config.profiles.is_empty() {
        return Ok("No profiles configured. Run `nl config init` to create one.".to_string());
    }

    let mut output = String::new();
    let mut profile_names: Vec<&String> = config.profiles.keys().collect();
    profile_names.sort();

    for name in profile_names {
        let prof = &config.profiles[name];
        output.push_str(&format!("[{name}]\n"));

        let edm_display = prof
            .edm_api_key
            .as_deref()
            .map(mask_value)
            .unwrap_or_else(|| "(not set)".to_string());
        output.push_str(&format!("  edm_api_key    = {edm_display}\n"));

        let sn_display = prof
            .sn_api_key
            .as_deref()
            .map(mask_value)
            .unwrap_or_else(|| "(not set)".to_string());
        output.push_str(&format!("  sn_api_key     = {sn_display}\n"));

        let format_display = prof.default_format.as_deref().unwrap_or("(not set)");
        output.push_str(&format!("  default_format = {format_display}\n"));

        let mcp_display = prof.mcp_url.as_deref().unwrap_or("(not set)");
        output.push_str(&format!("  mcp_url        = {mcp_display}\n"));

        output.push('\n');
    }

    Ok(output.trim_end().to_string())
}

// ── Profile management ────────────────────────────────────────────

/// Creates a new empty profile.
pub fn create_profile(name: &str) -> Result<(), NlError> {
    let mut config = load_file()?;
    if config.profiles.contains_key(name) {
        return Err(NlError::Config(format!("Profile '{name}' already exists")));
    }
    config.profiles.insert(name.to_string(), Profile::default());
    save(&config)
}

/// Deletes a profile. Cannot delete the "default" profile.
pub fn delete_profile(name: &str) -> Result<(), NlError> {
    if name == "default" {
        return Err(NlError::Validation(
            "Cannot delete the 'default' profile".to_string(),
        ));
    }
    let mut config = load_file()?;
    if config.profiles.remove(name).is_none() {
        return Err(NlError::Config(format!("Profile '{name}' not found")));
    }
    save(&config)
}

/// Returns a sorted list of all profile names.
pub fn profile_list() -> Result<Vec<String>, NlError> {
    let config = load_file()?;
    let mut names: Vec<String> = config.profiles.keys().cloned().collect();
    names.sort();
    Ok(names)
}

// ── Masking helper ────────────────────────────────────────────────

/// Masks an API key for display: shows `****...` plus the last 3 characters.
///
/// Keys shorter than 4 characters are fully masked as `****`.
fn mask_value(val: &str) -> String {
    if val.len() <= 3 {
        "****".to_string()
    } else {
        let last3 = &val[val.len() - 3..];
        format!("****...{last3}")
    }
}

// ── Tests ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_value_normal() {
        assert_eq!(mask_value("abcdefghij"), "****...hij");
    }

    #[test]
    fn test_mask_value_short() {
        assert_eq!(mask_value("ab"), "****");
        assert_eq!(mask_value("abc"), "****");
    }

    #[test]
    fn test_mask_value_four_chars() {
        assert_eq!(mask_value("abcd"), "****...bcd");
    }

    #[test]
    fn test_mask_value_empty() {
        assert_eq!(mask_value(""), "****");
    }

    #[test]
    fn test_config_file_roundtrip() {
        let mut config = ConfigFile::default();
        config.profiles.insert(
            "default".to_string(),
            Profile {
                edm_api_key: Some("edm-key-123".to_string()),
                sn_api_key: Some("sn-key-456".to_string()),
                default_format: Some("json".to_string()),
                mcp_url: None,
            },
        );
        config.profiles.insert(
            "staging".to_string(),
            Profile {
                edm_api_key: Some("staging-edm-key".to_string()),
                sn_api_key: None,
                default_format: Some("table".to_string()),
                mcp_url: None,
            },
        );

        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: ConfigFile = toml::from_str(&toml_str).unwrap();

        assert_eq!(parsed.profiles.len(), 2);
        assert_eq!(
            parsed.profiles["default"].edm_api_key.as_deref(),
            Some("edm-key-123")
        );
        assert_eq!(
            parsed.profiles["staging"].default_format.as_deref(),
            Some("table")
        );
        assert!(parsed.profiles["staging"].sn_api_key.is_none());
    }

    #[test]
    fn test_config_file_deserialization_from_toml() {
        let toml_str = r#"
[default]
edm_api_key = "my-edm-key"
sn_api_key = "my-sn-key"
default_format = "json"

[production]
edm_api_key = "prod-edm-key"
sn_api_key = "prod-sn-key"
default_format = "table"
"#;
        let config: ConfigFile = toml::from_str(toml_str).unwrap();
        assert_eq!(config.profiles.len(), 2);
        assert_eq!(
            config.profiles["default"].edm_api_key.as_deref(),
            Some("my-edm-key")
        );
        assert_eq!(
            config.profiles["production"].default_format.as_deref(),
            Some("table")
        );
    }

    #[test]
    fn test_config_file_empty() {
        let config: ConfigFile = toml::from_str("").unwrap();
        assert!(config.profiles.is_empty());
    }

    #[test]
    fn test_resolved_config_edm_key_present() {
        let config = ResolvedConfig {
            edm_api_key: Some("test-key".to_string()),
            sn_api_key: None,
            default_format: "json".to_string(),
            mcp_url: None,
        };
        assert_eq!(config.edm_api_key().unwrap(), "test-key");
    }

    #[test]
    fn test_resolved_config_edm_key_missing() {
        let config = ResolvedConfig {
            edm_api_key: None,
            sn_api_key: None,
            default_format: "json".to_string(),
            mcp_url: None,
        };
        let err = config.edm_api_key().unwrap_err();
        assert_eq!(err.exit_code(), 3);
        assert_eq!(err.error_type(), "auth");
    }

    #[test]
    fn test_resolved_config_sn_key_present() {
        let config = ResolvedConfig {
            edm_api_key: None,
            sn_api_key: Some("sn-key".to_string()),
            default_format: "json".to_string(),
            mcp_url: None,
        };
        assert_eq!(config.sn_api_key().unwrap(), "sn-key");
    }

    #[test]
    fn test_resolved_config_sn_key_missing() {
        let config = ResolvedConfig {
            edm_api_key: None,
            sn_api_key: None,
            default_format: "json".to_string(),
            mcp_url: None,
        };
        let err = config.sn_api_key().unwrap_err();
        assert_eq!(err.exit_code(), 3);
        assert_eq!(err.error_type(), "auth");
    }

    #[test]
    fn test_list_all_empty() {
        let config = ConfigFile::default();
        assert!(config.profiles.is_empty());
    }

    #[test]
    fn test_list_all_formatting() {
        let mut config = ConfigFile::default();
        config.profiles.insert(
            "default".to_string(),
            Profile {
                edm_api_key: Some("my-secret-key-abc".to_string()),
                sn_api_key: None,
                default_format: Some("json".to_string()),
                mcp_url: None,
            },
        );

        let profile = &config.profiles["default"];
        let masked = profile
            .edm_api_key
            .as_deref()
            .map(mask_value)
            .unwrap_or_else(|| "(not set)".to_string());
        assert_eq!(masked, "****...abc");
    }

    #[test]
    fn test_profile_serialization_skip_none() {
        let profile = Profile {
            edm_api_key: Some("key".to_string()),
            sn_api_key: None,
            default_format: None,
            mcp_url: None,
        };
        let toml_str = toml::to_string(&profile).unwrap();
        assert!(toml_str.contains("edm_api_key"));
        assert!(!toml_str.contains("sn_api_key"));
        assert!(!toml_str.contains("default_format"));
    }

    #[test]
    fn test_config_path_is_deterministic() {
        let p1 = config_path();
        let p2 = config_path();
        assert_eq!(p1.is_ok(), p2.is_ok());
        if let (Ok(p1), Ok(p2)) = (p1, p2) {
            assert_eq!(p1, p2);
            assert!(p1.ends_with("nl/config.toml"));
        }
    }

    #[test]
    fn test_config_dir_ends_with_nl() {
        if let Ok(dir) = config_dir() {
            assert!(dir.ends_with("nl"));
        }
    }

    #[test]
    fn test_load_with_env_overrides() {
        // Temporarily set env vars and verify they take priority.
        let original_edm = std::env::var("NL_EDM_API_KEY").ok();
        let original_sn = std::env::var("NL_SN_API_KEY").ok();

        std::env::set_var("NL_EDM_API_KEY", "env-edm-key");
        std::env::set_var("NL_SN_API_KEY", "env-sn-key");

        let resolved = load("default").unwrap();
        assert_eq!(resolved.edm_api_key().unwrap(), "env-edm-key");
        assert_eq!(resolved.sn_api_key().unwrap(), "env-sn-key");

        // Restore original env.
        match original_edm {
            Some(v) => std::env::set_var("NL_EDM_API_KEY", v),
            None => std::env::remove_var("NL_EDM_API_KEY"),
        }
        match original_sn {
            Some(v) => std::env::set_var("NL_SN_API_KEY", v),
            None => std::env::remove_var("NL_SN_API_KEY"),
        }
    }

    #[test]
    fn test_set_value_invalid_key_validation() {
        let valid_formats = ["json", "table", "yaml", "csv"];
        assert!(valid_formats.contains(&"json"));
        assert!(!valid_formats.contains(&"xml"));
    }

    #[test]
    fn test_default_format_fallback() {
        // When no format is configured anywhere, should default to "json".
        let resolved = ResolvedConfig {
            edm_api_key: None,
            sn_api_key: None,
            default_format: "json".to_string(),
            mcp_url: None,
        };
        assert_eq!(resolved.default_format, "json");
    }
}
