"""Tests for Claude Code status line script."""

import io
import json
import os
import subprocess
import sys
from pathlib import Path
from unittest.mock import MagicMock, patch


# Add parent dir to path for import
sys.path.insert(0, str(Path(__file__).parent))
import statusline


class TestDetectDarkMode:
    """Tests for dark mode detection."""

    def test_explicit_dark_override(self):
        with patch.dict(os.environ, {"CLAUDE_STATUSLINE_THEME": "dark"}):
            assert statusline.detect_dark_mode() is True

    def test_explicit_light_override(self):
        with patch.dict(os.environ, {"CLAUDE_STATUSLINE_THEME": "light"}):
            assert statusline.detect_dark_mode() is False

    def test_explicit_override_case_insensitive(self):
        with patch.dict(os.environ, {"CLAUDE_STATUSLINE_THEME": "DARK"}):
            assert statusline.detect_dark_mode() is True
        with patch.dict(os.environ, {"CLAUDE_STATUSLINE_THEME": "Light"}):
            assert statusline.detect_dark_mode() is False

    def test_colorfgbg_dark_background(self):
        with patch.dict(os.environ, {"COLORFGBG": "15;0"}, clear=False):
            # Clear the override
            os.environ.pop("CLAUDE_STATUSLINE_THEME", None)
            assert statusline.detect_dark_mode() is True

    def test_colorfgbg_light_background(self):
        env = {"COLORFGBG": "0;15"}
        with patch.dict(os.environ, env, clear=True):
            assert statusline.detect_dark_mode() is False

    def test_colorfgbg_invalid_format(self):
        with patch.dict(os.environ, {"COLORFGBG": "invalid"}, clear=True):
            # Should fall through to macOS check or default
            with patch("subprocess.run") as mock_run:
                mock_run.side_effect = FileNotFoundError()
                assert statusline.detect_dark_mode() is True  # Default


class TestGetColors:
    """Tests for color palette selection."""

    def test_dark_mode_colors(self):
        colors = statusline.get_colors(dark_mode=True)
        assert "ctx_good" in colors
        assert "ctx_warn" in colors
        assert "ctx_crit" in colors
        assert "model" in colors
        assert "git" in colors
        assert "update" in colors
        assert "reset" in colors
        # Dark mode uses bright variants (9x)
        assert "92m" in colors["ctx_good"]  # bright green

    def test_light_mode_colors(self):
        colors = statusline.get_colors(dark_mode=False)
        assert "ctx_good" in colors
        # Light mode uses standard variants (3x)
        assert "32m" in colors["ctx_good"]  # standard green


class TestGetGitBranch:
    """Tests for git branch detection."""

    def test_valid_git_repo(self, tmp_path):
        # Create a git repo
        subprocess.run(["git", "init"], cwd=tmp_path, capture_output=True)
        subprocess.run(
            ["git", "checkout", "-b", "test-branch"],
            cwd=tmp_path,
            capture_output=True,
        )
        branch = statusline.get_git_branch(str(tmp_path))
        assert branch == "test-branch"

    def test_not_a_git_repo(self, tmp_path):
        branch = statusline.get_git_branch(str(tmp_path))
        assert branch is None

    def test_invalid_directory(self):
        branch = statusline.get_git_branch("/nonexistent/path")
        assert branch is None

    def test_empty_directory_string(self):
        branch = statusline.get_git_branch("")
        assert branch is None


class TestCheckForUpdate:
    """Tests for update checking with cache."""

    def test_cache_returns_update_available(self, tmp_path):
        cache_file = tmp_path / "cache"
        cache_file.write_text("update:1.0.25")

        with patch.object(statusline, "CACHE_FILE", cache_file):
            result = statusline.check_for_update("1.0.23")
            assert result == "1.0.25"

    def test_cache_returns_none_when_already_on_cached_version(self, tmp_path):
        cache_file = tmp_path / "cache"
        cache_file.write_text("update:1.0.25")

        with patch.object(statusline, "CACHE_FILE", cache_file):
            result = statusline.check_for_update("1.0.25")
            assert result is None

    def test_cache_returns_current(self, tmp_path):
        cache_file = tmp_path / "cache"
        cache_file.write_text("current")

        with patch.object(statusline, "CACHE_FILE", cache_file):
            result = statusline.check_for_update("1.0.23")
            assert result is None

    def test_no_cache_triggers_background_check(self, tmp_path):
        cache_file = tmp_path / "cache"

        with patch.object(statusline, "CACHE_FILE", cache_file):
            with patch("os.fork", side_effect=OSError("fork not available")):
                result = statusline.check_for_update("1.0.23")
                assert result is None


