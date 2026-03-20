# Claude Code Status Line

Custom status line for Claude Code displaying context usage, model, git branch, Claude.ai usage limits, and update availability. Available in Python and [Rust](#rust-rewrite) (recommended, ~7ms with env var tuning).

## Output Format

```
◑ 42% ✦ Opus ⎇ main ⏱ 40%→2am ↑1.0.24
```

- `◔◑◕●` - Context window usage with fill-level icon (color-coded: green <50%, yellow 50-74%, red ≥75%)
- `✦ Opus` - Current model
- `⎇ main` - Git branch (only shown in git repos; dirty status marked with a `*`)
- `⏱ 40%→2am` - Claude.ai 5-hour usage + reset time (color-coded: green <50%, yellow 50-79%, red ≥80%)
- `↑1.0.24` - Update available (only shown when newer version exists)

## Requirements

**Runtime (Rust):**
- macOS (Keychain access for OAuth, `defaults` for theme detection)
- Rust toolchain (to build) or pre-built binary
- npm - for update version check
- git - for branch and dirty status detection
- curl - for API calls

**Runtime (Python):**
- macOS (Keychain access for OAuth, `defaults` for theme detection)
- Python 3.9+ (standard library only, no pip install needed)
- npm - for update version check
- git - for branch and dirty status detection

**Development:**
- [uv](https://docs.astral.sh/uv/) - to run Python tests
- [just](https://github.com/casey/just) - command runner
- [taplo](https://taplo.tamasfe.dev/) - TOML formatting
- ruff runs via `uvx`, no separate install needed

## Installation

### Rust (recommended)

Build the Rust binary for best performance:

```bash
just build
```

Add to `~/.claude/settings.json`:

```json
{
  "statusLine": {
    "type": "command",
    "command": "CLAUDE_STATUSLINE_THEME=dark CLAUDE_STATUSLINE_SKIP_DIRTY=1 /path/to/claude-statusline/rust/target/release/claude-statusline"
  }
}
```

### Python

Create a venv with a recent Python, then add to `~/.claude/settings.json`:

```bash
uv venv --python 3.14 /path/to/claude-statusline/.venv
```

```json
{
  "statusLine": {
    "type": "command",
    "command": "/path/to/claude-statusline/.venv/bin/python /path/to/claude-statusline/statusline.py"
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
| `CLAUDE_STATUSLINE_SKIP_DIRTY` | Skip `git status` dirty check; read `.git/HEAD` directly (Rust only) |
| `CLAUDE_STATUSLINE_DEBUG` | Path to dump input JSON (e.g. `/tmp/debug.json`) |

## Theme Detection

Colors adapt to light/dark mode via:

1. **Explicit override**: `CLAUDE_STATUSLINE_THEME=light` or `dark`
2. **COLORFGBG**: Terminal-set environment variable
3. **macOS appearance**: System dark mode detection
4. **Default**: Dark mode

## Development

```bash
just build         # Build Rust release binary
just check         # Run all checks (lint + format + toml + test)
just lint          # Lint with ruff + clippy
just format        # Format with ruff + cargo fmt
just test          # Run all tests (Python 3.14 + 3.9 + Rust)
just test-cov      # Python tests with coverage (80%)
just smoke         # Quick visual test
just smoke-colors  # Show all 3 color states
just smoke-light   # Test light theme
just smoke-dark    # Test dark theme
just smoke-usage   # Test with mock usage data
just bench         # Benchmark Python versions
just cache-clear   # Reset caches
just cache-status  # Check cache age
```

## Caching & Performance

| Check | Cache Duration | File |
|-------|---------------|------|
| Claude.ai usage | 5 minutes | `usage-cache` |
| Update check | 1 hour | `update-check` |

Cache directory resolution: `$XDG_CACHE_HOME/claude-statusline` > `~/.cache/claude-statusline` > `~/Library/Caches/claude-statusline` > `/tmp`.

All API calls run in background via `fork()` - they never block the status line.

**Startup benchmarks** (3 rounds × 50 iterations, randomized order, `just bench`):

| Method | Avg |
|--------|-----|
| Rust (skip dirty + theme) | ~7ms |
| Rust (skip dirty) | ~14ms |
| Rust | ~26ms |
| venv Python 3.14 | ~64ms |
| System Python 3.9 | ~83ms |
| `uv run` | ~89ms |

## Rust Rewrite

A full Rust rewrite lives in [`rust/`](rust/). It is functionally equivalent to the Python version with identical output, shared cache files, and the same environment variables. See [`rust/README.md`](rust/README.md) for architecture and details.

Two env vars eliminate subprocess overhead:
- `CLAUDE_STATUSLINE_THEME=dark` — skips `defaults` call (~7ms saved)
- `CLAUDE_STATUSLINE_SKIP_DIRTY=1` — reads `.git/HEAD` instead of `git status` (~12ms saved, loses dirty `*` indicator)

```bash
just build
# Binary at rust/target/release/claude-statusline
```
