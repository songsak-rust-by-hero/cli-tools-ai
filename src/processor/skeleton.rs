use regex::Regex;
use std::sync::OnceLock;

static RE_SIG: OnceLock<Regex> = OnceLock::new();

pub fn extract_signatures(content: &str) -> String {
    let re = RE_SIG.get_or_init(|| {
        Regex::new(r"(?m)^(pub\s+)?(fn|struct|enum|trait|impl|type)\s+[\w<>, ]+")
            .expect("Invalid signature regex")
    });

    let mut skeleton: Vec<String> = Vec::new();
    for cap in re.captures_iter(content) {
        let line = cap.get(0).map_or("", |m| m.as_str()).trim();
        let clean_line = line.split('{').next().unwrap_or(line).trim();
        if !clean_line.is_empty() {
            skeleton.push(format!("  {}...", clean_line));
        }
    }
    skeleton.join("\n")
}