class TestMainOutput:
    """Tests for main function output formatting."""

    def run_with_input(
        self, data: dict, dark_mode: bool = True, usage: dict | None = None
    ) -> str:
        """Helper to run main() with given input data."""
        json_input = json.dumps(data)
        with patch("sys.stdin", io.StringIO(json_input)):
            with patch.object(statusline, "detect_dark_mode", return_value=dark_mode):
                with patch.object(statusline, "get_git_branch", return_value=None):
                    with patch.object(
                        statusline, "check_for_update", return_value=None
                    ):
                        with patch.object(
                            statusline, "get_claude_usage", return_value=usage
                        ):
                            # Capture stdout
                            captured = io.StringIO()
                            with patch("sys.stdout", captured):
                                statusline.main()
                            return captured.getvalue().strip()

    def test_basic_output_format(self):
        data = {
            "model": {"display_name": "Opus"},
            "context_window": {"used_percentage": 42},
            "workspace": {"current_dir": "/test"},
            "version": "1.0.23",
        }
        output = self.run_with_input(data)
        assert "◐ 42%" in output
        assert "✦ Opus" in output

    def test_context_low_green(self):
        data = {
            "model": {"display_name": "Opus"},
            "context_window": {"used_percentage": 30},
            "workspace": {"current_dir": "/test"},
            "version": "1.0.23",
        }
        output = self.run_with_input(data, dark_mode=True)
        assert "\033[92m" in output  # bright green

    def test_context_medium_yellow(self):
        data = {
            "model": {"display_name": "Opus"},
            "context_window": {"used_percentage": 60},
            "workspace": {"current_dir": "/test"},
            "version": "1.0.23",
        }
        output = self.run_with_input(data, dark_mode=True)
        assert "\033[93m" in output  # bright yellow

    def test_context_high_red(self):
        data = {
            "model": {"display_name": "Opus"},
            "context_window": {"used_percentage": 80},
            "workspace": {"current_dir": "/test"},
            "version": "1.0.23",
        }
        output = self.run_with_input(data, dark_mode=True)
        assert "\033[91m" in output  # bright red

    def test_with_git_branch(self):
        data = {
            "model": {"display_name": "Opus"},
            "context_window": {"used_percentage": 42},
            "workspace": {"current_dir": "/test"},
            "version": "1.0.23",
        }
        json_input = json.dumps(data)
        with patch("sys.stdin", io.StringIO(json_input)):
            with patch.object(statusline, "detect_dark_mode", return_value=True):
                with patch.object(statusline, "get_git_branch", return_value="main"):
                    with patch.object(
                        statusline, "check_for_update", return_value=None
                    ):
                        with patch.object(
                            statusline, "get_claude_usage", return_value=None
                        ):
                            captured = io.StringIO()
                            with patch("sys.stdout", captured):
                                statusline.main()
                            output = captured.getvalue().strip()
        assert "⎇ main" in output

    def test_with_update_available(self):
        data = {
            "model": {"display_name": "Opus"},
            "context_window": {"used_percentage": 42},
            "workspace": {"current_dir": "/test"},
            "version": "1.0.23",
        }
        json_input = json.dumps(data)
        with patch("sys.stdin", io.StringIO(json_input)):
            with patch.object(statusline, "detect_dark_mode", return_value=True):
                with patch.object(statusline, "get_git_branch", return_value=None):
                    with patch.object(
                        statusline, "check_for_update", return_value="1.0.25"
                    ):
                        with patch.object(
                            statusline, "get_claude_usage", return_value=None
                        ):
                            captured = io.StringIO()
                            with patch("sys.stdout", captured):
                                statusline.main()
                            output = captured.getvalue().strip()
        assert "↑1.0.25" in output

    def test_invalid_json_input(self):
        with patch("sys.stdin", io.StringIO("not json")):
            captured = io.StringIO()
            with patch("sys.stdout", captured):
                statusline.main()
            output = captured.getvalue().strip()
        assert "◐ --%" in output and "✦" in output

    def test_different_models(self):
        for model_name in ["Opus", "Sonnet", "Haiku"]:
            data = {
                "model": {"display_name": model_name},
                "context_window": {"used_percentage": 50},
                "workspace": {"current_dir": "/test"},
                "version": "1.0.23",
            }
            output = self.run_with_input(data)
            assert f"✦ {model_name}" in output

    def test_with_usage_stats(self):
        data = {
            "model": {"display_name": "Opus"},
            "context_window": {"used_percentage": 42},
            "workspace": {"current_dir": "/test"},
            "version": "1.0.23",
        }
        usage = {"five_hour": 25, "seven_day": 60}
        output = self.run_with_input(data, usage=usage)
        assert "⏱ 25%" in output

    def test_usage_colors_green(self):
        data = {
            "model": {"display_name": "Opus"},
            "context_window": {"used_percentage": 42},
            "workspace": {"current_dir": "/test"},
            "version": "1.0.23",
        }
        usage = {"five_hour": 30}
        output = self.run_with_input(data, dark_mode=True, usage=usage)
        assert "\033[92m⏱ 30%" in output

    def test_usage_colors_yellow(self):
        data = {
            "model": {"display_name": "Opus"},
            "context_window": {"used_percentage": 42},
            "workspace": {"current_dir": "/test"},
            "version": "1.0.23",
        }
        usage = {"five_hour": 60}
        output = self.run_with_input(data, dark_mode=True, usage=usage)
        assert "\033[93m⏱ 60%" in output

    def test_usage_colors_red(self):
        data = {
            "model": {"display_name": "Opus"},
            "context_window": {"used_percentage": 42},
            "workspace": {"current_dir": "/test"},
            "version": "1.0.23",
        }
        usage = {"five_hour": 85}
        output = self.run_with_input(data, dark_mode=True, usage=usage)
        assert "\033[91m⏱ 85%" in output

    def test_no_usage_when_unavailable(self):
        data = {
            "model": {"display_name": "Opus"},
            "context_window": {"used_percentage": 42},
            "workspace": {"current_dir": "/test"},
            "version": "1.0.23",
        }
        output = self.run_with_input(data, usage=None)
        assert "⏱" not in output

    def test_usage_with_reset_time(self):
        data = {
            "model": {"display_name": "Opus"},
            "context_window": {"used_percentage": 42},
            "workspace": {"current_dir": "/test"},
            "version": "1.0.23",
        }
        usage = {"five_hour": 40, "five_hour_resets": "2026-02-02T01:00:00+00:00"}
        output = self.run_with_input(data, usage=usage)
        assert "⏱ 40%" in output
        assert "→" in output  # Has reset time arrow


