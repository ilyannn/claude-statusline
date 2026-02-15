# Claude Code Status Line

A Python script that generates a custom status line for Claude Code CLI.

## Architecture

- **statusline.py** - Main script, reads JSON from stdin, outputs ANSI-colored status line
- **test_statusline.py** - 105 tests covering observable behavior (80% coverage)
- **justfile** - Development commands

## Key Functions

- `detect_dark_mode()` - Detects light/dark theme via env vars or macOS defaults
- `get_colors(dark_mode)` - Returns ANSI color codes for current theme
- `get_git_status(directory)` - Gets current git branch and dirty status in a single call
- `get_claude_oauth_token()` - Reads OAuth token from macOS Keychain
- `get_claude_usage()` - Fetches 5-hour usage from Anthropic API (cached 5 min)
- `check_for_update(version)` - Checks npm for newer version (cached 1 hour)
- `format_reset_time(iso_timestamp)` - Formats reset time as "2am", "3pm", etc.
- `main()` - Parses stdin JSON, assembles and prints status line

## Input Format (stdin JSON)

```json
{
  "model": {"display_name": "Opus"},
  "context_window": {"used_percentage": 42.5},
  "workspace": {"current_dir": "/path/to/project"},
  "version": "2.1.29"
}
```

## Output Format

```
◑ 42% ✦ Opus ⎇ main ⏱ 40%→2am ↑1.0.24
```

Order: context → model → git branch → usage → update

## Color Thresholds

| Metric | Green | Yellow | Red |
|--------|-------|--------|-----|
| Context | <50% | 50-74% | ≥75% |
| Usage | <50% | 50-79% | ≥80% |

## Testing

Run `just test` - tests document all observable behavior for potential Rust rewrite.

## API Endpoints

- Usage: `https://api.anthropic.com/api/oauth/usage` (Bearer token auth)
- Update: `npm view @anthropic-ai/claude-code version`
