use assert_cmd::Command;

fn run_statusline(input: &str) -> assert_cmd::assert::Assert {
    Command::cargo_bin("claude-statusline")
        .unwrap()
        .env("CLAUDE_STATUSLINE_THEME", "dark")
        .write_stdin(input)
        .assert()
}

fn sample_input(pct: u32, model: &str) -> String {
    format!(
        r#"{{"model":{{"display_name":"{}"}},"context_window":{{"used_percentage":{}}},"workspace":{{"current_dir":"/tmp"}},"version":"1.0.23"}}"#,
        model, pct
    )
}

#[test]
fn test_basic_output_format() {
    let output = run_statusline(&sample_input(42, "Opus"))
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(output).unwrap();
    assert!(text.contains("42%"));
    assert!(text.contains("Opus"));
    assert!(text.contains("✦"));
}

#[test]
fn test_invalid_json_input() {
    let output = run_statusline("not json")
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(output).unwrap();
    assert!(text.contains("◐"));
    assert!(text.contains("--%"));
}

#[test]
fn test_empty_input() {
    let output = run_statusline("").success().get_output().stdout.clone();
    let text = String::from_utf8(output).unwrap();
    assert!(text.contains("◐"));
}

#[test]
fn test_single_line_output() {
    let output = run_statusline(&sample_input(42, "Opus"))
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(output).unwrap();
    let lines: Vec<&str> = text.trim().lines().collect();
    assert_eq!(lines.len(), 1, "Expected single line, got: {:?}", lines);
}

#[test]
fn test_context_icons() {
    // Low: ◔
    let output = run_statusline(&sample_input(10, "Opus"))
        .success()
        .get_output()
        .stdout
        .clone();
    assert!(String::from_utf8(output).unwrap().contains('◔'));

    // Medium-low: ◑
    let output = run_statusline(&sample_input(30, "Opus"))
        .success()
        .get_output()
        .stdout
        .clone();
    assert!(String::from_utf8(output).unwrap().contains('◑'));

    // Medium-high: ◕
    let output = run_statusline(&sample_input(60, "Opus"))
        .success()
        .get_output()
        .stdout
        .clone();
    assert!(String::from_utf8(output).unwrap().contains('◕'));

    // High: ●
    let output = run_statusline(&sample_input(85, "Opus"))
        .success()
        .get_output()
        .stdout
        .clone();
    assert!(String::from_utf8(output).unwrap().contains('●'));
}

#[test]
fn test_different_models() {
    for model in ["Opus", "Sonnet", "Haiku"] {
        let output = run_statusline(&sample_input(42, model))
            .success()
            .get_output()
            .stdout
            .clone();
        assert!(String::from_utf8(output).unwrap().contains(model));
    }
}

#[test]
fn test_empty_json_object() {
    let output = run_statusline("{}").success().get_output().stdout.clone();
    let text = String::from_utf8(output).unwrap();
    assert!(text.contains("0%"));
    assert!(text.contains("?"));
}

#[test]
fn test_null_values() {
    let output =
        run_statusline(r#"{"model":null,"context_window":null,"workspace":null,"version":null}"#)
            .success()
            .get_output()
            .stdout
            .clone();
    let text = String::from_utf8(output).unwrap();
    assert!(text.contains("0%"));
    assert!(text.contains("?"));
}

#[test]
fn test_no_stderr_on_success() {
    run_statusline(&sample_input(42, "Opus"))
        .success()
        .stderr("");
}

#[test]
fn test_print_cache_dir() {
    let output = Command::cargo_bin("claude-statusline")
        .unwrap()
        .arg("--print-cache-dir")
        .output()
        .unwrap();
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.trim().contains("claude-statusline"));
}

#[test]
fn test_with_all_fields() {
    let input = r#"{"model":{"display_name":"Opus"},"context_window":{"used_percentage":42.5},"workspace":{"current_dir":"/tmp"},"version":"2.1.29"}"#;
    let output = run_statusline(input).success().get_output().stdout.clone();
    let text = String::from_utf8(output).unwrap();
    assert!(text.contains("42%"));
    assert!(text.contains("Opus"));
}
