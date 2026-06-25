//! v0.4.16: CLI early-exit regression tests.
//!
//! `relay-node --version` / `--help` must print and exit immediately, BEFORE
//! the rustls CryptoProvider is installed or the tokio runtime / network is
//! touched. The fix installs the provider only on the no-early-exit path, so
//! these flags must keep working without any TLS/network side effects.
//!
//! These run the compiled binary as a subprocess via the
//! `CARGO_BIN_EXE_relay-node` env var (set by cargo for integration tests).

use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_relay-node");

#[test]
fn version_flag_prints_and_exits_zero() {
    let output = Command::new(BIN)
        .arg("--version")
        .output()
        .expect("failed to spawn relay-node --version");
    assert!(
        output.status.success(),
        "non-zero exit: {:?}",
        output.status
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("relay-node "),
        "expected version banner, got: {stdout:?}"
    );
}

#[test]
fn short_version_flag_works() {
    let output = Command::new(BIN)
        .arg("-V")
        .output()
        .expect("failed to spawn relay-node -V");
    assert!(
        output.status.success(),
        "non-zero exit: {:?}",
        output.status
    );
}

#[test]
fn help_flag_prints_and_exits_zero() {
    let output = Command::new(BIN)
        .arg("--help")
        .output()
        .expect("failed to spawn relay-node --help");
    assert!(
        output.status.success(),
        "non-zero exit: {:?}",
        output.status
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Usage:"),
        "expected help text, got: {stdout:?}"
    );
}

/// Early-exit must be FAST: it neither builds a runtime nor installs the TLS
/// provider, so it should return well under a second. This guards against a
/// refactor that accidentally moves provider install before the flag parse.
#[test]
fn version_flag_is_fast_and_side_effect_free() {
    let start = std::time::Instant::now();
    let output = Command::new(BIN)
        .arg("--version")
        .output()
        .expect("failed to spawn relay-node --version");
    let elapsed = start.elapsed();
    assert!(
        output.status.success(),
        "non-zero exit: {:?}",
        output.status
    );
    assert!(
        elapsed < std::time::Duration::from_secs(2),
        "--version took {:?} — it must not build a runtime or install a TLS provider",
        elapsed
    );
    // No panic trace should reach stderr.
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("CryptoProvider") && !stderr.contains("panic"),
        "unexpected panic/provider noise on stderr for --version: {stderr:?}"
    );
}
