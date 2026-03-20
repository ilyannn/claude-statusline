use std::path::{Path, PathBuf};
use std::time::SystemTime;
use std::{env, fs};

/// Get cache directory: $XDG_CACHE_HOME > ~/.cache > ~/Library/Caches > /tmp.
pub fn get_cache_dir() -> PathBuf {
    let home = dirs_home();
    let candidates: Vec<Option<PathBuf>> = vec![
        env::var("XDG_CACHE_HOME").ok().map(PathBuf::from),
        home.as_ref().map(|h| h.join(".cache")),
        home.as_ref().map(|h| h.join("Library").join("Caches")),
        Some(PathBuf::from("/tmp")),
    ];

    for candidate in candidates.into_iter().flatten() {
        let cache_dir = candidate.join("claude-statusline");
        if fs::create_dir_all(&cache_dir).is_ok() {
            return cache_dir;
        }
    }
    PathBuf::from("/tmp")
}

fn dirs_home() -> Option<PathBuf> {
    env::var("HOME").ok().map(PathBuf::from)
}

/// Read cache file if it exists and is younger than max_age_secs.
pub fn read_cache(path: &Path, max_age_secs: u64) -> Option<String> {
    let metadata = fs::metadata(path).ok()?;
    let modified = metadata.modified().ok()?;
    let age = SystemTime::now().duration_since(modified).ok()?;
    if age.as_secs() >= max_age_secs {
        return None;
    }
    fs::read_to_string(path).ok()
}

/// Write content to cache file, ignoring errors.
pub fn write_cache(path: &Path, content: &str) {
    let _ = fs::write(path, content);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_cache_dir_xdg() {
        let dir = tempfile::tempdir().unwrap();
        env::set_var("XDG_CACHE_HOME", dir.path());
        let result = get_cache_dir();
        assert_eq!(result, dir.path().join("claude-statusline"));
        assert!(result.exists());
        env::remove_var("XDG_CACHE_HOME");
    }

    #[test]
    fn test_read_cache_fresh() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test-cache");
        fs::write(&path, "hello").unwrap();
        assert_eq!(read_cache(&path, 60), Some("hello".to_string()));
    }

    #[test]
    fn test_read_cache_missing() {
        let path = Path::new("/nonexistent/cache/file");
        assert_eq!(read_cache(path, 60), None);
    }

    #[test]
    fn test_write_and_read_cache() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test-cache");
        write_cache(&path, "test data");
        assert_eq!(read_cache(&path, 60), Some("test data".to_string()));
    }
}