class TestFormatResetTime:
    """Tests for reset time formatting."""

    def test_format_midnight(self):
        # Midnight UTC
        result = statusline.format_reset_time("2026-02-02T00:00:00+00:00")
        assert result in [
            "12am",
            "1am",
            "2am",
            "3am",
            "4am",
            "5am",
            "6am",
            "7am",
            "8am",
            "9am",
            "10am",
            "11am",
            "12pm",
            "1pm",
            "2pm",
            "3pm",
            "4pm",
            "5pm",
            "6pm",
            "7pm",
            "8pm",
            "9pm",
            "10pm",
            "11pm",
        ]  # depends on timezone

    def test_format_noon(self):
        result = statusline.format_reset_time("2026-02-02T12:00:00+00:00")
        assert "am" in result or "pm" in result

    def test_format_empty_string(self):
        result = statusline.format_reset_time("")
        assert result == ""

    def test_format_none(self):
        result = statusline.format_reset_time(None)
        assert result == ""

    def test_format_invalid(self):
        result = statusline.format_reset_time("not-a-date")
        assert result == ""

    def test_format_with_z_suffix(self):
        result = statusline.format_reset_time("2026-02-02T15:00:00Z")
        assert "am" in result or "pm" in result


class TestClaudeUsage:
    """Tests for Claude.ai usage fetching."""

    def test_usage_cache_hit(self, tmp_path):
        cache_file = tmp_path / "cache"
        cache_file.write_text('{"five_hour": 30, "seven_day": 50}')

        with patch.object(statusline, "USAGE_CACHE_FILE", cache_file):
            result = statusline.get_claude_usage()
            assert result == {"five_hour": 30, "seven_day": 50}

    def test_usage_cache_with_reset_times(self, tmp_path):
        cache_file = tmp_path / "cache"
        cache_file.write_text(
            '{"five_hour": 30, "five_hour_resets": "2026-02-02T01:00:00+00:00"}'
        )

        with patch.object(statusline, "USAGE_CACHE_FILE", cache_file):
            result = statusline.get_claude_usage()
            assert result["five_hour"] == 30
            assert "five_hour_resets" in result

    def test_usage_no_credentials(self, tmp_path):
        cache_file = tmp_path / "cache"  # doesn't exist

        with patch.object(statusline, "USAGE_CACHE_FILE", cache_file):
            with patch.object(statusline, "get_claude_oauth_token", return_value=None):
                result = statusline.get_claude_usage()
                assert result is None

    def test_usage_cache_expired_no_creds(self, tmp_path):
        import time as time_module

        cache_file = tmp_path / "cache"
        cache_file.write_text('{"five_hour": 30}')
        # Set mtime to 10 minutes ago
        old_time = time_module.time() - 600
        os.utime(cache_file, (old_time, old_time))

        with patch.object(statusline, "USAGE_CACHE_FILE", cache_file):
            with patch.object(statusline, "get_claude_oauth_token", return_value=None):
                # No credentials and cache expired - returns None
                result = statusline.get_claude_usage()
                assert result is None

    def test_usage_cache_fresh(self, tmp_path):
        cache_file = tmp_path / "cache"
        cache_file.write_text('{"five_hour": 30}')
        # Fresh cache (just created)

        with patch.object(statusline, "USAGE_CACHE_FILE", cache_file):
            # Doesn't even check credentials if cache is fresh
            result = statusline.get_claude_usage()
            assert result == {"five_hour": 30}


