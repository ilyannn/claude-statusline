pub mod cache;
pub mod format;
pub mod git;
pub mod input;
pub mod theme;
pub mod update;
pub mod usage;

use input::StatusInput;

/// Run the status line generation. Takes raw JSON string, returns formatted status line.
pub fn run(raw_input: &str) -> String {
    let data: StatusInput = match serde_json::from_str(raw_input) {
        Ok(d) => d,
        Err(_) => return "◐ --% ✦".to_string(),
    };

    let dark_mode = theme::detect_dark_mode();
    let colors = theme::get_colors(dark_mode);
    let cache_dir = cache::get_cache_dir();

    let (branch, dirty) = git::get_git_status(data.current_dir());
    let usage_data = usage::get_claude_usage(&cache_dir);
    let update_version = update::check_for_update(data.version(), &cache_dir);

    format::build_status_line(
        data.context_pct(),
        data.model_name(),
        colors,
        (branch.as_deref(), dirty),
        usage_data.as_ref(),
        update_version.as_deref(),
    )
}

/// Fork a background child process that runs `work` then exits.
/// Returns immediately in the parent. On fork failure, silently returns.
#[cfg(unix)]
pub fn fork_background<F: FnOnce()>(work: F) {
    unsafe {
        let pid = libc::fork();
        if pid == 0 {
            work();
            libc::_exit(0);
        }
        // Parent continues (pid > 0) or fork failed (pid == -1)
    }
}
