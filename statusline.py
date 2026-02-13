#!/usr/bin/env python3
"""Claude Code Status Line - displays context %, model, git branch, update status, usage limits."""

import json
import os
import subprocess
import sys
import time
import urllib.request
from pathlib import Path

# Cache settings
CACHE_FILE = Path("/tmp/claude-update-check")
CACHE_MAX_AGE = 3600  # 1 hour

USAGE_CACHE_FILE = Path("/tmp/claude-usage-cache")
USAGE_CACHE_MAX_AGE = 300  # 5 minutes


def detect_dark_mode() -> bool:
    """Detect if we should use dark mode colors. Returns True for dark, False for light."""
    # 1. Check explicit override
    theme = os.environ.get("CLAUDE_STATUSLINE_THEME", "").lower()
    if theme == "dark":
        return True
    if theme == "light":
        return False

    # 2. Check COLORFGBG (format: "fg;bg" - low bg number = dark)
    colorfgbg = os.environ.get("COLORFGBG", "")
    if colorfgbg:
        try:
            parts = colorfgbg.split(";")
            if len(parts) >= 2:
                bg = int(parts[-1])
                return bg < 8  # 0-7 are typically dark colors
        except ValueError:
            pass

    # 3. Check macOS system appearance
    try:
        result = subprocess.run(
            ["defaults", "read", "-g", "AppleInterfaceStyle"],
            capture_output=True,
            text=True,
            timeout=1,
        )
        return result.stdout.strip().lower() == "dark"
    except (subprocess.TimeoutExpired, FileNotFoundError, subprocess.SubprocessError):
        pass

    # Default to dark mode (most common for terminals)
    return True


def get_colors(dark_mode: bool) -> dict:
    """Get color codes based on theme."""
    reset = "\033[0m"
    bold = "\033[1m"
    dim = "\033[2m"

    if dark_mode:
        return {
            "reset": reset,
            "bold": bold,
            "dim": dim,
            "ctx_good": "\033[92m",  # bright green
            "ctx_warn": "\033[93m",  # bright yellow
            "ctx_crit": "\033[91m",  # bright red
            "model": "\033[96m",  # bright cyan
            "git": "\033[90m",  # gray
            "update": "\033[93m",  # bright yellow
            "usage_good": "\033[92m",  # bright green
            "usage_warn": "\033[93m",  # bright yellow
            "usage_crit": "\033[91m",  # bright red
        }
    else:
        return {
            "reset": reset,
            "bold": bold,
            "dim": dim,
            "ctx_good": "\033[32m",  # dark green
            "ctx_warn": "\033[33m",  # dark yellow/orange
            "ctx_crit": "\033[31m",  # dark red
            "model": "\033[34m",  # blue
            "git": "\033[90m",  # dark gray
            "update": "\033[33m",  # dark yellow
            "usage_good": "\033[32m",  # dark green
            "usage_warn": "\033[33m",  # dark yellow
            "usage_crit": "\033[31m",  # dark red
        }


def get_claude_oauth_token() -> str | None:
    """Get Claude Code OAuth token from macOS Keychain."""
    try:
        result = subprocess.run(
            [
                "security",
                "find-generic-password",
                "-s",
                "Claude Code-credentials",
                "-w",
            ],
            capture_output=True,
            text=True,
            timeout=2,
        )
        if result.returncode != 0:
            return None

        creds = json.loads(result.stdout.strip())
        return creds.get("claudeAiOauth", {}).get("accessToken")
    except (
        subprocess.TimeoutExpired,
        subprocess.SubprocessError,
        json.JSONDecodeError,
        KeyError,
        FileNotFoundError,
    ):
        pass
    return None


def get_claude_usage() -> dict | None:
    """Get Claude.ai usage stats (5h and weekly). Uses cache to avoid frequent API calls.

    Returns dict with 'five_hour' and 'seven_day' percentages, or None.
    """
    # Check cache
    if USAGE_CACHE_FILE.exists():
        cache_age = time.time() - USAGE_CACHE_FILE.stat().st_mtime
        if cache_age < USAGE_CACHE_MAX_AGE:
            try:
                return json.loads(USAGE_CACHE_FILE.read_text())
            except (json.JSONDecodeError, IOError):
                pass

    # Get OAuth token
    token = get_claude_oauth_token()
    if not token:
        return None

    # Fetch usage in background to not block status line
    try:
        pid = os.fork()
        if pid == 0:
            # Child process - fetch and cache
            try:
                # Use OAuth endpoint - no org ID needed
                req = urllib.request.Request(
                    "https://api.anthropic.com/api/oauth/usage",
                    headers={
                        "Authorization": f"Bearer {token}",
                        "Content-Type": "application/json",
                        "User-Agent": "claude-code/2.1.5",
                        "anthropic-beta": "oauth-2025-04-20",
                    },
                )
                with urllib.request.urlopen(req, timeout=10) as resp:
                    data = json.loads(resp.read().decode())
                    usage = {
                        "five_hour": data.get("five_hour", {}).get("utilization", 0),
                        "five_hour_resets": data.get("five_hour", {}).get(
                            "resets_at", ""
                        ),
                        "seven_day": data.get("seven_day", {}).get("utilization", 0),
                        "seven_day_resets": data.get("seven_day", {}).get(
                            "resets_at", ""
                        ),
                    }
                    USAGE_CACHE_FILE.write_text(json.dumps(usage))
            except Exception:
                pass
            os._exit(0)
    except (OSError, AttributeError):
        pass

    # Return cached value if exists
    if USAGE_CACHE_FILE.exists():
        try:
            return json.loads(USAGE_CACHE_FILE.read_text())
        except (json.JSONDecodeError, IOError):
            pass
    return None


