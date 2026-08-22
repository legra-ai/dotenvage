//! Integration tests for the dump-to-writer functionality.

use async_fs_io::TempDir;
use dotenvage::SecretManager;
use dotenvage::loader::EnvLoader;
use serial_test::serial;

mod common;
use common::write;

#[tokio::test]
async fn test_dump_to_writer_basic() {
    let temp_dir = TempDir::create(std::env::temp_dir()).await.unwrap();
    let temp_path = temp_dir.path();

    // Create test .env files
    write(temp_path.join(".env"), "VAR1=value1\nVAR2=value2")
        .await
        .unwrap();
    write(temp_path.join(".env.local"), "VAR3=value3\nVAR1=overridden")
        .await
        .unwrap();

    let manager = SecretManager::generate().unwrap();
    let loader = EnvLoader::with_manager(manager);

    let mut buffer = Vec::new();
    loader
        .dump_to_writer_from_dir(temp_path, &mut buffer)
        .await
        .unwrap();
    let output = String::from_utf8(buffer).unwrap();

    // VAR1 should be overridden by .env.local
    assert!(output.contains("VAR1=overridden"));
    assert!(output.contains("VAR2=value2"));
    assert!(output.contains("VAR3=value3"));

    // Variables should be sorted
    let lines: Vec<&str> = output.lines().collect();
    assert_eq!(lines[0], "VAR1=overridden");
    assert_eq!(lines[1], "VAR2=value2");
    assert_eq!(lines[2], "VAR3=value3");
}

#[tokio::test]
async fn test_dump_to_writer_filters_age_keys() {
    let temp_dir = TempDir::create(std::env::temp_dir()).await.unwrap();
    let temp_path = temp_dir.path();

    // Create test .env file with AGE keys
    write(
        temp_path.join(".env"),
        "VAR1=value1\nDOTENVAGE_AGE_KEY=secret_key\nAGE_KEY=another_key\nVAR2=value2\nEKG_AGE_KEY=ekg_key\nAGE_KEY_NAME=mykey\nPROJECT_AGE_KEY_NAME=project_key",
    ).await.unwrap();

    let manager = SecretManager::generate().unwrap();
    let loader = EnvLoader::with_manager(manager);

    let mut buffer = Vec::new();
    loader
        .dump_to_writer_from_dir(temp_path, &mut buffer)
        .await
        .unwrap();
    let output = String::from_utf8(buffer).unwrap();

    // Regular variables should be present
    assert!(
        output.contains("VAR1=value1"),
        "Output should contain VAR1=value1"
    );
    assert!(
        output.contains("VAR2=value2"),
        "Output should contain VAR2=value2"
    );

    // AGE key variables should be filtered out
    assert!(
        !output.contains("DOTENVAGE_AGE_KEY"),
        "Should not contain DOTENVAGE_AGE_KEY"
    );
    assert!(!output.contains("AGE_KEY="), "Should not contain AGE_KEY=");
    assert!(
        !output.contains("EKG_AGE_KEY"),
        "Should not contain EKG_AGE_KEY"
    );
    assert!(
        !output.contains("AGE_KEY_NAME"),
        "Should not contain AGE_KEY_NAME"
    );
    assert!(
        !output.contains("PROJECT_AGE_KEY_NAME"),
        "Should not contain PROJECT_AGE_KEY_NAME"
    );
}

#[tokio::test]
async fn test_dump_to_writer_handles_encrypted_values() {
    let temp_dir = TempDir::create(std::env::temp_dir()).await.unwrap();
    let temp_path = temp_dir.path();

    let manager = SecretManager::generate().unwrap();
    let encrypted = manager.encrypt_value("secret_value").unwrap();

    // Create test .env file with encrypted value
    write(
        temp_path.join(".env"),
        format!("VAR1=plain\nVAR2={}", encrypted),
    )
    .await
    .unwrap();

    let loader = EnvLoader::with_manager(manager);

    let mut buffer = Vec::new();
    loader
        .dump_to_writer_from_dir(temp_path, &mut buffer)
        .await
        .unwrap();
    let output = String::from_utf8(buffer).unwrap();

    // Should contain decrypted value, not encrypted
    assert!(output.contains("VAR1=plain"));
    assert!(output.contains("VAR2=secret_value"));
    assert!(!output.contains("age:"));
}

#[tokio::test]
async fn test_dump_to_writer_quotes_special_values() {
    let temp_dir = TempDir::create(std::env::temp_dir()).await.unwrap();
    let temp_path = temp_dir.path();

    // Create test .env file with values that need quoting
    write(
        temp_path.join(".env"),
        "VAR1=simple\nVAR2=has spaces\nVAR3=has\"quotes\nVAR4=\nVAR5=has=equals",
    )
    .await
    .unwrap();

    let manager = SecretManager::generate().unwrap();
    let loader = EnvLoader::with_manager(manager);

    let mut buffer = Vec::new();
    loader
        .dump_to_writer_from_dir(temp_path, &mut buffer)
        .await
        .unwrap();
    let output = String::from_utf8(buffer).unwrap();

    // Simple value should not be quoted
    assert!(output.contains("VAR1=simple"));

    // Values with special characters should be quoted
    assert!(output.contains("VAR2=\"has spaces\""));
    assert!(output.contains("VAR3=\"has\\\"quotes\""));
    assert!(output.contains("VAR4=\"\""));
    assert!(output.contains("VAR5=\"has=equals\""));
}

#[tokio::test]
#[serial]
async fn test_dump_to_writer_with_dynamic_discovery() {
    // Test that dump_to_writer uses dynamic dimension discovery
    // This ensures variables from files loaded via discovered dimensions are
    // included

    // Clear dimension env vars to ensure clean test state
    for var in [
        "DOTENVAGE_ENV",
        "EKG_ENV",
        "VERCEL_ENV",
        "NODE_ENV",
        "DOTENVAGE_VARIANT",
        "EKG_VARIANT",
        "VARIANT",
    ] {
        unsafe {
            std::env::remove_var(var);
        }
    }

    let temp_dir = TempDir::create(std::env::temp_dir()).await.unwrap();
    let temp_path = temp_dir.path();

    let manager = SecretManager::generate().unwrap();

    // .env sets EKG_VARIANT=oxigraph (dimension discovery)
    write(
        temp_path.join(".env"),
        "BASE_VAR=base\nEKG_VARIANT=oxigraph\n",
    )
    .await
    .unwrap();

    // .env.local is loaded (ENV defaults to local)
    write(temp_path.join(".env.local"), "LOCAL_VAR=local\n")
        .await
        .unwrap();

    // .env.local.oxigraph should be loaded via dynamic discovery
    write(
        temp_path.join(".env.local.oxigraph"),
        "OXIGRAPH_VAR=oxigraph_value\n",
    )
    .await
    .unwrap();

    let loader = EnvLoader::with_manager(manager);

    let mut buffer = Vec::new();
    loader
        .dump_to_writer_from_dir(temp_path, &mut buffer)
        .await
        .unwrap();
    let output = String::from_utf8(buffer).unwrap();

    // Verify that OXIGRAPH_VAR is included (from dynamically discovered file)
    assert!(
        output.contains("OXIGRAPH_VAR=oxigraph_value"),
        "dump_to_writer should use dynamic discovery to include variables \
         from .env.local.oxigraph (loaded because EKG_VARIANT=oxigraph in .env). \
         Output was: {}",
        output
    );
    assert!(output.contains("BASE_VAR=base"));
    assert!(output.contains("LOCAL_VAR=local"));
}
