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

/// Get Claude Code OAuth token from macOS Keychain.
pub fn get_claude_oauth_token() -> Option<String> {
    let output = Command::new("security")
        .args([
            "find-generic-password",
            "-s",
            "Claude Code-credentials",
            "-w",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8(output.stdout).ok()?;
    let creds: serde_json::Value = serde_json::from_str(stdout.trim()).ok()?;
    creds
        .get("claudeAiOauth")?
        .get("accessToken")?
        .as_str()
        .map(|s| s.to_string())
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
