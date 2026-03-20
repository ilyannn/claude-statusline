# Claude Code Status Line — Rust

Rust rewrite of the Python status line script. Functionally equivalent, shares cache files, same environment variables. See the [main README](../README.md) for installation, benchmarks, and usage.

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

105 unit tests + 11 integration tests covering all observable behavior from the Python test suite.

## Dependencies

Runtime (4 crates): `serde`, `serde_json`, `chrono`, `libc`. No async runtime, no network crates — HTTP calls use `curl` via subprocess in a forked child.