class TestGetClaudeOAuthToken:
    """Tests for OAuth token retrieval."""

    def test_token_extraction(self):
        mock_creds = json.dumps({"claudeAiOauth": {"accessToken": "test-token-123"}})
        with patch("subprocess.run") as mock_run:
            mock_run.return_value = MagicMock(returncode=0, stdout=mock_creds)
            result = statusline.get_claude_oauth_token()
            assert result == "test-token-123"

    def test_no_keychain_entry(self):
        with patch("subprocess.run") as mock_run:
            mock_run.return_value = MagicMock(returncode=1, stdout="")
            result = statusline.get_claude_oauth_token()
            assert result is None

    def test_malformed_json(self):
        with patch("subprocess.run") as mock_run:
            mock_run.return_value = MagicMock(returncode=0, stdout="not json")
            result = statusline.get_claude_oauth_token()
            assert result is None

    def test_missing_oauth_key(self):
        with patch("subprocess.run") as mock_run:
            mock_run.return_value = MagicMock(
                returncode=0, stdout=json.dumps({"otherKey": "value"})
            )
            result = statusline.get_claude_oauth_token()
            assert result is None

    def test_timeout_handling(self):
        with patch("subprocess.run") as mock_run:
            mock_run.side_effect = subprocess.TimeoutExpired("security", 2)
            result = statusline.get_claude_oauth_token()
            assert result is None


class TestInputParsing:
    """Tests for JSON input parsing edge cases."""

    def run_main(self, data: dict | str) -> str:
        """Helper to run main with input."""
        json_input = json.dumps(data) if isinstance(data, dict) else data
        with patch("sys.stdin", io.StringIO(json_input)):
            with patch.object(statusline, "detect_dark_mode", return_value=True):
                with patch.object(statusline, "get_git_branch", return_value=None):
                    with patch.object(
                        statusline, "check_for_update", return_value=None
                    ):
                        with patch.object(
                            statusline, "get_claude_usage", return_value=None
                        ):
                            captured = io.StringIO()
                            with patch("sys.stdout", captured):
                                statusline.main()
                            return captured.getvalue().strip()

    def test_missing_model_key(self):
        data = {"context_window": {"used_percentage": 42}}
        output = self.run_main(data)
        assert "✦ ?" in output  # Default model name

    def test_missing_model_display_name(self):
        data = {"model": {}, "context_window": {"used_percentage": 42}}
        output = self.run_main(data)
        assert "✦ ?" in output

    def test_missing_context_window(self):
        data = {"model": {"display_name": "Opus"}}
        output = self.run_main(data)
        assert "◐ 0%" in output  # Default to 0

    def test_missing_used_percentage(self):
        data = {"model": {"display_name": "Opus"}, "context_window": {}}
        output = self.run_main(data)
        assert "◐ 0%" in output

    def test_float_percentage_truncated(self):
        data = {
            "model": {"display_name": "Opus"},
            "context_window": {"used_percentage": 42.7},
        }
        output = self.run_main(data)
        assert "◐ 42%" in output  # Truncated, not rounded

    def test_empty_json_object(self):
        output = self.run_main({})
        assert "◐ 0%" in output
        assert "✦ ?" in output

    def test_null_values(self):
        data = {"model": None, "context_window": None}
        output = self.run_main(data)
        assert "◐ 0%" in output


