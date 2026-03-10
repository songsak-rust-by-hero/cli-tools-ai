use regex::Regex;

pub fn trim_fat(content: &str) -> String {
    // 1. ลบ Multi-line comments /* ... */
    let re_multi = Regex::new(r"(?s)/\*.*?\*/").unwrap();
    let no_multi = re_multi.replace_all(content, "");

    // 2. ลบ Doc comments (///) และ Single-line (//)
    let re_single = Regex::new(r"(?m)//[!/]?.*$").unwrap();
    let no_comments = re_single.replace_all(&no_multi, "");

    // 3. ยุบบรรทัดว่างที่ซ้อนกัน และ Trim หัวท้าย
    no_comments
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}
