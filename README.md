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

## Requirements

**Runtime:**
- macOS (Keychain access for OAuth, `defaults` for theme detection)
- Python 3.11+ (standard library only, no pip install needed)
- npm - for update version check (`npm view @anthropic-ai/claude-code version`)
- git - for branch detection

**Development:**
- [uv](https://docs.astral.sh/uv/) - to run tests
- [just](https://github.com/casey/just) - command runner
- [taplo](https://taplo.tamasfe.dev/) - TOML formatting
- ruff runs via `uvx`, no separate install needed

## Installation

Add to `~/.claude/settings.json`:

```json
{
  "statusLine": {
    "type": "command",
    "command": "/path/to/statusline.py"
  }
}
```

## Claude.ai Usage Tracking

The script reads Claude Code's OAuth credentials from macOS Keychain (`security find-generic-password -s "Claude Code-credentials"`) to fetch usage stats from the Anthropic API. Requires being logged into Claude Code with your claude.ai account.

Inspired by [this Reddit post](https://old.reddit.com/r/ClaudeCode/comments/1qgzvth/macos_app_for_claude_sessionkeyfree_tracking_v223/).

## Environment Variables

| Variable | Description |
|----------|-------------|
| `CLAUDE_STATUSLINE_THEME` | Force `light` or `dark` theme |
| `CLAUDE_STATUSLINE_DEBUG` | Path to dump input JSON (e.g. `/tmp/debug.json`) |

## Theme Detection

Colors adapt to light/dark mode via:

1. **Explicit override**: `CLAUDE_STATUSLINE_THEME=light` or `dark`
2. **COLORFGBG**: Terminal-set environment variable
3. **macOS appearance**: System dark mode detection
4. **Default**: Dark mode

## Development

```bash
just check         # Run all checks (lint + format + toml + test)
just lint          # Lint with ruff
just format        # Format with ruff
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

| Check | Cache Duration | File |
|-------|---------------|------|
| Claude.ai usage | 5 minutes | `usage-cache` |
| Update check | 1 hour | `update-check` |

Cache directory resolution: `$XDG_CACHE_HOME/claude-statusline` > `~/.cache/claude-statusline` > `~/Library/Caches/claude-statusline` > `/tmp`.

All API calls run in background via `fork()` - they never block the status line.

**Python startup overhead:** ~30-50ms per invocation. A Rust rewrite would reduce this to ~1-5ms.