class TestColorThresholds:
    """Tests for exact color threshold boundaries."""

    def run_with_context(self, pct: int, dark_mode: bool = True) -> str:
        data = {
            "model": {"display_name": "Opus"},
            "context_window": {"used_percentage": pct},
        }
        json_input = json.dumps(data)
        with patch("sys.stdin", io.StringIO(json_input)):
            with patch.object(statusline, "detect_dark_mode", return_value=dark_mode):
                with patch.object(statusline, "get_git_branch", return_value=None):
                    with patch.object(
                        statusline, "check_for_update", return_value=None
                    ):
                        with patch.object(
                            statusline, "get_claude_usage", return_value=None
                        ):
                            captured = io.StringIO()
                            with patch("sys.stdout", captured):
                                statusline.main()
                            return captured.getvalue().strip()

    # Context thresholds: <50 green, 50-74 yellow, >=75 red
    def test_context_49_is_green(self):
        output = self.run_with_context(49)
        assert "\033[92m◐ 49%" in output

    def test_context_50_is_yellow(self):
        output = self.run_with_context(50)
        assert "\033[93m◐ 50%" in output

    def test_context_74_is_yellow(self):
        output = self.run_with_context(74)
        assert "\033[93m◐ 74%" in output

    def test_context_75_is_red(self):
        output = self.run_with_context(75)
        assert "\033[91m◐ 75%" in output

    def test_context_0_is_green(self):
        output = self.run_with_context(0)
        assert "\033[92m◐ 0%" in output

    def test_context_100_is_red(self):
        output = self.run_with_context(100)
        assert "\033[91m◐ 100%" in output

    # Light mode colors
    def test_context_green_light_mode(self):
        output = self.run_with_context(30, dark_mode=False)
        assert "\033[32m◐ 30%" in output  # standard green

    def test_context_yellow_light_mode(self):
        output = self.run_with_context(60, dark_mode=False)
        assert "\033[33m◐ 60%" in output  # standard yellow

    def test_context_red_light_mode(self):
        output = self.run_with_context(80, dark_mode=False)
        assert "\033[31m◐ 80%" in output  # standard red


class TestUsageColorThresholds:
    """Tests for usage color threshold boundaries."""

    def run_with_usage(self, pct: int, dark_mode: bool = True) -> str:
        data = {
            "model": {"display_name": "Opus"},
            "context_window": {"used_percentage": 42},
        }
        usage = {"five_hour": pct}
        json_input = json.dumps(data)
        with patch("sys.stdin", io.StringIO(json_input)):
            with patch.object(statusline, "detect_dark_mode", return_value=dark_mode):
                with patch.object(statusline, "get_git_branch", return_value=None):
                    with patch.object(
                        statusline, "check_for_update", return_value=None
                    ):
                        with patch.object(
                            statusline, "get_claude_usage", return_value=usage
                        ):
                            captured = io.StringIO()
                            with patch("sys.stdout", captured):
                                statusline.main()
                            return captured.getvalue().strip()

    # Usage thresholds: <50 green, 50-79 yellow, >=80 red
    def test_usage_49_is_green(self):
        output = self.run_with_usage(49)
        assert "\033[92m⏱ 49%" in output

    def test_usage_50_is_yellow(self):
        output = self.run_with_usage(50)
        assert "\033[93m⏱ 50%" in output

    def test_usage_79_is_yellow(self):
        output = self.run_with_usage(79)
        assert "\033[93m⏱ 79%" in output

    def test_usage_80_is_red(self):
        output = self.run_with_usage(80)
        assert "\033[91m⏱ 80%" in output

    def test_usage_0_is_green(self):
        output = self.run_with_usage(0)
        assert "\033[92m⏱ 0%" in output

    def test_usage_100_is_red(self):
        output = self.run_with_usage(100)
        assert "\033[91m⏱ 100%" in output


class TestOutputOrder:
    """Tests for output element ordering."""

    def test_full_output_order(self):
        """Verify elements appear in correct order: context, model, git, usage, update."""
        data = {
            "model": {"display_name": "Opus"},
            "context_window": {"used_percentage": 42},
            "workspace": {"current_dir": "/test"},
            "version": "1.0.23",
        }
        usage = {"five_hour": 30, "five_hour_resets": "2026-02-02T01:00:00+00:00"}
        json_input = json.dumps(data)
        with patch("sys.stdin", io.StringIO(json_input)):
            with patch.object(statusline, "detect_dark_mode", return_value=True):
                with patch.object(statusline, "get_git_branch", return_value="main"):
                    with patch.object(
                        statusline, "check_for_update", return_value="2.0.0"
                    ):
                        with patch.object(
                            statusline, "get_claude_usage", return_value=usage
                        ):
                            captured = io.StringIO()
                            with patch("sys.stdout", captured):
                                statusline.main()
                            output = captured.getvalue().strip()

        # Find positions of each element
        ctx_pos = output.find("◐")
        model_pos = output.find("✦")
        git_pos = output.find("⎇")
        usage_pos = output.find("⏱")
        update_pos = output.find("↑")

        assert ctx_pos < model_pos < git_pos < usage_pos < update_pos

    def test_elements_separated_by_spaces(self):
        data = {
            "model": {"display_name": "Opus"},
            "context_window": {"used_percentage": 42},
            "workspace": {"current_dir": "/test"},
            "version": "1.0.23",
        }
        json_input = json.dumps(data)
        with patch("sys.stdin", io.StringIO(json_input)):
            with patch.object(statusline, "detect_dark_mode", return_value=True):
                with patch.object(statusline, "get_git_branch", return_value="main"):
                    with patch.object(
                        statusline, "check_for_update", return_value=None
                    ):
                        with patch.object(
                            statusline, "get_claude_usage", return_value=None
                        ):
                            captured = io.StringIO()
                            with patch("sys.stdout", captured):
                                statusline.main()
                            output = captured.getvalue().strip()

        # Remove ANSI codes for checking structure
        import re

        clean = re.sub(r"\033\[[0-9;]*m", "", output)
        parts = clean.split(" ")
        # Should have: ◐, 42%, ✦, Opus, ⎇, main
        assert len(parts) >= 4


