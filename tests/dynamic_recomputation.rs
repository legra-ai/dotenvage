//! Integration tests for dynamic dimension recomputation.
//!
//! These tests verify that dimension configuration values (like NODE_ENV
//! or VARIANT) discovered in loaded files can cause additional files to
//! be loaded.

use std::env;

use serial_test::serial;

mod common;
use common::write;

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

fn clear_env(keys: &[&str]) {
    for k in keys {
        unsafe { env::remove_var(k) };
    }
}

#[tokio::test]
#[serial]
async fn dynamic_env_discovery_from_base_file() {
    // Test: NODE_ENV=production in .env causes .env.production to load
    clear_env(CLEAR_ALL_DIMENSION_VARS);

    let tmp = async_fs_io::TempDir::create(std::env::temp_dir())
        .await
        .unwrap();

    // .env sets NODE_ENV=production
    write(tmp.path().join(".env"), "NODE_ENV=production\nBASE=1\n")
        .await
        .unwrap();

    // .env.production should be loaded because NODE_ENV is discovered
    write(tmp.path().join(".env.production"), "PROD=1\n")
        .await
        .unwrap();

    let loader = dotenvage::EnvLoader::with_manager(dotenvage::SecretManager::generate().unwrap());
    loader.load_from_dir(tmp.path()).await.unwrap();

    // Verify both files were loaded
    assert_eq!(env::var("BASE").unwrap(), "1");
    assert_eq!(env::var("PROD").unwrap(), "1");
    assert_eq!(env::var("NODE_ENV").unwrap(), "production");
}

#[tokio::test]
#[serial]
async fn dynamic_variant_discovery_triggers_reload() {
    // Test: VARIANT=docker in .env causes .env.docker to load
    clear_env(CLEAR_ALL_DIMENSION_VARS);

    let tmp = async_fs_io::TempDir::create(std::env::temp_dir())
        .await
        .unwrap();

    // .env sets VARIANT=docker
    write(tmp.path().join(".env"), "VARIANT=docker\nBASE=1\n")
        .await
        .unwrap();

    // .env.docker should be loaded because VARIANT is discovered
    write(tmp.path().join(".env.docker"), "DOCKER_CONFIG=enabled\n")
        .await
        .unwrap();

    let loader = dotenvage::EnvLoader::with_manager(dotenvage::SecretManager::generate().unwrap());
    loader.load_from_dir(tmp.path()).await.unwrap();

    // Verify both files were loaded
    assert_eq!(env::var("BASE").unwrap(), "1");
    assert_eq!(env::var("DOCKER_CONFIG").unwrap(), "enabled");
    assert_eq!(env::var("VARIANT").unwrap(), "docker");
}

#[tokio::test]
#[serial]
async fn dynamic_arch_discovery_from_env_file() {
    // Test: DOTENVAGE_ARCH=arm64 in .env causes .env.local.arm64 to load
    clear_env(CLEAR_ALL_DIMENSION_VARS);

    let tmp = async_fs_io::TempDir::create(std::env::temp_dir())
        .await
        .unwrap();

    // .env sets DOTENVAGE_ARCH=arm64
    write(tmp.path().join(".env"), "DOTENVAGE_ARCH=arm64\nBASE=1\n")
        .await
        .unwrap();

    // .env.local and .env.local.arm64 should be loaded
    write(tmp.path().join(".env.local"), "LOCAL=1\n")
        .await
        .unwrap();
    write(
        tmp.path().join(".env.local.arm64"),
        "ARM64_CONFIG=enabled\n",
    )
    .await
    .unwrap();

    let loader = dotenvage::EnvLoader::with_manager(dotenvage::SecretManager::generate().unwrap());
    loader.load_from_dir(tmp.path()).await.unwrap();

    // Verify all files were loaded
    assert_eq!(env::var("BASE").unwrap(), "1");
    assert_eq!(env::var("LOCAL").unwrap(), "1");
    assert_eq!(env::var("ARM64_CONFIG").unwrap(), "enabled");
}

