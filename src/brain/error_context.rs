use std::process::Command;

pub struct ErrorContext {
    pub has_errors: bool,
    pub output: String,
    pub affected_files: Vec<String>,
}

pub fn run_cargo_check(project_dir: &str) -> ErrorContext {
    let result = Command::new("cargo")
        .arg("check")
        .arg("--message-format=short")
        .current_dir(project_dir)
        .output();

    match result {
        Err(e) => ErrorContext {
            has_errors: false,
            output: format!("Could not run cargo check: {}", e),
            affected_files: Vec::new(),
        },
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let has_errors = !output.status.success();

            // ดึงชื่อไฟล์ที่ error ชี้มา
            let affected_files: Vec<String> = stderr
                .lines()
                .filter(|line| line.contains(".rs:"))
                .filter_map(|line| {
                    line.split(".rs:").next().map(|s| {
                        let trimmed = s.trim();
                        // เอาแค่ส่วนหลัง --> หรือ error[
                        trimmed
                            .rsplit_once(' ')
                            .map(|(_, f)| format!("{}.rs", f))
                            .unwrap_or_else(|| format!("{}.rs", trimmed))
                    })
                })
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();

            ErrorContext {
                has_errors,
                output: stderr,
                affected_files,
            }
        }
    }
}

pub fn format_for_context(ctx: &ErrorContext) -> String {
    if !ctx.has_errors {
        return "### CARGO CHECK ###\n✅ No errors found.\n".to_string();
    }

    let mut out = String::from("### CARGO CHECK ERRORS ###\n");
    out.push_str(&ctx.output);
    out.push('\n');

    if !ctx.affected_files.is_empty() {
        out.push_str("### AFFECTED FILES ###\n");
        for f in &ctx.affected_files {
            out.push_str(&format!("  - {}\n", f));
        }
    }

    out
}
