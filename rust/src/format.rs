use crate::theme::Colors;
use crate::usage::UsageData;

/// Select context fill-level icon based on percentage.
pub fn context_icon(pct: u32) -> char {
    match pct {
        0..25 => '◔',
        25..50 => '◑',
        50..75 => '◕',
        _ => '●',
    }
}

/// Get context color from Colors based on percentage thresholds.
pub fn context_color(c: &Colors, pct: u32) -> &'static str {
    match pct {
        0..50 => c.ctx_good,
        50..75 => c.ctx_warn,
        _ => c.ctx_crit,
    }
}

/// Get usage color from Colors based on percentage thresholds.
pub fn usage_color(c: &Colors, pct: u32) -> &'static str {
    match pct {
        0..50 => c.usage_good,
        50..80 => c.usage_warn,
        _ => c.usage_crit,
    }
}

/// Format ISO timestamp to local time like "2am" or "3pm".
pub fn format_reset_time(iso_timestamp: &str) -> String {
    if iso_timestamp.is_empty() {
        return String::new();
    }

    use chrono::{DateTime, Local, Timelike};

    let dt = match DateTime::parse_from_rfc3339(iso_timestamp) {
        Ok(dt) => dt,
        Err(_) => {
            // Try with Z suffix handling
            let normalized = iso_timestamp.replace('Z', "+00:00");
            match DateTime::parse_from_rfc3339(&normalized) {
                Ok(dt) => dt,
                Err(_) => return String::new(),
            }
        }
    };

    let local_dt: DateTime<Local> = dt.into();

    // Round up to next hour if there are any minutes/seconds
    let local_dt = if local_dt.minute() > 0 || local_dt.second() > 0 {
        local_dt + chrono::Duration::hours(1)
    } else {
        local_dt
    };

    let hour = local_dt.hour();
    match hour {
        0 => "12am".to_string(),
        1..12 => format!("{}am", hour),
        12 => "12pm".to_string(),
        _ => format!("{}pm", hour - 12),
    }
}

