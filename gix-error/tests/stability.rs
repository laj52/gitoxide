use std::{fs, process::Command};

#[test]
fn downstream_api_requires_unstable_only_for_experimental_items() -> gix_error::TestResult {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixture = gix_testtools::tempfile::tempdir()?;
    fs::write(
        fixture.path().join("Cargo.toml"),
        format!(
            "\
[workspace]
[package]
name = \"error-stability-fixture\"
version = \"0.0.0\"
edition = \"2024\"
[dependencies]
gix-error = {{ path = {root:?} }}
gix-lock = {{ path = {lock:?}, optional = true }}
",
            lock = root.join("../gix-lock"),
        ),
    )?;
    fs::create_dir_all(fixture.path().join("src/bin"))?;
    fs::write(
        fixture.path().join("src/bin/stable.rs"),
        include_str!("fixtures/stability/stable.rs"),
    )?;
    fs::write(
        fixture.path().join("src/bin/experimental.rs"),
        include_str!("fixtures/stability/experimental.rs"),
    )?;
    let run = |args: &[&str]| -> std::io::Result<std::process::Output> {
        let mut command = Command::new(env!("CARGO"));
        gix_testtools::configure_git_environment(&mut command, fixture.path());
        command
            .current_dir(fixture.path())
            .args(args)
            .args(["--offline", "--manifest-path"])
            .arg(fixture.path().join("Cargo.toml"))
            .arg("--target-dir")
            .arg(root.join("../target/gix-error-stability"))
            .output()
    };
    for features in [None, Some("gix-error/error-print-location"), Some("gix-lock")] {
        let mut args = vec!["run", "--bin", "stable"];
        if let Some(features) = features {
            args.extend(["--features", features]);
        }
        let output = run(&args)?;
        assert!(
            output.status.success(),
            "the supported API works without test feature unification, including lock recovery: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let output = run(&["check", "--bin", "experimental"])?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "experimental APIs require explicit opt-in");
    for diagnostic in [
        "struct `Exn` is private",
        "type alias `ExnResult` is private",
        "module `exn` is private",
        "module `types` is private",
        "struct `Message` is private",
        "trait `ErrorExt` is private",
        "method `into_exn` is private",
        "method `metadata` is private",
    ] {
        assert!(
            stderr.contains(diagnostic),
            "missing visibility diagnostic {diagnostic}: {stderr}"
        );
    }
    let output = run(&["check", "--bin", "experimental", "--features", "gix-error/unstable"])?;
    assert!(
        output.status.success(),
        "experimental APIs and their macros compile with unstable: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}
