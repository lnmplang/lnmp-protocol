#![allow(deprecated)]
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_benchmark_codec_small() {
    let mut cmd = Command::cargo_bin("lnmp-cli").unwrap();
    cmd.arg("perf")
        .arg("benchmark")
        .arg("codec")
        .arg("--preset")
        .arg("small")
        .assert()
        .success()
        .stdout(predicate::str::contains("LNMP Codec Benchmark"))
        .stdout(predicate::str::contains("Parse Performance"));
}

#[test]
fn test_benchmark_embedding_small() {
    let mut cmd = Command::cargo_bin("lnmp-cli").unwrap();
    cmd.arg("perf")
        .arg("benchmark")
        .arg("embedding")
        .arg("--preset")
        .arg("small")
        .assert()
        .success()
        .stdout(predicate::str::contains("LNMP Embedding Benchmark"))
        .stdout(predicate::str::contains("Quantization"));
}

#[test]
fn test_benchmark_with_timing() {
    let mut cmd = Command::cargo_bin("lnmp-cli").unwrap();
    cmd.arg("perf")
        .arg("benchmark")
        .arg("codec")
        .arg("--preset")
        .arg("small")
        .arg("--timing")
        .assert()
        .success()
        .stdout(predicate::str::contains("Timing Breakdown"));
}

#[test]
fn test_benchmark_with_progress() {
    // Progress bar output goes to stderr usually, but might be hard to capture in test env
    // Just checking it doesn't crash
    let mut cmd = Command::cargo_bin("lnmp-cli").unwrap();
    cmd.arg("perf")
        .arg("benchmark")
        .arg("codec")
        .arg("--preset")
        .arg("small")
        .arg("--progress")
        .assert()
        .success();
}

#[test]
fn test_invalid_preset() {
    let mut cmd = Command::cargo_bin("lnmp-cli").unwrap();
    cmd.arg("perf")
        .arg("benchmark")
        .arg("codec")
        .arg("--preset")
        .arg("invalid_preset")
        .assert()
        .success() // Should still succeed but use default or show error message in output
        .stderr(predicate::str::contains("Unknown preset"));
}
