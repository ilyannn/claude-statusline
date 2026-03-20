# Claude Code Status Line

A status line for Claude Code CLI, implemented in both Python and Rust.

## Architecture

- **statusline.py** - Python implementation, reads JSON from stdin, outputs ANSI-colored status line
- **test_statusline.py** - 117 Python tests covering observable behavior
- **rust/** - Rust rewrite, functionally equivalent, shares cache files
- **justfile** - Development commands (`just check` runs everything)

## Key Commands

- `just check` - lint + format-check + toml-check + test (Python 3.14 + 3.9 + Rust)
- `just bench` - 3 rounds × 50 iterations, randomized order
- `just format` - ruff + cargo fmt

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

## Environment Variables

| Variable | Description |
|----------|-------------|
| `CLAUDE_STATUSLINE_THEME` | Force `light` or `dark` theme |
| `CLAUDE_STATUSLINE_SKIP_DIRTY` | Read `.git/HEAD` directly, skip dirty check (Rust only) |
| `CLAUDE_STATUSLINE_DEBUG` | Path to dump input JSON |

## Benchmarks

| Method | Avg |
|--------|-----|
| Rust + skip dirty | ~15ms |
| Rust | ~28ms |
| venv Python 3.14 | ~67ms |
| System Python 3.9 | ~85ms |

## API Endpoints

- Usage: `https://api.anthropic.com/api/oauth/usage` (Bearer token auth)
- Update: `npm view @anthropic-ai/claude-code version`