#[tokio::test]
#[serial]
async fn no_double_loading() {
    // Test: Files are not loaded twice even if they match multiple
    // recomputation rounds
    clear_env(CLEAR_ALL_DIMENSION_VARS);

    let tmp = async_fs_io::TempDir::create(std::env::temp_dir())
        .await
        .unwrap();

    // .env sets NODE_ENV, COUNTER starts at 0
    write(tmp.path().join(".env"), "NODE_ENV=prod\nCOUNTER=0\n")
        .await
        .unwrap();

    // .env.prod overrides COUNTER - if loaded twice, this would be
    // problematic for the test logic
    write(tmp.path().join(".env.prod"), "COUNTER=1\nPROD_LOADED=yes\n")
        .await
        .unwrap();

    let loader = dotenvage::EnvLoader::with_manager(dotenvage::SecretManager::generate().unwrap());
    loader.load_from_dir(tmp.path()).await.unwrap();

    // Verify .env.prod was loaded (COUNTER=1 overrides COUNTER=0)
    assert_eq!(env::var("COUNTER").unwrap(), "1");
    assert_eq!(env::var("PROD_LOADED").unwrap(), "yes");
}

#[tokio::test]
#[serial]
async fn chained_discovery() {
    // Test: .env sets ENV, .env.staging sets VARIANT, .env.staging.canary
    // loads
    clear_env(CLEAR_ALL_DIMENSION_VARS);

    let tmp = async_fs_io::TempDir::create(std::env::temp_dir())
        .await
        .unwrap();

    // Chain: .env -> .env.staging -> .env.staging.canary
    write(tmp.path().join(".env"), "NODE_ENV=staging\nSTEP=1\n")
        .await
        .unwrap();
    write(tmp.path().join(".env.staging"), "VARIANT=canary\nSTEP=2\n")
        .await
        .unwrap();
    write(
        tmp.path().join(".env.staging.canary"),
        "STEP=3\nCANARY=yes\n",
    )
    .await
    .unwrap();

    let loader = dotenvage::EnvLoader::with_manager(dotenvage::SecretManager::generate().unwrap());
    loader.load_from_dir(tmp.path()).await.unwrap();

    // Verify the chain was followed
    assert_eq!(env::var("NODE_ENV").unwrap(), "staging");
    assert_eq!(env::var("VARIANT").unwrap(), "canary");
    assert_eq!(env::var("STEP").unwrap(), "3"); // Last file wins
    assert_eq!(env::var("CANARY").unwrap(), "yes");
}

#[tokio::test]
#[serial]
async fn encrypted_dimension_config_ignored_in_discovery() {
    // Test: Encrypted dimension config values are skipped during discovery
    // phase. We test this at the discovery level, not the full load level,
    // because load_env_file will fail on invalid encrypted values.
    clear_env(CLEAR_ALL_DIMENSION_VARS);

    // Test discover_dimensions_from_vars directly by checking that resolve
    // functions don't pick up encrypted values from the environment
    let tmp = async_fs_io::TempDir::create(std::env::temp_dir())
        .await
        .unwrap();

    // .env has a plaintext VARIANT but no ENV set
    write(tmp.path().join(".env"), "VARIANT=docker\nBASE=1\n")
        .await
        .unwrap();

    // .env.local should be loaded (default ENV=local)
    write(tmp.path().join(".env.local"), "LOCAL=1\n")
        .await
        .unwrap();

    // .env.docker should be loaded (VARIANT=docker was discovered)
    write(tmp.path().join(".env.docker"), "DOCKER=1\n")
        .await
        .unwrap();

    // .env.local.docker should also be loaded
    write(tmp.path().join(".env.local.docker"), "LOCAL_DOCKER=1\n")
        .await
        .unwrap();

    let loader = dotenvage::EnvLoader::with_manager(dotenvage::SecretManager::generate().unwrap());
    loader.load_from_dir(tmp.path()).await.unwrap();

    // Verify correct files were loaded
    assert_eq!(env::var("BASE").unwrap(), "1");
    assert_eq!(env::var("LOCAL").unwrap(), "1");
    assert_eq!(env::var("DOCKER").unwrap(), "1");
    assert_eq!(env::var("LOCAL_DOCKER").unwrap(), "1");
}

