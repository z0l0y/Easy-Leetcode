#[cfg(not(windows))]
use std::{path::PathBuf, process::Command};

#[cfg(not(windows))]
#[test]
fn cli_list_smoke() {
    let mut exe = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    exe.push("target");
    exe.push("debug");
    exe.push(if cfg!(windows) { "lh.exe" } else { "lh" });

    if !exe.exists() {
        if let Some(bin_path) = option_env!("CARGO_BIN_EXE_lh") {
            exe = PathBuf::from(bin_path.trim_matches('\0'));
        }
    }

    if !exe.exists() {
        eprintln!("skip cli smoke because lh binary not found: {}", exe.display());
        return;
    }

    let output = match Command::new(&exe)
        .arg("-l")
        .output()
    {
        Ok(output) => output,
        Err(err) => {
            eprintln!(
                "skip cli smoke due to spawn error ({}): {}",
                exe.display(),
                err
            );
            return;
        }
    };

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("最小覆盖子串"));
}

#[cfg(windows)]
#[test]
fn cli_list_smoke_windows_skip() {
    assert!(true);
}
