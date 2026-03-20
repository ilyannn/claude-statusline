use crate::cache;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;

const USAGE_CACHE_MAX_AGE: u64 = 300; // 5 minutes

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UsageData {
    pub five_hour: f64,
    #[serde(default)]
    pub five_hour_resets: String,
    pub seven_day: f64,
    #[serde(default)]
    pub seven_day_resets: String,
}

fn usage_cache_path(cache_dir: &Path) -> PathBuf {
    cache_dir.join("usage-cache")
}

/// Get Claude config directory: $CLAUDE_CONFIG_DIR or ~/.claude.
fn get_claude_config_dir() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("CLAUDE_CONFIG_DIR") {
        if !dir.is_empty() {
            return Some(PathBuf::from(dir));
        }
    }
    std::env::var("HOME")
        .ok()
        .map(|h| PathBuf::from(h).join(".claude"))
}

/// Read OAuth token from ~/.claude/.credentials.json.
fn read_credentials_file() -> Option<String> {
    let cred_path = get_claude_config_dir()?.join(".credentials.json");
    let content = std::fs::read_to_string(cred_path).ok()?;
    parse_oauth_token(&content)
}

/// Get Claude Code OAuth token from macOS Keychain, falling back to credentials file.
pub fn get_claude_oauth_token() -> Option<String> {
    // Try macOS Keychain first
    if let Ok(output) = Command::new("security")
        .args([
            "find-generic-password",
            "-s",
            "Claude Code-credentials",
            "-w",
        ])
        .output()
    {
        if output.status.success() {
            if let Ok(stdout) = String::from_utf8(output.stdout) {
                if let Some(token) = parse_oauth_token(&stdout) {
                    return Some(token);
                }
            }
        }
    }

    // Fall back to credentials file (works on Linux/non-macOS)
    read_credentials_file()
}

/// Extract OAuth access token from keychain JSON. Extracted for testability.
pub fn parse_oauth_token(json_str: &str) -> Option<String> {
    let creds: serde_json::Value = serde_json::from_str(json_str.trim()).ok()?;
    let token = creds.get("claudeAiOauth")?.get("accessToken")?.as_str()?;
    if token.is_empty() {
        return None;
    }
    Some(token.to_string())
}

/// Get Claude.ai usage stats. Uses cache to avoid frequent API calls.
pub fn get_claude_usage(cache_dir: &Path) -> Option<UsageData> {
    let cache_path = usage_cache_path(cache_dir);

    // Check cache
    if let Some(content) = cache::read_cache(&cache_path, USAGE_CACHE_MAX_AGE) {
        if let Ok(data) = serde_json::from_str::<UsageData>(&content) {
            return Some(data);
        }
    }

    // Get OAuth token
    let token = get_claude_oauth_token()?;

    // Fetch in background via fork
    #[cfg(unix)]
    {
        let cache_path_bg = cache_path.clone();
        crate::fork_background(move || {
            fetch_and_cache_usage(&token, &cache_path_bg);
        });
    }

    // Return stale cached value if exists
    if let Ok(content) = std::fs::read_to_string(&cache_path) {
        serde_json::from_str(&content).ok()
    } else {
        None
    }
}

