# Statusline development commands

# Default recipe to display help
_default:
    @just --list --unsorted

# ---- Quality ----------------------------------------------------------------

# Run all checks (lint + format-check + toml-check + test)
check: lint format-check toml-check test

# Lint Python code with ruff
lint:
    uvx ruff check .

# Format Python code with ruff
format:
    uvx ruff format .
    uvx ruff check --select I --fix .

# Check if code is formatted correctly (fails if not)
format-check:
    uvx ruff format --check .

# Check TOML formatting with taplo
toml-check:
    taplo check
    taplo format --check

# Format TOML files with taplo
toml-format:
    taplo format

# ---- Testing ----------------------------------------------------------------

# Run all tests (including Python 3.9 compatibility)
test:
    uv run --extra dev pytest test_statusline.py -v
    uv run --python 3.9 --extra dev pytest test_statusline.py -v

# Run tests with coverage
test-cov:
    uv run --extra dev pytest test_statusline.py -v --cov=statusline --cov-report=term-missing

# ---- Smoke Tests ------------------------------------------------------------

# Run a quick smoke test with sample data
smoke:
    @echo '{"model":{"display_name":"Opus"},"context_window":{"used_percentage":42},"workspace":{"current_dir":"'$(pwd)'"},"version":"1.0.23"}' | uv run ./statusline.py

# Test all context percentage colors
smoke-colors:
    @echo "Low (green):"
    @echo '{"model":{"display_name":"Opus"},"context_window":{"used_percentage":30},"workspace":{"current_dir":"/tmp"},"version":"1.0.23"}' | uv run ./statusline.py
    @echo ""
    @echo "Medium (yellow):"
    @echo '{"model":{"display_name":"Opus"},"context_window":{"used_percentage":60},"workspace":{"current_dir":"/tmp"},"version":"1.0.23"}' | uv run ./statusline.py
    @echo ""
    @echo "High (red):"
    @echo '{"model":{"display_name":"Opus"},"context_window":{"used_percentage":85},"workspace":{"current_dir":"/tmp"},"version":"1.0.23"}' | uv run ./statusline.py

# Test light mode colors
smoke-light:
    @export CLAUDE_STATUSLINE_THEME=light && echo '{"model":{"display_name":"Opus"},"context_window":{"used_percentage":42},"workspace":{"current_dir":"/tmp"},"version":"1.0.23"}' | uv run ./statusline.py

# Test dark mode colors
smoke-dark:
    @export CLAUDE_STATUSLINE_THEME=dark && echo '{"model":{"display_name":"Opus"},"context_window":{"used_percentage":42},"workspace":{"current_dir":"/tmp"},"version":"1.0.23"}' | uv run ./statusline.py

# Test with mock usage data
smoke-usage:
    #!/usr/bin/env bash
    d=$(uv run ./statusline.py --print-cache-dir)
    echo '{"five_hour": 25, "seven_day": 60}' > "$d/usage-cache"
    echo '{"model":{"display_name":"Opus"},"context_window":{"used_percentage":42},"workspace":{"current_dir":"/tmp"},"version":"1.0.23"}' | uv run ./statusline.py
    rm "$d/usage-cache"

# ---- Benchmarks -------------------------------------------------------------

# Benchmark: system python vs new python vs uv run
bench:
    #!/usr/bin/env bash
    input='{"model":{"display_name":"Opus"},"context_window":{"used_percentage":42},"workspace":{"current_dir":"/tmp"},"version":"1.0.23"}'
    n=50
    echo "Running $n iterations each..."
    echo ""

    # System Python
    echo "System Python ($(python3 --version 2>&1)):"
    start=$(python3 -c 'import time; print(time.time())')
    for i in $(seq 1 $n); do echo "$input" | python3 ./statusline.py > /dev/null 2>&1; done
    end=$(python3 -c 'import time; print(time.time())')
    avg=$(python3 -c "print(f'{($end - $start) / $n * 1000:.0f}ms')")
    echo "  avg: $avg"
    echo ""

    # New Python (via venv)
    uv venv --python 3.14 .bench-venv -q
    echo "New Python ($(.bench-venv/bin/python --version 2>&1)):"
    start=$(python3 -c 'import time; print(time.time())')
    for i in $(seq 1 $n); do echo "$input" | .bench-venv/bin/python ./statusline.py > /dev/null 2>&1; done
    end=$(python3 -c 'import time; print(time.time())')
    avg=$(python3 -c "print(f'{($end - $start) / $n * 1000:.0f}ms')")
    echo "  avg: $avg"
    echo ""

    # uv run with script
    echo "uv run ./statusline.py:"
    start=$(python3 -c 'import time; print(time.time())')
    for i in $(seq 1 $n); do echo "$input" | uv run ./statusline.py > /dev/null 2>&1; done
    end=$(python3 -c 'import time; print(time.time())')
    avg=$(python3 -c "print(f'{($end - $start) / $n * 1000:.0f}ms')")
    echo "  avg: $avg"

    rm -rf .bench-venv

# ---- Cache Management -------------------------------------------------------

# Clear update cache
cache-clear:
    #!/usr/bin/env bash
    d=$(uv run ./statusline.py --print-cache-dir)
    rm -f "$d/update-check" "$d/usage-cache"
    echo "Cache cleared"

# Show current cache status
cache-status:
    #!/usr/bin/env bash
    d=$(uv run ./statusline.py --print-cache-dir)
    if [ -f "$d/update-check" ]; then
        echo "Update cache:"
        cat "$d/update-check"
        echo ""
        echo "Age: $(( ($(date +%s) - $(stat -f %m "$d/update-check")) / 60 )) minutes"
    else
        echo "No update cache file"
    fi
    echo ""
    if [ -f "$d/usage-cache" ]; then
        echo "Usage cache:"
        cat "$d/usage-cache"
        echo ""
        echo "Age: $(( ($(date +%s) - $(stat -f %m "$d/usage-cache")) )) seconds"
    else
        echo "No usage cache file"
    fi
