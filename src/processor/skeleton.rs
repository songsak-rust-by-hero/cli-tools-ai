use regex::Regex;

pub fn extract_signatures(content: &str) -> String {
    // Regex สำหรับจับ pub fn, struct, enum, impl
    let re = Regex::new(r"(?m)^(pub\s+)?(fn|struct|enum|trait|impl|type)\s+[\w<>, ]+").unwrap();

    let mut skeleton = Vec::new();
    for cap in re.captures_iter(content) {
        let line = cap.get(0).map_or("", |m| m.as_str()).trim();
        let clean_line = line.split('{').next().unwrap_or(line).trim();
        if !clean_line.is_empty() {
            skeleton.push(format!("  {}...", clean_line));
        }
    }
    skeleton.join("\n")
}
