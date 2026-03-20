use std::io::Read;
use std::path::Path;

const HELP_TEXT: &str = "\
Claude Code Status Line — displays context %, model, git branch, usage, and update info.

Usage: claude-statusline [OPTIONS]
  Reads Claude Code status JSON from stdin and prints a formatted status line.

Options:
  --help             Show this help message
  --print-cache-dir  Print the cache directory path and exit

Environment variables:
  CLAUDE_STATUSLINE_THEME       Force color theme: \"dark\" or \"light\"
  CLAUDE_STATUSLINE_SKIP_DIRTY  Skip git dirty check (faster, reads .git/HEAD directly)
  CLAUDE_STATUSLINE_DEBUG       Write raw stdin JSON to this file path for debugging
  CLAUDE_CONFIG_DIR             Override Claude config directory (default: ~/.claude)";

fn main() {
    let arg = std::env::args().nth(1);

    // Handle --help flag
    if arg.as_deref() == Some("--help") {
        println!("{HELP_TEXT}");
        return;
    }

    // Handle --print-cache-dir flag
    if arg.as_deref() == Some("--print-cache-dir") {
        println!("{}", claude_statusline::cache::get_cache_dir().display());
        return;
    }

    // Read JSON from stdin
    let mut input = String::new();
    if std::io::stdin().read_to_string(&mut input).is_err() {
        println!("◐ --% ✦");
        return;
    }

    // Debug: dump full JSON if CLAUDE_STATUSLINE_DEBUG is set
    if let Ok(debug_file) = std::env::var("CLAUDE_STATUSLINE_DEBUG") {
        let _ = std::fs::write(Path::new(&debug_file), &input);
    }

    let output = claude_statusline::run(&input);
    println!("{}", output);
}