/// Build the full status line string.
pub fn build_status_line(
    context_pct: u32,
    model: &str,
    colors: &Colors,
    git: (Option<&str>, bool),
    usage: Option<&UsageData>,
    update: Option<&str>,
) -> String {
    let mut parts = Vec::new();

    // Context percentage with fill-level icon
    let cc = context_color(colors, context_pct);
    let icon = context_icon(context_pct);
    parts.push(format!("{}{}  {}%{}", cc, icon, context_pct, colors.reset));

    // Model
    parts.push(format!("{}✦ {}{}", colors.model, model, colors.reset));

    // Git branch
    if let (Some(branch), dirty) = git {
        let dirty_mark = if dirty {
            format!("{}*{}", colors.ctx_warn, colors.reset)
        } else {
            String::new()
        };
        parts.push(format!(
            "{}⎇ {}{}{}",
            colors.git, branch, dirty_mark, colors.reset
        ));
    }

    // Usage
    if let Some(u) = usage {
        let five_h = u.five_hour as u32;
        let uc = usage_color(colors, five_h);
        let reset_str = format_reset_time(&u.five_hour_resets);
        let usage_str = if reset_str.is_empty() {
            format!("⏱ {}%", five_h)
        } else {
            format!("⏱ {}%→{}", five_h, reset_str)
        };
        parts.push(format!("{}{}{}", uc, usage_str, colors.reset));
    }

    // Update
    if let Some(version) = update {
        parts.push(format!("{}↑{}{}", colors.update, version, colors.reset));
    }

    parts.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_icon_low() {
        assert_eq!(context_icon(0), '◔');
        assert_eq!(context_icon(24), '◔');
    }

    #[test]
    fn test_context_icon_medium_low() {
        assert_eq!(context_icon(25), '◑');
        assert_eq!(context_icon(49), '◑');
    }

    #[test]
    fn test_context_icon_medium_high() {
        assert_eq!(context_icon(50), '◕');
        assert_eq!(context_icon(74), '◕');
    }

    #[test]
    fn test_context_icon_high() {
        assert_eq!(context_icon(75), '●');
        assert_eq!(context_icon(100), '●');
    }

    // Context color thresholds
    #[test]
    fn test_context_49_is_green() {
        let c = crate::theme::get_colors(true);
        assert_eq!(context_color(c, 49), c.ctx_good);
    }

    #[test]
    fn test_context_50_is_yellow() {
        let c = crate::theme::get_colors(true);
        assert_eq!(context_color(c, 50), c.ctx_warn);
    }

    #[test]
    fn test_context_74_is_yellow() {
        let c = crate::theme::get_colors(true);
        assert_eq!(context_color(c, 74), c.ctx_warn);
    }

    #[test]
    fn test_context_75_is_red() {
        let c = crate::theme::get_colors(true);
        assert_eq!(context_color(c, 75), c.ctx_crit);
    }

    #[test]
    fn test_context_0_is_green() {
        let c = crate::theme::get_colors(true);
        assert_eq!(context_color(c, 0), c.ctx_good);
    }

    #[test]
    fn test_context_100_is_red() {
        let c = crate::theme::get_colors(true);
        assert_eq!(context_color(c, 100), c.ctx_crit);
    }

    #[test]
    fn test_context_green_light_mode() {
        let c = crate::theme::get_colors(false);
        assert_eq!(context_color(c, 30), c.ctx_good);
    }

    #[test]
    fn test_context_yellow_light_mode() {
        let c = crate::theme::get_colors(false);
        assert_eq!(context_color(c, 60), c.ctx_warn);
    }

    #[test]
    fn test_context_red_light_mode() {
        let c = crate::theme::get_colors(false);
        assert_eq!(context_color(c, 80), c.ctx_crit);
    }

    // Usage color thresholds
    #[test]
    fn test_usage_49_is_green() {
        let c = crate::theme::get_colors(true);
        assert_eq!(usage_color(c, 49), c.usage_good);
    }

    #[test]
    fn test_usage_50_is_yellow() {
        let c = crate::theme::get_colors(true);
        assert_eq!(usage_color(c, 50), c.usage_warn);
    }

    #[test]
    fn test_usage_79_is_yellow() {
        let c = crate::theme::get_colors(true);
        assert_eq!(usage_color(c, 79), c.usage_warn);
    }

    #[test]
    fn test_usage_80_is_red() {
        let c = crate::theme::get_colors(true);
        assert_eq!(usage_color(c, 80), c.usage_crit);
    }

    #[test]
    fn test_usage_0_is_green() {
        let c = crate::theme::get_colors(true);
        assert_eq!(usage_color(c, 0), c.usage_good);
    }

    #[test]
    fn test_usage_100_is_red() {
        let c = crate::theme::get_colors(true);
        assert_eq!(usage_color(c, 100), c.usage_crit);
    }

    // Format reset time
    #[test]
    fn test_format_midnight() {
        // Midnight UTC = depends on local timezone, but test the parsing works
        let result = format_reset_time("2025-01-01T00:00:00+00:00");
        assert!(!result.is_empty());
        assert!(result.ends_with("am") || result.ends_with("pm"));
    }

    #[test]
    fn test_format_empty_string() {
        assert_eq!(format_reset_time(""), "");
    }

    #[test]
    fn test_format_invalid() {
        assert_eq!(format_reset_time("not-a-date"), "");
    }

    #[test]
    fn test_format_with_z_suffix() {
        let result = format_reset_time("2025-01-01T12:00:00Z");
        assert!(!result.is_empty());
    }

    #[test]
    fn test_format_noon() {
        // Noon UTC — result depends on local tz but should parse
        let result = format_reset_time("2025-01-01T12:00:00+00:00");
        assert!(!result.is_empty());
    }

    #[test]
    fn test_format_truncated_string() {
        assert_eq!(format_reset_time("2025-01"), "");
    }

    #[test]
    fn test_format_partial_iso_string() {
        assert_eq!(format_reset_time("2025-01-01"), "");
    }

    #[test]
    fn test_format_integer_input() {
        assert_eq!(format_reset_time("12345"), "");
    }

    #[test]
    fn test_format_with_positive_offset() {
        let result = format_reset_time("2025-01-01T15:00:00+05:00");
        assert!(!result.is_empty());
    }

    #[test]
    fn test_format_with_negative_offset() {
        let result = format_reset_time("2025-01-01T15:00:00-05:00");
        assert!(!result.is_empty());
    }

    // Build status line
    #[test]
    fn test_basic_output_contains_context_and_model() {
        let c = crate::theme::get_colors(true);
        let output = build_status_line(42, "Opus", c, (None, false), None, None);
        assert!(output.contains("42%"));
        assert!(output.contains("Opus"));
        assert!(output.contains('◑'));
    }

    #[test]
    fn test_output_with_git_branch() {
        let c = crate::theme::get_colors(true);
        let output = build_status_line(42, "Opus", c, (Some("main"), false), None, None);
        assert!(output.contains("main"));
        assert!(output.contains('⎇'));
    }

    #[test]
    fn test_dirty_repo_shows_asterisk() {
        let c = crate::theme::get_colors(true);
        let output = build_status_line(42, "Opus", c, (Some("main"), true), None, None);
        assert!(output.contains('*'));
    }

    #[test]
    fn test_clean_repo_no_asterisk() {
        let c = crate::theme::get_colors(true);
        let output = build_status_line(42, "Opus", c, (Some("main"), false), None, None);
        // The only * would be from dirty indicator
        let stripped = strip_ansi(&output);
        assert!(!stripped.contains('*'));
    }

    #[test]
    fn test_no_branch_no_git_section() {
        let c = crate::theme::get_colors(true);
        let output = build_status_line(42, "Opus", c, (None, false), None, None);
        assert!(!output.contains('⎇'));
    }

    #[test]
    fn test_with_update() {
        let c = crate::theme::get_colors(true);
        let output = build_status_line(42, "Opus", c, (None, false), None, Some("1.0.24"));
        assert!(output.contains("↑1.0.24"));
    }

    #[test]
    fn test_with_usage() {
        let c = crate::theme::get_colors(true);
        let usage = UsageData {
            five_hour: 40.0,
            five_hour_resets: "2025-01-01T07:00:00+00:00".to_string(),
            seven_day: 60.0,
            seven_day_resets: String::new(),
        };
        let output = build_status_line(42, "Opus", c, (None, false), Some(&usage), None);
        assert!(output.contains("⏱"));
        assert!(output.contains("40%"));
    }

    #[test]
    fn test_output_order() {
        let c = crate::theme::get_colors(true);
        let usage = UsageData {
            five_hour: 40.0,
            five_hour_resets: String::new(),
            seven_day: 0.0,
            seven_day_resets: String::new(),
        };
        let output = build_status_line(
            42,
            "Opus",
            c,
            (Some("main"), false),
            Some(&usage),
            Some("1.0.24"),
        );
        let stripped = strip_ansi(&output);

        let ctx_pos = stripped.find("42%").unwrap();
        let model_pos = stripped.find("Opus").unwrap();
        let git_pos = stripped.find("main").unwrap();
        let usage_pos = stripped.find("40%").unwrap();
        let update_pos = stripped.find("1.0.24").unwrap();

        assert!(ctx_pos < model_pos);
        assert!(model_pos < git_pos);
        assert!(git_pos < usage_pos);
        assert!(usage_pos < update_pos);
    }

    #[test]
    fn test_elements_separated_by_spaces() {
        let c = crate::theme::get_colors(true);
        let output = build_status_line(42, "Opus", c, (Some("main"), false), None, None);
        let stripped = strip_ansi(&output);
        // Parts are separated by spaces
        assert!(stripped.contains(" ✦ "));
        assert!(stripped.contains(" ⎇ "));
    }

    fn strip_ansi(s: &str) -> String {
        let mut result = String::new();
        let mut in_escape = false;
        for c in s.chars() {
            if c == '\x1b' {
                in_escape = true;
            } else if in_escape {
                if c == 'm' {
                    in_escape = false;
                }
            } else {
                result.push(c);
            }
        }
        result
    }
}
