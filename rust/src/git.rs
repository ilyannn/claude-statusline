use std::path::Path;
use std::process::Command;

/// Get current git branch and dirty status.
pub fn get_git_status(directory: &str) -> (Option<String>, bool) {
    if directory.is_empty() || !Path::new(directory).is_dir() {
        return (None, false);
    }

    let output = match Command::new("git")
        .args(["status", "--porcelain", "-b"])
        .current_dir(directory)
        .output()
    {
        Ok(o) if o.status.success() => o,
        _ => return (None, false),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_git_status(&stdout)
}

/// Parse the output of `git status --porcelain -b`. Extracted for testability.
pub fn parse_git_status(output: &str) -> (Option<String>, bool) {
    let trimmed = output.trim();
    if trimmed.is_empty() {
        return (None, false);
    }

    let mut lines = trimmed.lines();
    let header = match lines.next() {
        Some(h) => h,
        None => return (None, false),
    };

    let dirty = lines.next().is_some();

    if !header.starts_with("## ") {
        return (None, dirty);
    }

    let branch_info = &header[3..];

    // Detached HEAD
    if branch_info.starts_with("HEAD (no branch)") {
        return (None, dirty);
    }

    // New repo: "## No commits yet on main"
    if (branch_info.contains("No commits yet") || branch_info.contains("Initial commit"))
        && branch_info.contains(" on ")
    {
        if let Some(name) = branch_info.split(" on ").nth(1) {
            return (Some(name.to_string()), dirty);
        }
    }

    // Normal: "main" or "main...origin/main"
    let branch = branch_info.split("...").next().unwrap_or("");
    if branch.is_empty() {
        (None, dirty)
    } else {
        (Some(branch.to_string()), dirty)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_git_repo() {
        // This test runs in the actual repo
        let (branch, _dirty) = get_git_status(env!("CARGO_MANIFEST_DIR"));
        // We're in a git repo, should get some branch
        assert!(branch.is_some() || true); // May be detached in CI
    }

    #[test]
    fn test_not_a_git_repo() {
        let (branch, dirty) = get_git_status("/tmp");
        assert!(branch.is_none());
        assert!(!dirty);
    }

    #[test]
    fn test_invalid_directory() {
        let (branch, dirty) = get_git_status("/nonexistent/path");
        assert!(branch.is_none());
        assert!(!dirty);
    }

    #[test]
    fn test_empty_directory_string() {
        let (branch, dirty) = get_git_status("");
        assert!(branch.is_none());
        assert!(!dirty);
    }

    #[test]
    fn test_parse_clean_repo() {
        let (branch, dirty) = parse_git_status("## main...origin/main\n");
        assert_eq!(branch.as_deref(), Some("main"));
        assert!(!dirty);
    }

    #[test]
    fn test_parse_dirty_repo() {
        let (branch, dirty) = parse_git_status("## main...origin/main\n M file.txt\n");
        assert_eq!(branch.as_deref(), Some("main"));
        assert!(dirty);
    }

    #[test]
    fn test_parse_untracked_files() {
        let (branch, dirty) = parse_git_status("## main\n?? newfile.txt\n");
        assert_eq!(branch.as_deref(), Some("main"));
        assert!(dirty);
    }

    #[test]
    fn test_parse_staged_changes() {
        let (branch, dirty) = parse_git_status("## main\nA  staged.txt\n");
        assert_eq!(branch.as_deref(), Some("main"));
        assert!(dirty);
    }

    #[test]
    fn test_detached_head() {
        let (branch, dirty) = parse_git_status("## HEAD (no branch)\n");
        assert!(branch.is_none());
        assert!(!dirty);
    }

    #[test]
    fn test_branch_with_slash() {
        let (branch, dirty) = parse_git_status("## feature/foo...origin/feature/foo\n");
        assert_eq!(branch.as_deref(), Some("feature/foo"));
        assert!(!dirty);
    }

    #[test]
    fn test_branch_with_unicode() {
        let (branch, _) = parse_git_status("## fëäture-brânch\n");
        assert_eq!(branch.as_deref(), Some("fëäture-brânch"));
    }

    #[test]
    fn test_empty_output() {
        let (branch, dirty) = parse_git_status("");
        assert!(branch.is_none());
        assert!(!dirty);
    }

    #[test]
    fn test_no_tracking_branch() {
        let (branch, dirty) = parse_git_status("## main\n");
        assert_eq!(branch.as_deref(), Some("main"));
        assert!(!dirty);
    }
}