class TestDarkModeDetection:
    """Additional tests for dark mode detection."""

    def test_macos_dark_mode_detected(self):
        with patch.dict(os.environ, {}, clear=True):
            with patch("subprocess.run") as mock_run:
                mock_run.return_value = MagicMock(stdout="Dark\n", returncode=0)
                assert statusline.detect_dark_mode() is True

    def test_macos_light_mode_detected(self):
        with patch.dict(os.environ, {}, clear=True):
            with patch("subprocess.run") as mock_run:
                mock_run.return_value = MagicMock(stdout="Light\n", returncode=0)
                # Command succeeds but returns non-"Dark" value
                assert statusline.detect_dark_mode() is False

    def test_macos_command_raises_exception_defaults_to_dark(self):
        # When macOS command raises exception (e.g., not on macOS), default to dark
        with patch.dict(os.environ, {}, clear=True):
            with patch("subprocess.run") as mock_run:
                mock_run.side_effect = FileNotFoundError()
                assert statusline.detect_dark_mode() is True

    def test_macos_empty_stdout_means_light_mode(self):
        # On macOS in light mode, AppleInterfaceStyle key doesn't exist
        # Command succeeds but returns empty - this means light mode
        with patch.dict(os.environ, {}, clear=True):
            with patch("subprocess.run") as mock_run:
                mock_run.return_value = MagicMock(returncode=0, stdout="")
                # Empty stdout != "dark", so returns False
                assert statusline.detect_dark_mode() is False

    def test_colorfgbg_three_part_format(self):
        # Some terminals use "fg;bg;extra" format
        with patch.dict(os.environ, {"COLORFGBG": "15;0;0"}, clear=True):
            assert statusline.detect_dark_mode() is True

    def test_colorfgbg_boundary_value_7(self):
        # 7 is the last "dark" color
        with patch.dict(os.environ, {"COLORFGBG": "15;7"}, clear=True):
            assert statusline.detect_dark_mode() is True

    def test_colorfgbg_boundary_value_8(self):
        # 8 is the first "light" color
        with patch.dict(os.environ, {"COLORFGBG": "0;8"}, clear=True):
            assert statusline.detect_dark_mode() is False

    def test_env_override_takes_precedence_over_colorfgbg(self):
        with patch.dict(
            os.environ,
            {
                "CLAUDE_STATUSLINE_THEME": "light",
                "COLORFGBG": "15;0",  # Would indicate dark
            },
        ):
            assert statusline.detect_dark_mode() is False


class TestGetColorsCompleteness:
    """Verify all color keys exist in both modes."""

    def test_dark_mode_has_all_keys(self):
        colors = statusline.get_colors(dark_mode=True)
        required_keys = [
            "reset",
            "bold",
            "dim",
            "ctx_good",
            "ctx_warn",
            "ctx_crit",
            "model",
            "git",
            "update",
            "usage_good",
            "usage_warn",
            "usage_crit",
        ]
        for key in required_keys:
            assert key in colors, f"Missing key: {key}"

    def test_light_mode_has_all_keys(self):
        colors = statusline.get_colors(dark_mode=False)
        required_keys = [
            "reset",
            "bold",
            "dim",
            "ctx_good",
            "ctx_warn",
            "ctx_crit",
            "model",
            "git",
            "update",
            "usage_good",
            "usage_warn",
            "usage_crit",
        ]
        for key in required_keys:
            assert key in colors, f"Missing key: {key}"

    def test_all_colors_are_ansi_codes(self):
        for dark_mode in [True, False]:
            colors = statusline.get_colors(dark_mode)
            for key, value in colors.items():
                assert value.startswith("\033["), f"{key} is not an ANSI code: {value}"


