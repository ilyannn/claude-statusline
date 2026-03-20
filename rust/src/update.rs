use crate::cache;
use std::path::{Path, PathBuf};
use std::process::Command;

const CACHE_MAX_AGE: u64 = 3600; // 1 hour

fn update_cache_path(cache_dir: &Path) -> PathBuf {
    cache_dir.join("update-check")
}

/// Check if a newer version is available. Uses cache to avoid frequent npm calls.
pub fn check_for_update(current_version: &str, cache_dir: &Path) -> Option<String> {
    if current_version.is_empty() {
        return None;
    }

    let cache_path = update_cache_path(cache_dir);

    // Check cache
    if let Some(content) = cache::read_cache(&cache_path, CACHE_MAX_AGE) {
        return parse_cache_content(content.trim(), current_version);
    }

    // Cache expired or missing - check in background
    #[cfg(unix)]
    {
        let cache_path_bg = cache_path.clone();
        let current = current_version.to_string();
        crate::fork_background(move || {
            fetch_and_cache_version(&current, &cache_path_bg);
        });
    }

    // Return stale cached value if exists
    if let Ok(content) = std::fs::read_to_string(&cache_path) {
        parse_cache_content(content.trim(), current_version)
    } else {
        None
    }
}

fn parse_cache_content(content: &str, current_version: &str) -> Option<String> {
    if let Some(version) = content.strip_prefix("update:") {
        if !version.is_empty() && version != current_version {
            return Some(version.to_string());
        }
    }
    None
}

fn fetch_and_cache_version(current_version: &str, cache_path: &Path) {
    let output = match Command::new("npm")
        .args(["view", "@anthropic-ai/claude-code", "version"])
        .output()
    {
        Ok(o) => o,
        Err(_) => {
            cache::write_cache(cache_path, "error");
            return;
        }
    };

    let latest = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !latest.is_empty() && latest != current_version {
        cache::write_cache(cache_path, &format!("update:{}", latest));
    } else {
        cache::write_cache(cache_path, "current");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_returns_update_available() {
        let dir = tempfile::tempdir().unwrap();
        let cache_path = dir.path().join("update-check");
        std::fs::write(&cache_path, "update:2.0.0").unwrap();

        let result = check_for_update("1.0.0", dir.path());
        assert_eq!(result.as_deref(), Some("2.0.0"));
    }

    #[test]
    fn test_cache_returns_none_when_already_on_cached_version() {
        let dir = tempfile::tempdir().unwrap();
        let cache_path = dir.path().join("update-check");
        std::fs::write(&cache_path, "update:1.0.0").unwrap();

        let result = check_for_update("1.0.0", dir.path());
        assert!(result.is_none());
    }

    #[test]
    fn test_cache_returns_current() {
        let dir = tempfile::tempdir().unwrap();
        let cache_path = dir.path().join("update-check");
        std::fs::write(&cache_path, "current").unwrap();

        let result = check_for_update("1.0.0", dir.path());
        assert!(result.is_none());
    }

    #[test]
    fn test_cache_with_error_value() {
        let dir = tempfile::tempdir().unwrap();
        let cache_path = dir.path().join("update-check");
        std::fs::write(&cache_path, "error").unwrap();

        let result = check_for_update("1.0.0", dir.path());
        assert!(result.is_none());
    }

    #[test]
    fn test_cache_with_empty_update_version() {
        let dir = tempfile::tempdir().unwrap();
        let cache_path = dir.path().join("update-check");
        std::fs::write(&cache_path, "update:").unwrap();

        let result = check_for_update("1.0.0", dir.path());
        assert!(result.is_none());
    }

    #[test]
    fn test_cache_file_corrupted() {
        let dir = tempfile::tempdir().unwrap();
        let cache_path = dir.path().join("update-check");
        std::fs::write(&cache_path, "garbage data").unwrap();

        let result = check_for_update("1.0.0", dir.path());
        assert!(result.is_none());
    }

    #[test]
    fn test_empty_version_skips_check() {
        let dir = tempfile::tempdir().unwrap();
        let result = check_for_update("", dir.path());
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_cache_content_update() {
        assert_eq!(
            parse_cache_content("update:2.0.0", "1.0.0"),
            Some("2.0.0".to_string())
        );
    }

    #[test]
    fn test_parse_cache_content_same_version() {
        assert_eq!(parse_cache_content("update:1.0.0", "1.0.0"), None);
    }

    #[test]
    fn test_parse_cache_content_current() {
        assert_eq!(parse_cache_content("current", "1.0.0"), None);
    }

    #[test]
    fn test_parse_cache_content_error() {
        assert_eq!(parse_cache_content("error", "1.0.0"), None);
    }
}