def format_reset_time(iso_timestamp: str) -> str:
    """Format ISO timestamp to local time like '2am' or '3pm'."""
    if not iso_timestamp:
        return ""
    try:
        from datetime import datetime, timedelta

        # Parse ISO format and convert to local time
        dt = datetime.fromisoformat(iso_timestamp.replace("Z", "+00:00"))
        local_dt = dt.astimezone()
        # Round up to next hour if there are any minutes/seconds
        if local_dt.minute > 0 or local_dt.second > 0:
            local_dt = local_dt + timedelta(hours=1)
        hour = local_dt.hour
        if hour == 0:
            return "12am"
        elif hour < 12:
            return f"{hour}am"
        elif hour == 12:
            return "12pm"
        else:
            return f"{hour - 12}pm"
    except (ValueError, TypeError, AttributeError):
        return ""


def get_git_branch(directory: str) -> str | None:
    """Get current git branch if in a git repo."""
    if not directory or not Path(directory).is_dir():
        return None
    try:
        result = subprocess.run(
            ["git", "branch", "--show-current"],
            capture_output=True,
            text=True,
            timeout=1,
            cwd=directory,
        )
        if result.returncode == 0 and result.stdout.strip():
            return result.stdout.strip()
    except (subprocess.TimeoutExpired, FileNotFoundError, subprocess.SubprocessError):
        pass
    return None


def check_for_update(current_version: str) -> str | None:
    """Check if update is available. Uses cache to avoid frequent npm calls."""
    # Check cache age
    if CACHE_FILE.exists():
        cache_age = time.time() - CACHE_FILE.stat().st_mtime
        if cache_age < CACHE_MAX_AGE:
            content = CACHE_FILE.read_text().strip()
            if content.startswith("update:"):
                cached_version = content[7:]
                if cached_version != current_version:
                    return cached_version
            return None

    # Cache expired or doesn't exist - check in background
    try:
        # Fork to background for the check
        pid = os.fork()
        if pid == 0:
            # Child process - do the check
            try:
                result = subprocess.run(
                    ["npm", "view", "@anthropic-ai/claude-code", "version"],
                    capture_output=True,
                    text=True,
                    timeout=10,
                )
                latest = result.stdout.strip()
                if latest and latest != current_version:
                    CACHE_FILE.write_text(f"update:{latest}")
                else:
                    CACHE_FILE.write_text("current")
            except Exception:
                CACHE_FILE.write_text("error")
            os._exit(0)
    except (OSError, AttributeError):
        # os.fork not available (Windows) or failed - skip update check
        pass

    # Return cached value if exists
    if CACHE_FILE.exists():
        content = CACHE_FILE.read_text().strip()
        if content.startswith("update:"):
            cached_version = content[7:]
            if cached_version != current_version:
                return cached_version
    return None


def main():
    # Read JSON from stdin
    try:
        data = json.load(sys.stdin)
    except json.JSONDecodeError:
        print("◐ --% ✦")
        return

    # Debug: dump full JSON if CLAUDE_STATUSLINE_DEBUG is set
    debug_file = os.environ.get("CLAUDE_STATUSLINE_DEBUG")
    if debug_file:
        Path(debug_file).write_text(json.dumps(data, indent=2))

    # Extract values (handle None values)
    model_data = data.get("model") or {}
    model = model_data.get("display_name", "?") if isinstance(model_data, dict) else "?"
    ctx_data = data.get("context_window") or {}
    context_pct = int(
        ctx_data.get("used_percentage", 0) if isinstance(ctx_data, dict) else 0
    )
    workspace_data = data.get("workspace") or {}
    current_dir = (
        workspace_data.get("current_dir", "")
        if isinstance(workspace_data, dict)
        else ""
    )
    version = data.get("version", "") or ""

    # Detect theme and get colors
    dark_mode = detect_dark_mode()
    c = get_colors(dark_mode)

    # Context color based on percentage
    if context_pct < 50:
        ctx_color = c["ctx_good"]
    elif context_pct < 75:
        ctx_color = c["ctx_warn"]
    else:
        ctx_color = c["ctx_crit"]

    # Build status parts
    parts = []

    # Context percentage
    parts.append(f"{ctx_color}◐ {context_pct}%{c['reset']}")

    # Model
    parts.append(f"{c['model']}✦ {model}{c['reset']}")

    # Git branch
    branch = get_git_branch(current_dir)
    if branch:
        parts.append(f"{c['git']}⎇ {branch}{c['reset']}")

    # Claude.ai usage (5h limit)
    usage = get_claude_usage()
    if usage:
        five_h = int(usage.get("five_hour", 0))
        five_h_reset = format_reset_time(usage.get("five_hour_resets", ""))

        # Color based on usage level
        def usage_color(pct: int) -> str:
            if pct < 50:
                return c["usage_good"]
            elif pct < 80:
                return c["usage_warn"]
            return c["usage_crit"]

        five_h_str = f"⏱ {five_h}%"
        if five_h_reset:
            five_h_str += f"→{five_h_reset}"
        parts.append(f"{usage_color(five_h)}{five_h_str}{c['reset']}")

    # Update indicator
    if version:
        latest = check_for_update(version)
        if latest:
            parts.append(f"{c['update']}↑{latest}{c['reset']}")

    print(" ".join(parts))


if __name__ == "__main__":
    main()
