use std::io::Read;
use std::path::Path;

fn main() {
    // Handle --print-cache-dir flag
    if std::env::args().nth(1).as_deref() == Some("--print-cache-dir") {
        println!("{}", claude_statusline::cache::get_cache_dir().display());
        return;
    }

    // Read JSON from stdin
    let mut input = String::new();
    if std::io::stdin().read_to_string(&mut input).is_err() {
        println!("◐ --% ✦");
        return;
    }

    // Debug: dump full JSON if CLAUDE_STATUSLINE_DEBUG is set
    if let Ok(debug_file) = std::env::var("CLAUDE_STATUSLINE_DEBUG") {
        let _ = std::fs::write(Path::new(&debug_file), &input);
    }

    let output = claude_statusline::run(&input);
    println!("{}", output);
}
