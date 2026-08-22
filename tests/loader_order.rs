//! Integration tests for loader evaluation order.

use std::env;

use serial_test::serial;

mod common;
use common::write;

fn names(paths: Vec<std::path::PathBuf>) -> Vec<String> {
    paths
        .into_iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
        .collect()
}

fn clear_env(keys: &[&str]) {
    for k in keys {
        unsafe { env::remove_var(k) };
    }
}

/// Standard list of environment variables to clear before each test
const CLEAR_ALL_DIMENSION_VARS: &[&str] = &[
    "DOTENVAGE_ENV",
    "EKG_ENV",
    "VERCEL_ENV",
    "NODE_ENV",
    "DOTENVAGE_OS",
    "EKG_OS",
    "DOTENVAGE_ARCH",
    "EKG_ARCH",
    "CARGO_CFG_TARGET_ARCH",
    "TARGET",
    "TARGETARCH",
    "TARGETPLATFORM",
    "RUNNER_ARCH",
    "DOTENVAGE_USER",
    "EKG_USER",
    "GITHUB_ACTOR",
    "GITHUB_TRIGGERING_ACTOR",
    "GITHUB_REPOSITORY_OWNER",
    "USER",
    "USERNAME",
    "DOTENVAGE_VARIANT",
    "EKG_VARIANT",
    "VARIANT",
    "GITHUB_EVENT_NAME",
    "GITHUB_REF",
    "PR_NUMBER",
];

#[tokio::test]
#[serial]
async fn order_no_env_uses_local() {
    clear_env(&[
        "DOTENVAGE_ENV",
        "EKG_ENV",
        "DOTENVAGE_ARCH",
        "EKG_ARCH",
        "DOTENVAGE_USER",
        "EKG_USER",
        "GITHUB_EVENT_NAME",
        "GITHUB_REF",
        "PR_NUMBER",
        "USER",
        "USERNAME",
    ]);
    let tmp = async_fs_io::TempDir::create(std::env::temp_dir())
        .await
        .unwrap();
    write(tmp.path().join(".env"), "").await.unwrap();
    write(tmp.path().join(".env.local"), "").await.unwrap();
    let loader = dotenvage::EnvLoader::with_manager(dotenvage::SecretManager::generate().unwrap());
    let got = loader.resolve_env_paths(tmp.path()).await.unwrap();
    let got = names(got);
    assert_eq!(got, vec![".env", ".env.local"]);
}

#[tokio::test]
#[serial]
async fn order_env_prod() {
    clear_env(&[
        "DOTENVAGE_ENV",
        "EKG_ENV",
        "DOTENVAGE_ARCH",
        "EKG_ARCH",
        "DOTENVAGE_USER",
        "EKG_USER",
        "GITHUB_EVENT_NAME",
        "GITHUB_REF",
        "PR_NUMBER",
        "USER",
        "USERNAME",
    ]);
    unsafe {
        env::set_var("DOTENVAGE_ENV", "prod");
    }
    let tmp = async_fs_io::TempDir::create(std::env::temp_dir())
        .await
        .unwrap();
    write(tmp.path().join(".env"), "").await.unwrap();
    write(tmp.path().join(".env.prod"), "").await.unwrap();
    let loader = dotenvage::EnvLoader::with_manager(dotenvage::SecretManager::generate().unwrap());
    let got = names(loader.resolve_env_paths(tmp.path()).await.unwrap());
    assert_eq!(got, vec![".env", ".env.prod"]);
}

#[tokio::test]
#[serial]
async fn order_env_arch() {
    clear_env(&[
        "DOTENVAGE_ENV",
        "EKG_ENV",
        "DOTENVAGE_ARCH",
        "EKG_ARCH",
        "DOTENVAGE_USER",
        "EKG_USER",
        "GITHUB_EVENT_NAME",
        "GITHUB_REF",
        "PR_NUMBER",
        "USER",
        "USERNAME",
    ]);
    unsafe {
        env::set_var("EKG_ENV", "prod");
    }
    unsafe {
        env::set_var("DOTENVAGE_ARCH", "docker-s3");
    }
    let tmp = async_fs_io::TempDir::create(std::env::temp_dir())
        .await
        .unwrap();
    write(tmp.path().join(".env"), "").await.unwrap();
    write(tmp.path().join(".env.prod"), "").await.unwrap();
    write(tmp.path().join(".env.prod.docker-s3"), "")
        .await
        .unwrap();
    let loader = dotenvage::EnvLoader::with_manager(dotenvage::SecretManager::generate().unwrap());
    let got = names(loader.resolve_env_paths(tmp.path()).await.unwrap());
    assert_eq!(got, vec![".env", ".env.prod", ".env.prod.docker-s3"]);
}

