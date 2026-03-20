use std::process::Command;

pub struct Colors {
    pub reset: &'static str,
    pub bold: &'static str,
    pub dim: &'static str,
    pub ctx_good: &'static str,
    pub ctx_warn: &'static str,
    pub ctx_crit: &'static str,
    pub model: &'static str,
    pub git: &'static str,
    pub update: &'static str,
    pub usage_good: &'static str,
    pub usage_warn: &'static str,
    pub usage_crit: &'static str,
}

const DARK_COLORS: Colors = Colors {
    reset: "\x1b[0m",
    bold: "\x1b[1m",
    dim: "\x1b[2m",
    ctx_good: "\x1b[92m",
    ctx_warn: "\x1b[93m",
    ctx_crit: "\x1b[91m",
    model: "\x1b[96m",
    git: "\x1b[90m",
    update: "\x1b[93m",
    usage_good: "\x1b[92m",
    usage_warn: "\x1b[93m",
    usage_crit: "\x1b[91m",
};

const LIGHT_COLORS: Colors = Colors {
    reset: "\x1b[0m",
    bold: "\x1b[1m",
    dim: "\x1b[2m",
    ctx_good: "\x1b[32m",
    ctx_warn: "\x1b[33m",
    ctx_crit: "\x1b[31m",
    model: "\x1b[34m",
    git: "\x1b[90m",
    update: "\x1b[33m",
    usage_good: "\x1b[32m",
    usage_warn: "\x1b[33m",
    usage_crit: "\x1b[31m",
};

pub fn get_colors(dark_mode: bool) -> &'static Colors {
    if dark_mode {
        &DARK_COLORS
    } else {
        &LIGHT_COLORS
    }
}

pub fn detect_dark_mode() -> bool {
    detect_dark_mode_from_env(
        std::env::var("CLAUDE_STATUSLINE_THEME").ok().as_deref(),
        std::env::var("COLORFGBG").ok().as_deref(),
    )
}

/// Testable core: takes env values as parameters.
pub fn detect_dark_mode_from_env(theme: Option<&str>, colorfgbg: Option<&str>) -> bool {
    // 1. Explicit override
    if let Some(t) = theme {
        match t.to_lowercase().as_str() {
            "dark" => return true,
            "light" => return false,
            _ => {}
        }
    }

    // 2. COLORFGBG (format: "fg;bg")
    if let Some(cfg) = colorfgbg {
        if !cfg.is_empty() {
            if let Some(bg_str) = cfg.rsplit(';').next() {
                if let Ok(bg) = bg_str.parse::<u32>() {
                    return bg < 8;
                }
            }
        }
    }

    // 3. macOS system appearance
    if let Ok(output) = Command::new("defaults")
        .args(["read", "-g", "AppleInterfaceStyle"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        return stdout.trim().eq_ignore_ascii_case("dark");
    }

    // Default to dark
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explicit_dark_override() {
        assert!(detect_dark_mode_from_env(Some("dark"), None));
    }

    #[test]
    fn test_explicit_light_override() {
        assert!(!detect_dark_mode_from_env(Some("light"), None));
    }

    #[test]
    fn test_explicit_override_case_insensitive() {
        assert!(detect_dark_mode_from_env(Some("DARK"), None));
        assert!(!detect_dark_mode_from_env(Some("Light"), None));
    }

    #[test]
    fn test_colorfgbg_dark_background() {
        assert!(detect_dark_mode_from_env(None, Some("15;0")));
    }

    #[test]
    fn test_colorfgbg_light_background() {
        assert!(!detect_dark_mode_from_env(None, Some("0;15")));
    }

    #[test]
    fn test_colorfgbg_invalid_format() {
        // Non-numeric falls through to macOS/default
        let result = detect_dark_mode_from_env(None, Some("abc"));
        // Just check it doesn't panic; result depends on macOS state
        let _ = result;
    }

    #[test]
    fn test_colorfgbg_three_part_format() {
        assert!(detect_dark_mode_from_env(None, Some("15;0;0")));
    }

    #[test]
    fn test_colorfgbg_boundary_value_7() {
        assert!(detect_dark_mode_from_env(None, Some("0;7")));
    }

    #[test]
    fn test_colorfgbg_boundary_value_8() {
        assert!(!detect_dark_mode_from_env(None, Some("0;8")));
    }

    #[test]
    fn test_env_override_takes_precedence_over_colorfgbg() {
        assert!(!detect_dark_mode_from_env(Some("light"), Some("15;0")));
    }

    #[test]
    fn test_dark_mode_has_all_fields() {
        let c = get_colors(true);
        assert!(!c.reset.is_empty());
        assert!(!c.bold.is_empty());
        assert!(!c.dim.is_empty());
        assert!(!c.ctx_good.is_empty());
        assert!(!c.ctx_warn.is_empty());
        assert!(!c.ctx_crit.is_empty());
        assert!(!c.model.is_empty());
        assert!(!c.git.is_empty());
        assert!(!c.update.is_empty());
        assert!(!c.usage_good.is_empty());
        assert!(!c.usage_warn.is_empty());
        assert!(!c.usage_crit.is_empty());
    }

    #[test]
    fn test_light_mode_has_all_fields() {
        let c = get_colors(false);
        assert!(!c.reset.is_empty());
        assert!(!c.model.is_empty());
    }

    #[test]
    fn test_all_colors_are_ansi_codes() {
        for dark_mode in [true, false] {
            let c = get_colors(dark_mode);
            for code in [
                c.reset,
                c.bold,
                c.dim,
                c.ctx_good,
                c.ctx_warn,
                c.ctx_crit,
                c.model,
                c.git,
                c.update,
                c.usage_good,
                c.usage_warn,
                c.usage_crit,
            ] {
                assert!(code.starts_with("\x1b["), "Not an ANSI code: {:?}", code);
            }
        }
    }

    #[test]
    fn test_colorfgbg_non_numeric() {
        let _ = detect_dark_mode_from_env(None, Some("abc;def"));
    }

    #[test]
    fn test_colorfgbg_single_value() {
        // Single value without semicolon - tries to parse as bg
        let _ = detect_dark_mode_from_env(None, Some("5"));
    }
}