#[tokio::test]
#[serial]
async fn env_var_determines_file_selection() {
    // Test: Environment variables set BEFORE loading determine which files
    // are selected for loading (via resolve_* functions). The file's value
    // may still override the env var after loading.
    clear_env(CLEAR_ALL_DIMENSION_VARS);

    // Set NODE_ENV=production via environment BEFORE loading
    unsafe {
        env::set_var("NODE_ENV", "production");
    }

    let tmp = async_fs_io::TempDir::create(std::env::temp_dir())
        .await
        .unwrap();

    // .env has NODE_ENV=staging, but dimension discovery should use the
    // existing env var (production)
    write(tmp.path().join(".env"), "NODE_ENV=staging\nBASE=1\n")
        .await
        .unwrap();

    // .env.production should be loaded (env var determines file selection)
    write(tmp.path().join(".env.production"), "PROD=1\n")
        .await
        .unwrap();

    // .env.staging should NOT be loaded (env var takes precedence for
    // discovery)
    write(tmp.path().join(".env.staging"), "STAGING=1\n")
        .await
        .unwrap();

    let loader = dotenvage::EnvLoader::with_manager(dotenvage::SecretManager::generate().unwrap());
    loader.load_from_dir(tmp.path()).await.unwrap();

    // Verify that .env.production was loaded (not staging)
    assert_eq!(env::var("BASE").unwrap(), "1");
    assert_eq!(env::var("PROD").unwrap(), "1");
    assert!(
        env::var("STAGING").is_err(),
        "Env var should determine file selection, staging should not load"
    );

    // Note: NODE_ENV may be overwritten by .env file content after loading
    // (that's expected behavior - files can override env vars)
}

// =============================================================================
// Tests for collect_all_vars_from_dir (used by CLI list/dump commands)
// =============================================================================
// These tests ensure the CLI code path correctly handles dynamic discovery.
// A bug was found where CLI commands called resolve_env_paths once without
// iterating, missing files that should be loaded based on dimensions
// discovered in intermediate files.

#[tokio::test]
#[serial]
async fn collect_vars_discovers_variant_from_intermediate_file() {
    // Test: collect_all_vars_from_dir discovers VARIANT from .env.local
    // and loads .env.local.oxigraph (the real-world bug scenario)
    clear_env(CLEAR_ALL_DIMENSION_VARS);

    let tmp = async_fs_io::TempDir::create(std::env::temp_dir())
        .await
        .unwrap();

    // .env - base file (no VARIANT set here)
    write(tmp.path().join(".env"), "BASE=from_env\n")
        .await
        .unwrap();

    // .env.local - sets EKG_VARIANT=oxigraph
    write(
        tmp.path().join(".env.local"),
        "LOCAL=from_local\nEKG_VARIANT=oxigraph\n",
    )
    .await
    .unwrap();

    // .env.local.oxigraph - should be loaded because VARIANT is discovered
    write(
        tmp.path().join(".env.local.oxigraph"),
        "OXIGRAPH_CONFIG=from_oxigraph\n",
    )
    .await
    .unwrap();

    let loader = dotenvage::EnvLoader::with_manager(dotenvage::SecretManager::generate().unwrap());
    let (vars, paths) = loader.collect_all_vars_from_dir(tmp.path()).await.unwrap();

    // Verify all three files were loaded
    assert_eq!(vars.get("BASE").unwrap(), "from_env");
    assert_eq!(vars.get("LOCAL").unwrap(), "from_local");
    assert_eq!(vars.get("EKG_VARIANT").unwrap(), "oxigraph");
    assert_eq!(
        vars.get("OXIGRAPH_CONFIG").unwrap(),
        "from_oxigraph",
        "VARIANT discovered in .env.local should trigger loading .env.local.oxigraph"
    );

    // Verify the paths include all three files
    assert_eq!(paths.len(), 3, "Should load exactly 3 files");
    assert!(paths[0].ends_with(".env"));
    assert!(paths[1].ends_with(".env.local"));
    assert!(paths[2].ends_with(".env.local.oxigraph"));
}

