//! CLI tests for keygen --store behavior.

use std::process::Command;

use assert_cmd::prelude::*;
use async_fs_io::TempDir;
use predicates::prelude::*;

mod common;
use common::{
    read,
    write,
};

#[tokio::test]
async fn keygen_store_os_rejects_output() {
    let temp_dir = TempDir::create(std::env::temp_dir()).await.unwrap();
    let output = temp_dir.path().join("custom.key");

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("dotenvage"));
    cmd.current_dir(temp_dir.path())
        .arg("keygen")
        .arg("--store")
        .arg("os")
        .arg("--output")
        .arg(&output)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "--output is only valid when --store is 'file' or 'both'",
        ));
}

#[tokio::test]
async fn keygen_store_file_writes_to_output() {
    let temp_dir = TempDir::create(std::env::temp_dir()).await.unwrap();
    let output = temp_dir.path().join("custom.key");

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("dotenvage"));
    cmd.current_dir(temp_dir.path())
        .arg("keygen")
        .arg("--store")
        .arg("file")
        .arg("--output")
        .arg(&output)
        .assert()
        .success();

    let content = read(&output).await.unwrap();
    assert!(
        content.contains("AGE-SECRET-KEY-"),
        "generated key file should contain an AGE identity"
    );
}

#[tokio::test]
async fn keygen_store_file_force_required_for_existing_output() {
    let temp_dir = TempDir::create(std::env::temp_dir()).await.unwrap();
    let output = temp_dir.path().join("existing.key");
    write(&output, "existing").await.unwrap();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("dotenvage"));
    cmd.current_dir(temp_dir.path())
        .arg("keygen")
        .arg("--store")
        .arg("file")
        .arg("--output")
        .arg(&output)
        .assert()
        .failure()
        .stderr(predicate::str::contains("key already exists"));

    let mut force_cmd = Command::new(assert_cmd::cargo::cargo_bin!("dotenvage"));
    force_cmd
        .current_dir(temp_dir.path())
        .arg("keygen")
        .arg("--store")
        .arg("file")
        .arg("--force")
        .arg("--output")
        .arg(&output)
        .assert()
        .success();

    let content = read(&output).await.unwrap();
    assert!(
        content.contains("AGE-SECRET-KEY-"),
        "forced overwrite should write a valid AGE identity"
    );
}
