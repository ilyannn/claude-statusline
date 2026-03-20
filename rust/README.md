# Claude Code Status Line — Rust

Rust rewrite of the Python status line script. Functionally equivalent, shares cache files, same environment variables.

## Quick Start

```bash
cargo build --release
# or: just build (from repo root)
```

Add to `~/.claude/settings.json`:

```json
{
  "statusLine": {
    "type": "command",
    "command": "CLAUDE_STATUSLINE_THEME=dark CLAUDE_STATUSLINE_SKIP_DIRTY=1 /path/to/rust/target/release/claude-statusline"
  }
}
```

Set `CLAUDE_STATUSLINE_THEME` to `dark` or `light` to skip the `defaults` subprocess (~7ms). Set `CLAUDE_STATUSLINE_SKIP_DIRTY` to skip `git status` and read `.git/HEAD` directly (~14ms). Both together bring latency to ~7ms.

## Benchmarks

3 rounds x 50 iterations, randomized order (`just bench` from repo root):

| Method | Avg |
|--------|-----|
| Rust (skip dirty + theme) | ~7ms |
| Rust (skip dirty) | ~14ms |
| Rust | ~26ms |
| Python 3.14 (venv) | ~64ms |
| Python 3.9 (system) | ~83ms |

## Architecture

```
src/
  main.rs      Entry point: stdin → run() → stdout
  lib.rs       Orchestrator: wires all modules together
  input.rs     JSON deserialization from stdin
  theme.rs     Dark/light mode detection + ANSI color palette
  git.rs       Branch name + dirty status (subprocess or .git/HEAD)
  cache.rs     XDG-compliant cache directory + file read/write
  usage.rs     OAuth token from Keychain + usage API via curl (forked)
  update.rs    npm version check (forked)
  format.rs    Output assembly: icons, colors, thresholds
tests/
  integration.rs   End-to-end binary tests via assert_cmd
```

## Tests

```bash
cargo test
# or: just test (from repo root, also runs Python tests)
```

96 unit tests + 11 integration tests covering all observable behavior from the Python test suite.

## Dependencies

Runtime (4 crates): `serde`, `serde_json`, `chrono`, `libc`. No async runtime, no network crates — HTTP calls use `curl` via subprocess in a forked child.