#[tokio::test]
#[serial]
async fn order_user_and_arch() {
    clear_env(&[
        "DOTENVAGE_ENV",
        "EKG_ENV",
        "DOTENVAGE_ARCH",
        "EKG_ARCH",
        "DOTENVAGE_USER",
        "EKG_USER",
        "GITHUB_EVENT_NAME",
        "GITHUB_REF",
        "PR_NUMBER",
        "USER",
        "USERNAME",
    ]);
    unsafe {
        env::set_var("DOTENVAGE_ENV", "prod");
    }
    unsafe {
        env::set_var("DOTENVAGE_ARCH", "docker-s3");
    }
    unsafe {
        env::set_var("DOTENVAGE_USER", "alice");
    }
    let tmp = async_fs_io::TempDir::create(std::env::temp_dir())
        .await
        .unwrap();
    write(tmp.path().join(".env"), "").await.unwrap();
    write(tmp.path().join(".env.prod"), "").await.unwrap();
    write(tmp.path().join(".env.prod.docker-s3"), "")
        .await
        .unwrap();
    write(tmp.path().join(".env.alice"), "").await.unwrap();
    write(tmp.path().join(".env.prod.alice"), "").await.unwrap();
    write(tmp.path().join(".env.prod.docker-s3.alice"), "")
        .await
        .unwrap();
    let loader = dotenvage::EnvLoader::with_manager(dotenvage::SecretManager::generate().unwrap());
    let got = names(loader.resolve_env_paths(tmp.path()).await.unwrap());
    assert_eq!(
        got,
        vec![
            ".env",
            ".env.prod",
            ".env.prod.docker-s3",
            ".env.alice",
            ".env.prod.alice",
            ".env.prod.docker-s3.alice"
        ]
    );
}

#[tokio::test]
#[serial]
async fn order_user_without_env() {
    clear_env(&[
        "DOTENVAGE_ENV",
        "EKG_ENV",
        "DOTENVAGE_ARCH",
        "EKG_ARCH",
        "DOTENVAGE_USER",
        "EKG_USER",
        "GITHUB_EVENT_NAME",
        "GITHUB_REF",
        "PR_NUMBER",
        "USER",
        "USERNAME",
    ]);
    unsafe {
        env::set_var("DOTENVAGE_USER", "bob");
    }
    let tmp = async_fs_io::TempDir::create(std::env::temp_dir())
        .await
        .unwrap();
    write(tmp.path().join(".env"), "").await.unwrap();
    write(tmp.path().join(".env.local"), "").await.unwrap();
    write(tmp.path().join(".env.bob"), "").await.unwrap();
    write(tmp.path().join(".env.local.bob"), "").await.unwrap();
    let loader = dotenvage::EnvLoader::with_manager(dotenvage::SecretManager::generate().unwrap());
    let got = names(loader.resolve_env_paths(tmp.path()).await.unwrap());
    assert_eq!(
        got,
        vec![".env", ".env.local", ".env.bob", ".env.local.bob"]
    );
}

#[tokio::test]
#[serial]
async fn order_cargo_target_arch() {
    clear_env(&[
        "DOTENVAGE_ENV",
        "EKG_ENV",
        "DOTENVAGE_ARCH",
        "EKG_ARCH",
        "CARGO_CFG_TARGET_ARCH",
        "TARGET",
        "DOTENVAGE_USER",
        "EKG_USER",
        "GITHUB_EVENT_NAME",
        "GITHUB_REF",
        "PR_NUMBER",
        "USER",
        "USERNAME",
    ]);
    unsafe {
        env::set_var("CARGO_CFG_TARGET_ARCH", "aarch64");
    }
    let tmp = async_fs_io::TempDir::create(std::env::temp_dir())
        .await
        .unwrap();
    write(tmp.path().join(".env"), "").await.unwrap();
    write(tmp.path().join(".env.local"), "").await.unwrap();
    write(tmp.path().join(".env.local.arm64"), "")
        .await
        .unwrap();
    let loader = dotenvage::EnvLoader::with_manager(dotenvage::SecretManager::generate().unwrap());
    let got = names(loader.resolve_env_paths(tmp.path()).await.unwrap());
    assert_eq!(got, vec![".env", ".env.local", ".env.local.arm64"]);
}

