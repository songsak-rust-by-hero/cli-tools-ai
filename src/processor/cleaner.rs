use regex::Regex;
use std::sync::OnceLock;

static RE_MULTI: OnceLock<Regex> = OnceLock::new();
static RE_SINGLE: OnceLock<Regex> = OnceLock::new();

pub fn trim_fat(content: &str) -> String {
    let re_multi = RE_MULTI.get_or_init(|| {
        Regex::new(r"(?s)/\*.*?\*/").expect("Invalid multi-line comment regex")
    });
    let re_single = RE_SINGLE.get_or_init(|| {
        Regex::new(r"(?m)//[!/]?.*$").expect("Invalid single-line comment regex")
    });

    let no_multi = re_multi.replace_all(content, "");
    let no_comments = re_single.replace_all(&no_multi, "");

    no_comments
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}
