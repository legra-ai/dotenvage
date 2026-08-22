//! Integration tests for retrieving all environment variable names.

use async_fs_io::TempDir;
use dotenvage::{
    EnvLoader,
    SecretManager,
};
use serial_test::serial;

mod common;
use common::write;

#[tokio::test]
async fn test_get_all_variable_names() {
    let temp_dir = TempDir::create(std::env::temp_dir()).await.unwrap();
    let temp_path = temp_dir.path();

    // Create a test key
    let manager = SecretManager::generate().unwrap();

    // Create test .env files
    write(
        temp_path.join(".env"),
        "DATABASE_URL=postgres://localhost\nPORT=3000\n",
    )
    .await
    .unwrap();

    write(
        temp_path.join(".env.local"),
        "API_KEY=secret123\nDEBUG=true\n",
    )
    .await
    .unwrap();

    // Create loader and get all variable names
    let loader = EnvLoader::with_manager(manager);
    let variable_names = loader
        .get_all_variable_names_from_dir(temp_path)
        .await
        .unwrap();

    // Verify all variables are collected
    assert_eq!(variable_names.len(), 4);
    assert!(variable_names.contains(&"DATABASE_URL".to_string()));
    assert!(variable_names.contains(&"PORT".to_string()));
    assert!(variable_names.contains(&"API_KEY".to_string()));
    assert!(variable_names.contains(&"DEBUG".to_string()));
}

#[tokio::test]
async fn test_get_all_variable_names_with_duplicates() {
    let temp_dir = TempDir::create(std::env::temp_dir()).await.unwrap();
    let temp_path = temp_dir.path();

    let manager = SecretManager::generate().unwrap();

    // Create test .env files with duplicate variable names
    write(temp_path.join(".env"), "DATABASE_URL=base\nPORT=3000\n")
        .await
        .unwrap();

    write(
        temp_path.join(".env.local"),
        "DATABASE_URL=override\nAPI_KEY=secret\n",
    )
    .await
    .unwrap();

    let loader = EnvLoader::with_manager(manager);
    let variable_names = loader
        .get_all_variable_names_from_dir(temp_path)
        .await
        .unwrap();

    // Verify duplicates are removed
    assert_eq!(variable_names.len(), 3);
    assert!(variable_names.contains(&"DATABASE_URL".to_string()));
    assert!(variable_names.contains(&"PORT".to_string()));
    assert!(variable_names.contains(&"API_KEY".to_string()));

    // DATABASE_URL should only appear once despite being in both files
    assert_eq!(
        variable_names
            .iter()
            .filter(|&name| name == "DATABASE_URL")
            .count(),
        1
    );
}

#[tokio::test]
async fn test_get_all_variable_names_with_encrypted_values() {
    let temp_dir = TempDir::create(std::env::temp_dir()).await.unwrap();
    let temp_path = temp_dir.path();

    let manager = SecretManager::generate().unwrap();

    // Create an encrypted value
    let encrypted_secret = manager.encrypt_value("my-secret-token").unwrap();

    // Create test .env file with encrypted value
    write(
        temp_path.join(".env"),
        format!("PUBLIC_VAR=not-secret\nSECRET_TOKEN={}\n", encrypted_secret),
    )
    .await
    .unwrap();

    let loader = EnvLoader::with_manager(manager);
    let variable_names = loader
        .get_all_variable_names_from_dir(temp_path)
        .await
        .unwrap();

    // Verify encrypted variables are included in the names
    assert_eq!(variable_names.len(), 2);
    assert!(variable_names.contains(&"PUBLIC_VAR".to_string()));
    assert!(variable_names.contains(&"SECRET_TOKEN".to_string()));
}

#[tokio::test]
async fn test_get_all_variable_names_empty_directory() {
    let temp_dir = TempDir::create(std::env::temp_dir()).await.unwrap();
    let temp_path = temp_dir.path();

    let manager = SecretManager::generate().unwrap();
    let loader = EnvLoader::with_manager(manager);
    let variable_names = loader
        .get_all_variable_names_from_dir(temp_path)
        .await
        .unwrap();

    // Should return empty vector for directory with no .env files
    assert_eq!(variable_names.len(), 0);
}

#[tokio::test]
#[serial]
async fn test_get_all_variable_names_with_dynamic_discovery() {
    // Test that get_all_variable_names uses dynamic dimension discovery
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
        "OXIGRAPH_VAR=oxigraph\n",
    )
    .await
    .unwrap();

    let loader = EnvLoader::with_manager(manager);
    let variable_names = loader
        .get_all_variable_names_from_dir(temp_path)
        .await
        .unwrap();

    // Verify that OXIGRAPH_VAR is included (from dynamically discovered file)
    assert!(
        variable_names.contains(&"OXIGRAPH_VAR".to_string()),
        "get_all_variable_names should use dynamic discovery to find variables \
         from .env.local.oxigraph (loaded because EKG_VARIANT=oxigraph in .env)"
    );
    assert!(variable_names.contains(&"BASE_VAR".to_string()));
    assert!(variable_names.contains(&"LOCAL_VAR".to_string()));
    assert!(variable_names.contains(&"EKG_VARIANT".to_string()));
}
