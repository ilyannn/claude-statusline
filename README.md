# Claude Code Status Line

Custom status line script for Claude Code displaying context usage, model, git branch, Claude.ai usage limits, and update availability.

## Output Format

```
◐ 42% ✦ Opus ⎇ main ⏱ 40%→2am ↑1.0.24
```

- `◐ 42%` - Context window usage (color-coded: green <50%, yellow 50-74%, red ≥75%)
- `✦ Opus` - Current model
- `⎇ main` - Git branch (only shown in git repos)
- `⏱ 40%→2am` - Claude.ai 5-hour usage + reset time (color-coded: green <50%, yellow 50-79%, red ≥80%)
- `↑1.0.24` - Update available (only shown when newer version exists)

## Installation

Add to `~/.claude/settings.json`:

```json
{
  "statusLine": {
    "type": "command",
    "command": "uv run ~/Code/claude-statusline/statusline.py"
  }
}
```

## Claude.ai Usage Tracking

The script reads Claude Code's OAuth credentials from macOS Keychain to fetch usage stats from the Anthropic API. Requires being logged into Claude Code with your claude.ai account.

## Theme Detection

Colors adapt to light/dark mode via:

1. **Explicit override**: `CLAUDE_STATUSLINE_THEME=light` or `dark`
2. **COLORFGBG**: Terminal-set environment variable
3. **macOS appearance**: System dark mode detection
4. **Default**: Dark mode

## Development

```bash
just test          # Run 105 tests
just test-cov      # Tests with coverage (80%)
just smoke         # Quick visual test
just smoke-colors  # Show all 3 color states
just smoke-light   # Test light theme
just smoke-dark    # Test dark theme
just smoke-usage   # Test with mock usage data
just clear-cache   # Reset caches
just cache-status  # Check cache age
```

## Caching & Performance

| Check | Cache Duration | Location |
|-------|---------------|----------|
| Claude.ai usage | 5 minutes | `/tmp/claude-usage-cache` |
| Update check | 1 hour | `/tmp/claude-update-check` |

All API calls run in background via `fork()` - they never block the status line.

**Python startup overhead:** ~30-50ms per invocation. A Rust rewrite would reduce this to ~1-5ms.