#[tokio::test]
#[serial]
async fn order_target_triple() {
    clear_env(&[
        "DOTENVAGE_ENV",
        "EKG_ENV",
        "DOTENVAGE_ARCH",
        "EKG_ARCH",
        "CARGO_CFG_TARGET_ARCH",
        "TARGET",
        "DOTENVAGE_USER",
        "EKG_USER",
        "GITHUB_EVENT_NAME",
        "GITHUB_REF",
        "PR_NUMBER",
        "USER",
        "USERNAME",
    ]);
    unsafe {
        env::set_var("TARGET", "x86_64-unknown-linux-gnu");
    }
    let tmp = async_fs_io::TempDir::create(std::env::temp_dir())
        .await
        .unwrap();
    write(tmp.path().join(".env"), "").await.unwrap();
    write(tmp.path().join(".env.local"), "").await.unwrap();
    write(tmp.path().join(".env.local.amd64"), "")
        .await
        .unwrap();
    let loader = dotenvage::EnvLoader::with_manager(dotenvage::SecretManager::generate().unwrap());
    let got = names(loader.resolve_env_paths(tmp.path()).await.unwrap());
    assert_eq!(got, vec![".env", ".env.local", ".env.local.amd64"]);
}

#[tokio::test]
#[serial]
async fn order_github_pr_number() {
    clear_env(&[
        "DOTENVAGE_ENV",
        "EKG_ENV",
        "DOTENVAGE_ARCH",
        "EKG_ARCH",
        "DOTENVAGE_USER",
        "EKG_USER",
        "GITHUB_EVENT_NAME",
        "GITHUB_REF",
        "PR_NUMBER",
        "USER",
        "USERNAME",
    ]);
    unsafe {
        env::set_var("GITHUB_EVENT_NAME", "pull_request");
    }
    unsafe {
        env::set_var("PR_NUMBER", "123");
    }
    let tmp = async_fs_io::TempDir::create(std::env::temp_dir())
        .await
        .unwrap();
    write(tmp.path().join(".env"), "").await.unwrap();
    write(tmp.path().join(".env.pr-123"), "").await.unwrap();
    let loader = dotenvage::EnvLoader::with_manager(dotenvage::SecretManager::generate().unwrap());
    let got = names(loader.resolve_env_paths(tmp.path()).await.unwrap());
    assert_eq!(got, vec![".env", ".env.pr-123"]);
}

#[tokio::test]
#[serial]
async fn order_github_ref_parsing() {
    clear_env(&[
        "DOTENVAGE_ENV",
        "EKG_ENV",
        "DOTENVAGE_ARCH",
        "EKG_ARCH",
        "DOTENVAGE_USER",
        "EKG_USER",
        "GITHUB_EVENT_NAME",
        "GITHUB_REF",
        "PR_NUMBER",
        "USER",
        "USERNAME",
    ]);
    unsafe {
        env::set_var("GITHUB_REF", "refs/pull/456/merge");
    }
    let tmp = async_fs_io::TempDir::create(std::env::temp_dir())
        .await
        .unwrap();
    write(tmp.path().join(".env"), "").await.unwrap();
    write(tmp.path().join(".env.pr-456"), "").await.unwrap();
    let loader = dotenvage::EnvLoader::with_manager(dotenvage::SecretManager::generate().unwrap());
    let got = names(loader.resolve_env_paths(tmp.path()).await.unwrap());
    assert_eq!(got, vec![".env", ".env.pr-456"]);
}

#[tokio::test]
#[serial]
async fn case_insensitive_and_separators() {
    clear_env(&[
        "DOTENVAGE_ENV",
        "EKG_ENV",
        "DOTENVAGE_ARCH",
        "EKG_ARCH",
        "DOTENVAGE_USER",
        "EKG_USER",
        "GITHUB_EVENT_NAME",
        "GITHUB_REF",
        "PR_NUMBER",
        "USER",
        "USERNAME",
    ]);
    unsafe {
        env::set_var("DOTENVAGE_ENV", "PrOd");
    }
    unsafe {
        env::set_var("DOTENVAGE_ARCH", "DoCkEr-S3");
    }
    unsafe {
        env::set_var("DOTENVAGE_USER", "AlIcE");
    }
    let tmp = async_fs_io::TempDir::create(std::env::temp_dir())
        .await
        .unwrap();
    write(tmp.path().join(".ENV"), "").await.unwrap();
    write(tmp.path().join(".ENV.PROD"), "").await.unwrap();
    write(tmp.path().join(".ENV.PROD.DOCKER-S3.ALICE"), "")
        .await
        .unwrap();
    let loader = dotenvage::EnvLoader::with_manager(dotenvage::SecretManager::generate().unwrap());
    let got = names(loader.resolve_env_paths(tmp.path()).await.unwrap());
    assert_eq!(got, vec![".ENV", ".ENV.PROD", ".ENV.PROD.DOCKER-S3.ALICE"]);
}