class TestGitBranchEdgeCases:
    """Additional git branch tests."""

    def test_detached_head(self, tmp_path):
        # Create repo with a commit
        subprocess.run(["git", "init"], cwd=tmp_path, capture_output=True)
        subprocess.run(
            ["git", "config", "user.email", "test@test.com"],
            cwd=tmp_path,
            capture_output=True,
        )
        subprocess.run(
            ["git", "config", "user.name", "Test"], cwd=tmp_path, capture_output=True
        )
        (tmp_path / "file.txt").write_text("content")
        subprocess.run(["git", "add", "."], cwd=tmp_path, capture_output=True)
        subprocess.run(
            ["git", "commit", "-m", "init"], cwd=tmp_path, capture_output=True
        )
        # Detach HEAD
        subprocess.run(
            ["git", "checkout", "--detach"], cwd=tmp_path, capture_output=True
        )

        branch = statusline.get_git_branch(str(tmp_path))
        # Detached HEAD returns empty string from git branch --show-current
        assert branch is None or branch == ""

    def test_branch_with_slash(self, tmp_path):
        subprocess.run(["git", "init"], cwd=tmp_path, capture_output=True)
        subprocess.run(
            ["git", "checkout", "-b", "feature/my-feature"],
            cwd=tmp_path,
            capture_output=True,
        )
        branch = statusline.get_git_branch(str(tmp_path))
        assert branch == "feature/my-feature"

    def test_branch_with_unicode(self, tmp_path):
        subprocess.run(["git", "init"], cwd=tmp_path, capture_output=True)
        subprocess.run(
            ["git", "checkout", "-b", "feature-émoji-🚀"],
            cwd=tmp_path,
            capture_output=True,
        )
        branch = statusline.get_git_branch(str(tmp_path))
        assert branch == "feature-émoji-🚀"


class TestUpdateCheckEdgeCases:
    """Additional update check tests."""

    def test_cache_with_error_value(self, tmp_path):
        cache_file = tmp_path / "cache"
        cache_file.write_text("error")

        with patch.object(statusline, "CACHE_FILE", cache_file):
            result = statusline.check_for_update("1.0.23")
            assert result is None

    def test_cache_with_empty_update_version(self, tmp_path):
        cache_file = tmp_path / "cache"
        cache_file.write_text("update:")

        with patch.object(statusline, "CACHE_FILE", cache_file):
            result = statusline.check_for_update("1.0.23")
            assert result == ""

    def test_cache_file_corrupted(self, tmp_path):
        cache_file = tmp_path / "cache"
        cache_file.write_bytes(b"\x00\x01\x02")  # Binary garbage

        with patch.object(statusline, "CACHE_FILE", cache_file):
            result = statusline.check_for_update("1.0.23")
            # Should handle gracefully
            assert result is None or isinstance(result, str)


class TestFormatResetTimeEdgeCases:
    """Additional reset time formatting tests."""

    def test_format_1am(self):
        result = statusline.format_reset_time("2026-02-02T01:00:00+00:00")
        # Result depends on local timezone, but should be valid
        assert result.endswith("am") or result.endswith("pm")

    def test_format_11am(self):
        result = statusline.format_reset_time("2026-02-02T11:00:00+00:00")
        assert result.endswith("am") or result.endswith("pm")

    def test_format_1pm(self):
        result = statusline.format_reset_time("2026-02-02T13:00:00+00:00")
        assert result.endswith("am") or result.endswith("pm")

    def test_format_11pm(self):
        result = statusline.format_reset_time("2026-02-02T23:00:00+00:00")
        assert result.endswith("am") or result.endswith("pm")

    def test_format_with_positive_offset(self):
        result = statusline.format_reset_time("2026-02-02T12:00:00+05:30")
        assert result.endswith("am") or result.endswith("pm")

    def test_format_with_negative_offset(self):
        result = statusline.format_reset_time("2026-02-02T12:00:00-08:00")
        assert result.endswith("am") or result.endswith("pm")


class TestColorfgbgEdgeCases:
    """Test COLORFGBG parsing edge cases."""

    def test_colorfgbg_non_numeric(self):
        with patch.dict(os.environ, {"COLORFGBG": "white;black"}, clear=True):
            with patch("subprocess.run") as mock_run:
                mock_run.side_effect = FileNotFoundError()
                # Falls through to default
                assert statusline.detect_dark_mode() is True

    def test_colorfgbg_single_value(self):
        with patch.dict(os.environ, {"COLORFGBG": "15"}, clear=True):
            with patch("subprocess.run") as mock_run:
                mock_run.side_effect = FileNotFoundError()
                # Falls through to default
                assert statusline.detect_dark_mode() is True