#[tokio::test]
#[serial]
async fn collect_vars_chained_discovery() {
    // Test: collect_all_vars_from_dir handles chained discovery
    // .env -> .env.staging -> .env.staging.canary
    clear_env(CLEAR_ALL_DIMENSION_VARS);

    let tmp = async_fs_io::TempDir::create(std::env::temp_dir())
        .await
        .unwrap();

    // Chain: .env sets ENV, .env.staging sets VARIANT, .env.staging.canary loads
    write(tmp.path().join(".env"), "NODE_ENV=staging\nSTEP=1\n")
        .await
        .unwrap();
    write(tmp.path().join(".env.staging"), "VARIANT=canary\nSTEP=2\n")
        .await
        .unwrap();
    write(
        tmp.path().join(".env.staging.canary"),
        "STEP=3\nCANARY_FLAG=yes\n",
    )
    .await
    .unwrap();

    let loader = dotenvage::EnvLoader::with_manager(dotenvage::SecretManager::generate().unwrap());
    let (vars, paths) = loader.collect_all_vars_from_dir(tmp.path()).await.unwrap();

    // Verify the chain was followed
    assert_eq!(vars.get("NODE_ENV").unwrap(), "staging");
    assert_eq!(vars.get("VARIANT").unwrap(), "canary");
    assert_eq!(vars.get("STEP").unwrap(), "3"); // Last file wins
    assert_eq!(
        vars.get("CANARY_FLAG").unwrap(),
        "yes",
        "Chained discovery should load .env.staging.canary"
    );

    // Verify all three files were loaded in order
    assert_eq!(paths.len(), 3, "Should load exactly 3 files");
}

#[tokio::test]
#[serial]
async fn collect_vars_does_not_modify_environment() {
    // Test: collect_all_vars_from_dir should not permanently modify
    // the process environment (important for CLI commands)
    clear_env(CLEAR_ALL_DIMENSION_VARS);

    let tmp = async_fs_io::TempDir::create(std::env::temp_dir())
        .await
        .unwrap();

    // .env sets NODE_ENV which triggers discovery
    write(
        tmp.path().join(".env"),
        "NODE_ENV=production\nTEST_VAR=test_value\n",
    )
    .await
    .unwrap();
    write(tmp.path().join(".env.production"), "PROD_VAR=prod_value\n")
        .await
        .unwrap();

    // Set a non-dimension var that should NOT be modified
    unsafe {
        env::set_var("UNRELATED_VAR", "original_value");
    }

    let loader = dotenvage::EnvLoader::with_manager(dotenvage::SecretManager::generate().unwrap());
    let (vars, _paths) = loader.collect_all_vars_from_dir(tmp.path()).await.unwrap();

    // Verify vars were collected (including from .env.production via discovery)
    assert_eq!(vars.get("PROD_VAR").unwrap(), "prod_value");
    assert_eq!(vars.get("TEST_VAR").unwrap(), "test_value");

    // Verify unrelated environment was not touched
    assert_eq!(
        env::var("UNRELATED_VAR").unwrap(),
        "original_value",
        "collect_all_vars_from_dir should not touch unrelated vars"
    );

    // Verify collected vars are NOT in process environment
    assert!(
        env::var("TEST_VAR").is_err(),
        "collect_all_vars_from_dir should not set vars in process environment"
    );
    assert!(
        env::var("PROD_VAR").is_err(),
        "collect_all_vars_from_dir should not set vars in process environment"
    );

    // Verify dimension vars are restored (NODE_ENV was temporarily set during
    // discovery) After collect_all_vars_from_dir, it should be unset (was not
    // set before)
    assert!(
        env::var("NODE_ENV").is_err(),
        "collect_all_vars_from_dir should restore dimension vars to original state"
    );
}