#[tokio::test]
#[serial]
async fn order_variant_basic() {
    clear_env(CLEAR_ALL_DIMENSION_VARS);
    unsafe {
        env::set_var("DOTENVAGE_ENV", "prod");
    }
    unsafe {
        env::set_var("DOTENVAGE_VARIANT", "docker");
    }
    let tmp = async_fs_io::TempDir::create(std::env::temp_dir())
        .await
        .unwrap();
    write(tmp.path().join(".env"), "").await.unwrap();
    write(tmp.path().join(".env.prod"), "").await.unwrap();
    write(tmp.path().join(".env.docker"), "").await.unwrap();
    write(tmp.path().join(".env.prod.docker"), "")
        .await
        .unwrap();
    let loader = dotenvage::EnvLoader::with_manager(dotenvage::SecretManager::generate().unwrap());
    let got = names(loader.resolve_env_paths(tmp.path()).await.unwrap());
    assert_eq!(
        got,
        vec![".env", ".env.prod", ".env.docker", ".env.prod.docker"]
    );
}

#[tokio::test]
#[serial]
async fn order_all_five_dimensions() {
    clear_env(CLEAR_ALL_DIMENSION_VARS);
    unsafe {
        env::set_var("DOTENVAGE_ENV", "prod");
    }
    unsafe {
        env::set_var("DOTENVAGE_ARCH", "arm64");
    }
    unsafe {
        env::set_var("DOTENVAGE_USER", "alice");
    }
    unsafe {
        env::set_var("DOTENVAGE_VARIANT", "k8s");
    }
    let tmp = async_fs_io::TempDir::create(std::env::temp_dir())
        .await
        .unwrap();
    // Create files for a subset of combinations
    write(tmp.path().join(".env"), "").await.unwrap();
    write(tmp.path().join(".env.prod"), "").await.unwrap();
    write(tmp.path().join(".env.arm64"), "").await.unwrap();
    write(tmp.path().join(".env.alice"), "").await.unwrap();
    write(tmp.path().join(".env.k8s"), "").await.unwrap();
    write(tmp.path().join(".env.prod.arm64"), "").await.unwrap();
    write(tmp.path().join(".env.prod.alice"), "").await.unwrap();
    write(tmp.path().join(".env.prod.k8s"), "").await.unwrap();
    write(tmp.path().join(".env.arm64.alice"), "")
        .await
        .unwrap();
    write(tmp.path().join(".env.alice.k8s"), "").await.unwrap();
    write(tmp.path().join(".env.prod.arm64.alice.k8s"), "")
        .await
        .unwrap();
    let loader = dotenvage::EnvLoader::with_manager(dotenvage::SecretManager::generate().unwrap());
    let got = names(loader.resolve_env_paths(tmp.path()).await.unwrap());
    // Verify specificity ordering based on bitmask iteration
    // Bitmask: bit0=ENV, bit1=OS, bit2=ARCH, bit3=USER, bit4=VARIANT
    // Files are ordered by mask value (which naturally groups by part count)
    assert_eq!(
        got,
        vec![
            ".env",
            ".env.prod",                 // mask 1
            ".env.arm64",                // mask 4
            ".env.prod.arm64",           // mask 5
            ".env.alice",                // mask 8
            ".env.prod.alice",           // mask 9
            ".env.arm64.alice",          // mask 12
            ".env.k8s",                  // mask 16
            ".env.prod.k8s",             // mask 17
            ".env.alice.k8s",            // mask 24
            ".env.prod.arm64.alice.k8s"  // mask 29
        ]
    );
}

#[tokio::test]
#[serial]
async fn order_variant_with_ekg_prefix() {
    clear_env(CLEAR_ALL_DIMENSION_VARS);
    unsafe {
        env::set_var("EKG_VARIANT", "canary");
    }
    let tmp = async_fs_io::TempDir::create(std::env::temp_dir())
        .await
        .unwrap();
    write(tmp.path().join(".env"), "").await.unwrap();
    write(tmp.path().join(".env.local"), "").await.unwrap();
    write(tmp.path().join(".env.canary"), "").await.unwrap();
    write(tmp.path().join(".env.local.canary"), "")
        .await
        .unwrap();
    let loader = dotenvage::EnvLoader::with_manager(dotenvage::SecretManager::generate().unwrap());
    let got = names(loader.resolve_env_paths(tmp.path()).await.unwrap());
    assert_eq!(
        got,
        vec![".env", ".env.local", ".env.canary", ".env.local.canary"]
    );
}