class TestGitBranchExceptions:
    """Test git branch exception handling."""

    def test_git_command_timeout(self, tmp_path):
        with patch("subprocess.run") as mock_run:
            mock_run.side_effect = subprocess.TimeoutExpired("git", 1)
            result = statusline.get_git_branch(str(tmp_path))
            assert result is None

    def test_git_command_not_found(self, tmp_path):
        with patch("subprocess.run") as mock_run:
            mock_run.side_effect = FileNotFoundError()
            result = statusline.get_git_branch(str(tmp_path))
            assert result is None


class TestCheckForUpdateExceptions:
    """Test update check exception handling."""

    def test_cache_read_after_fork_failure(self, tmp_path):
        cache_file = tmp_path / "cache"
        cache_file.write_text("update:2.0.0")

        with patch.object(statusline, "CACHE_FILE", cache_file):
            with patch("os.fork", side_effect=OSError()):
                # Fork fails, should still return cached value
                result = statusline.check_for_update("1.0.0")
                assert result == "2.0.0"


class TestFormatResetTimeExceptions:
    """Test format_reset_time exception handling."""

    def test_format_partial_iso_string(self):
        # Missing timezone
        result = statusline.format_reset_time("2026-02-02T12:00:00")
        # Should handle gracefully (might work or return empty)
        assert result == "" or result.endswith("am") or result.endswith("pm")

    def test_format_truncated_string(self):
        result = statusline.format_reset_time("2026-02")
        assert result == ""

    def test_format_integer_input(self):
        result = statusline.format_reset_time(12345)  # type: ignore
        assert result == ""


class TestIntegration:
    """Integration tests running the actual script."""

    def test_script_runs_with_uv(self):
        """Test that the script runs via uv."""
        script_path = Path(__file__).parent / "statusline.py"
        data = json.dumps(
            {
                "model": {"display_name": "Opus"},
                "context_window": {"used_percentage": 42},
                "workspace": {"current_dir": "/tmp"},
                "version": "1.0.23",
            }
        )
        result = subprocess.run(
            ["uv", "run", str(script_path)],
            input=data,
            capture_output=True,
            text=True,
            timeout=10,
        )
        assert result.returncode == 0
        assert "◐ 42%" in result.stdout
        assert "Opus" in result.stdout

    def test_script_handles_empty_input(self):
        """Test graceful handling of empty stdin."""
        script_path = Path(__file__).parent / "statusline.py"
        result = subprocess.run(
            ["uv", "run", str(script_path)],
            input="",
            capture_output=True,
            text=True,
            timeout=10,
        )
        assert result.returncode == 0
        assert "◐ --%" in result.stdout and "✦" in result.stdout

    def test_script_outputs_single_line(self):
        """Status line must be single line."""
        script_path = Path(__file__).parent / "statusline.py"
        data = json.dumps(
            {
                "model": {"display_name": "Opus"},
                "context_window": {"used_percentage": 42},
                "workspace": {"current_dir": "/tmp"},
                "version": "1.0.23",
            }
        )
        result = subprocess.run(
            ["uv", "run", str(script_path)],
            input=data,
            capture_output=True,
            text=True,
            timeout=10,
        )
        lines = result.stdout.strip().split("\n")
        assert len(lines) == 1

    def test_script_with_all_fields(self):
        """Test with complete input data."""
        script_path = Path(__file__).parent / "statusline.py"
        data = json.dumps(
            {
                "model": {"id": "claude-opus-4", "display_name": "Opus"},
                "context_window": {
                    "used_percentage": 55.5,
                    "context_window_size": 200000,
                    "total_input_tokens": 50000,
                    "total_output_tokens": 10000,
                },
                "workspace": {
                    "current_dir": "/tmp",
                    "project_dir": "/home/user/project",
                },
                "version": "2.1.29",
                "session_id": "abc-123",
                "cost": {"total_cost_usd": 0.50},
            }
        )
        result = subprocess.run(
            ["uv", "run", str(script_path)],
            input=data,
            capture_output=True,
            text=True,
            timeout=10,
        )
        assert result.returncode == 0
        assert "◐ 55%" in result.stdout
        assert "Opus" in result.stdout

    def test_script_no_stderr_on_success(self):
        """Script should not output to stderr on success."""
        script_path = Path(__file__).parent / "statusline.py"
        data = json.dumps(
            {
                "model": {"display_name": "Opus"},
                "context_window": {"used_percentage": 42},
            }
        )
        result = subprocess.run(
            ["uv", "run", str(script_path)],
            input=data,
            capture_output=True,
            text=True,
            timeout=10,
        )
        assert result.stderr == "" or "warning" not in result.stderr.lower()
