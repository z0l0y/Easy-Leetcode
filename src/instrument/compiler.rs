use anyhow::Context;
use std::fs;
use std::io::Write;
use std::process::Command;
use std::time::Instant;

pub struct JavaResult {
    pub output: Vec<String>,
    #[allow(dead_code)]
    pub errors: Vec<String>,
    #[allow(dead_code)]
    pub exit_code: i32,
    #[allow(dead_code)]
    pub elapsed_ms: u64,
}

/// Compile and run a generated Java source string.
/// `class_name` is the name of the runner class with main().
pub fn compile_and_run(java_code: &str, class_name: &str) -> anyhow::Result<JavaResult> {
    // Create temp directory
    let temp_dir = std::env::temp_dir().join(format!("lc_trace_{}", rand_suffix()));
    fs::create_dir_all(&temp_dir).context("创建临时目录失败")?;

    let java_file = temp_dir.join(format!("{}.java", class_name));

    // Write Java file
    let mut f = fs::File::create(&java_file).context("写入 Java 文件失败")?;
    f.write_all(java_code.as_bytes()).context("写入 Java 文件失败")?;
    f.flush().context("刷新文件失败")?;

    // Compile with javac
    let javac_start = Instant::now();
    let javac_output = Command::new("javac")
        .arg("-encoding")
        .arg("UTF-8")
        .arg(java_file.to_str().unwrap())
        .current_dir(&temp_dir)
        .output()
        .context("未找到 javac，请确保 JDK 8+ 已安装")?;

    if !javac_output.status.success() {
        let stderr = String::from_utf8_lossy(&javac_output.stderr).to_string();
        let java_content = fs::read_to_string(&java_file).unwrap_or_default();
        let _ = fs::remove_dir_all(&temp_dir);
        anyhow::bail!(
            "编译失败 (文件: {}):\n--- Java 源码前 20 行 ---\n{}\n--- javac 错误 ---\n{}",
            java_file.display(),
            java_content.lines().take(20).collect::<Vec<_>>().join("\n"),
            stderr
        );
    }

    // Run with java
    let java_start = Instant::now();
    let java_output = Command::new("java")
        .arg("-cp")
        .arg(temp_dir.to_str().unwrap())
        .arg(class_name)
        .current_dir(&temp_dir)
        .output()
        .context("运行 Java 失败")?;

    let elapsed = java_start.duration_since(javac_start).as_millis() as u64;

    let stdout = String::from_utf8_lossy(&java_output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&java_output.stderr).to_string();

    let output_lines: Vec<String> = stdout.lines().map(|s| s.to_string()).collect();
    let error_lines: Vec<String> = stderr.lines().map(|s| s.to_string()).collect();

    let exit_code = java_output.status.code().unwrap_or(-1);

    // Clean up temp dir
    let _ = fs::remove_dir_all(&temp_dir);

    if exit_code != 0 {
        anyhow::bail!(
            "运行出错 (exit={}):\n{}",
            exit_code,
            error_lines.join("\n")
        );
    }

    Ok(JavaResult {
        output: output_lines,
        errors: error_lines,
        exit_code,
        elapsed_ms: elapsed,
    })
}

fn rand_suffix() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    format!("{:08x}", nanos)
}