fn fetch_and_cache_usage(token: &str, cache_path: &Path) {
    let output = match Command::new("curl")
        .args([
            "-s",
            "-H",
            &format!("Authorization: Bearer {}", token),
            "-H",
            "Content-Type: application/json",
            "-H",
            "User-Agent: claude-code/2.1.5",
            "-H",
            "anthropic-beta: oauth-2025-04-20",
            "--max-time",
            "10",
            "https://api.anthropic.com/api/oauth/usage",
        ])
        .output()
    {
        Ok(o) => o,
        Err(_) => return,
    };

    let body = match String::from_utf8(output.stdout) {
        Ok(s) => s,
        Err(_) => return,
    };

    if let Ok(data) = serde_json::from_str::<serde_json::Value>(&body) {
        let usage = UsageData {
            five_hour: data
                .get("five_hour")
                .and_then(|v| v.get("utilization"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            five_hour_resets: data
                .get("five_hour")
                .and_then(|v| v.get("resets_at"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            seven_day: data
                .get("seven_day")
                .and_then(|v| v.get("utilization"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            seven_day_resets: data
                .get("seven_day")
                .and_then(|v| v.get("resets_at"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        };

        if let Ok(json) = serde_json::to_string(&usage) {
            cache::write_cache(cache_path, &json);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usage_cache_hit() {
        let dir = tempfile::tempdir().unwrap();
        let cache_path = dir.path().join("usage-cache");
        let data = UsageData {
            five_hour: 25.0,
            five_hour_resets: String::new(),
            seven_day: 60.0,
            seven_day_resets: String::new(),
        };
        std::fs::write(&cache_path, serde_json::to_string(&data).unwrap()).unwrap();

        let result = get_claude_usage(dir.path());
        assert!(result.is_some());
        let r = result.unwrap();
        assert_eq!(r.five_hour as u32, 25);
        assert_eq!(r.seven_day as u32, 60);
    }

    #[test]
    fn test_usage_cache_with_reset_times() {
        let dir = tempfile::tempdir().unwrap();
        let cache_path = dir.path().join("usage-cache");
        let data = UsageData {
            five_hour: 25.0,
            five_hour_resets: "2025-01-01T07:00:00Z".to_string(),
            seven_day: 60.0,
            seven_day_resets: "2025-01-07T00:00:00Z".to_string(),
        };
        std::fs::write(&cache_path, serde_json::to_string(&data).unwrap()).unwrap();

        let result = get_claude_usage(dir.path());
        assert!(result.is_some());
        let r = result.unwrap();
        assert_eq!(r.five_hour_resets, "2025-01-01T07:00:00Z");
    }

    // OAuth token parsing tests
    #[test]
    fn test_token_extraction() {
        let json = r#"{"claudeAiOauth":{"accessToken":"test-token-123"}}"#;
        assert_eq!(parse_oauth_token(json), Some("test-token-123".to_string()));
    }

    #[test]
    fn test_malformed_json() {
        assert_eq!(parse_oauth_token("not json"), None);
    }

    #[test]
    fn test_missing_oauth_key() {
        let json = r#"{"otherKey":"value"}"#;
        assert_eq!(parse_oauth_token(json), None);
    }

    #[test]
    fn test_missing_access_token() {
        let json = r#"{"claudeAiOauth":{}}"#;
        assert_eq!(parse_oauth_token(json), None);
    }

    #[test]
    fn test_usage_no_cache_no_creds() {
        let dir = tempfile::tempdir().unwrap();
        // No cache file, no keychain — should return None
        let result = get_claude_usage(dir.path());
        assert!(result.is_none());
    }

    #[test]
    fn test_read_credentials_file() {
        let dir = tempfile::tempdir().unwrap();
        let cred_path = dir.path().join(".credentials.json");
        std::fs::write(
            &cred_path,
            r#"{"claudeAiOauth":{"accessToken":"file-token-456"}}"#,
        )
        .unwrap();

        // Point CLAUDE_CONFIG_DIR at the temp dir
        std::env::set_var("CLAUDE_CONFIG_DIR", dir.path().as_os_str());
        let result = read_credentials_file();
        std::env::remove_var("CLAUDE_CONFIG_DIR");

        assert_eq!(result, Some("file-token-456".to_string()));
    }

    #[test]
    fn test_read_credentials_file_missing() {
        let dir = tempfile::tempdir().unwrap();
        std::env::set_var("CLAUDE_CONFIG_DIR", dir.path().as_os_str());
        let result = read_credentials_file();
        std::env::remove_var("CLAUDE_CONFIG_DIR");

        assert_eq!(result, None);
    }

    #[test]
    fn test_read_credentials_file_malformed() {
        let dir = tempfile::tempdir().unwrap();
        let cred_path = dir.path().join(".credentials.json");
        std::fs::write(&cred_path, "not json").unwrap();

        std::env::set_var("CLAUDE_CONFIG_DIR", dir.path().as_os_str());
        let result = read_credentials_file();
        std::env::remove_var("CLAUDE_CONFIG_DIR");

        assert_eq!(result, None);
    }

    #[test]
    fn test_empty_access_token() {
        let json = r#"{"claudeAiOauth":{"accessToken":""}}"#;
        assert_eq!(parse_oauth_token(json), None);
    }

    #[test]
    fn test_null_access_token() {
        let json = r#"{"claudeAiOauth":{"accessToken":null}}"#;
        assert_eq!(parse_oauth_token(json), None);
    }

    #[test]
    fn test_token_with_whitespace() {
        let json = "  {\"claudeAiOauth\":{\"accessToken\":\"tok\"}}  \n";
        assert_eq!(parse_oauth_token(json), Some("tok".to_string()));
    }

    #[test]
    fn test_get_claude_config_dir_env_override() {
        let dir = tempfile::tempdir().unwrap();
        std::env::set_var("CLAUDE_CONFIG_DIR", dir.path().as_os_str());
        let result = get_claude_config_dir();
        std::env::remove_var("CLAUDE_CONFIG_DIR");

        assert_eq!(result, Some(dir.path().to_path_buf()));
    }

    #[test]
    fn test_get_claude_config_dir_empty_env() {
        std::env::set_var("CLAUDE_CONFIG_DIR", "");
        let result = get_claude_config_dir();
        std::env::remove_var("CLAUDE_CONFIG_DIR");

        // Empty env should fall back to $HOME/.claude
        let expected = std::env::var("HOME")
            .map(|h| PathBuf::from(h).join(".claude"))
            .ok();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_get_claude_config_dir_default() {
        std::env::remove_var("CLAUDE_CONFIG_DIR");
        let result = get_claude_config_dir();

        let expected = std::env::var("HOME")
            .map(|h| PathBuf::from(h).join(".claude"))
            .ok();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_read_credentials_file_empty_token() {
        let dir = tempfile::tempdir().unwrap();
        let cred_path = dir.path().join(".credentials.json");
        std::fs::write(&cred_path, r#"{"claudeAiOauth":{"accessToken":""}}"#).unwrap();

        std::env::set_var("CLAUDE_CONFIG_DIR", dir.path().as_os_str());
        let result = read_credentials_file();
        std::env::remove_var("CLAUDE_CONFIG_DIR");

        assert_eq!(result, None);
    }

    #[test]
    fn test_read_credentials_file_null_token() {
        let dir = tempfile::tempdir().unwrap();
        let cred_path = dir.path().join(".credentials.json");
        std::fs::write(&cred_path, r#"{"claudeAiOauth":{"accessToken":null}}"#).unwrap();

        std::env::set_var("CLAUDE_CONFIG_DIR", dir.path().as_os_str());
        let result = read_credentials_file();
        std::env::remove_var("CLAUDE_CONFIG_DIR");

        assert_eq!(result, None);
    }

    #[test]
    fn test_usage_data_serialization() {
        let data = UsageData {
            five_hour: 42.5,
            five_hour_resets: "2025-01-01T07:00:00Z".to_string(),
            seven_day: 80.0,
            seven_day_resets: "2025-01-07T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&data).unwrap();
        let parsed: UsageData = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.five_hour, 42.5);
        assert_eq!(parsed.seven_day, 80.0);
    }
}
