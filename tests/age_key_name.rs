//! Tests for AGE_KEY_NAME feature - project-specific key discovery

use std::env;

use serial_test::serial;

mod common;
use common::{
    ensure_dir,
    read,
    write,
};

fn clear_key_env() {
    unsafe {
        env::remove_var("AGE_KEY");
        env::remove_var("DOTENVAGE_AGE_KEY");
        env::remove_var("AGE_KEY_NAME");
        env::remove_var("XDG_STATE_HOME");
    }
}

#[tokio::test]
#[serial]
async fn age_key_name_from_env() {
    clear_key_env();

    let tmpdir = async_fs_io::TempDir::create(std::env::temp_dir())
        .await
        .unwrap();
    let state_dir = tmpdir.path().join("state");
    let key_dir = state_dir.join("myproject").join("myapp");
    ensure_dir(&key_dir).await.unwrap();

    // Generate a key and save it
    let manager = dotenvage::SecretManager::generate().unwrap();
    let key_path = key_dir.join("test.key");
    manager.save_key(&key_path).await.unwrap();

    // Set up environment
    unsafe {
        env::set_var("XDG_STATE_HOME", state_dir.to_str().unwrap());
        env::set_var("AGE_KEY_NAME", "myproject/myapp/test");
    }

    // Load key should find it via AGE_KEY_NAME
    let loaded_manager = dotenvage::SecretManager::load_key().await.unwrap();

    // Verify it's the same key by encrypting/decrypting
    let plaintext = "test-secret-123";
    let encrypted = manager.encrypt_value(plaintext).unwrap();
    let decrypted = loaded_manager.decrypt_value(&encrypted).unwrap();
    assert_eq!(plaintext, decrypted);

    // Verify the key path is correct
    let expected_path = state_dir.join("myproject/myapp/test.key");
    assert_eq!(
        dotenvage::SecretManager::key_path_from_env_or_default(),
        expected_path
    );
}

#[tokio::test]
#[serial]
async fn age_key_name_default_to_cargo_pkg_name() {
    clear_key_env();

    let tmpdir = async_fs_io::TempDir::create(std::env::temp_dir())
        .await
        .unwrap();
    let state_dir = tmpdir.path().join("state");

    unsafe {
        env::set_var("XDG_STATE_HOME", state_dir.to_str().unwrap());
    }

    // Without AGE_KEY_NAME, should default to {CARGO_PKG_NAME}/dotenvage
    let path = dotenvage::SecretManager::key_path_from_env_or_default();
    let expected = state_dir.join("dotenvage").join("dotenvage.key");
    assert_eq!(path, expected);
}

#[tokio::test]
#[serial]
async fn age_key_name_with_env_loader() {
    clear_key_env();

    let tmpdir = async_fs_io::TempDir::create(std::env::temp_dir())
        .await
        .unwrap();
    let state_dir = tmpdir.path().join("state");
    let key_dir = state_dir.join("testapp");
    ensure_dir(&key_dir).await.unwrap();

    // Generate a key and save it
    let manager = dotenvage::SecretManager::generate().unwrap();
    let key_path = key_dir.join("dotenvage.key");
    manager.save_key(&key_path).await.unwrap();

    // Create .env with AGE_KEY_NAME and encrypted secret
    let env_file = tmpdir.path().join(".env");
    let encrypted_value = manager.encrypt_value("supersecret").unwrap();
    write(
        &env_file,
        format!("AGE_KEY_NAME=testapp\nAPI_KEY={}", encrypted_value),
    )
    .await
    .unwrap();

    // Set up environment
    unsafe {
        env::set_var("XDG_STATE_HOME", state_dir.to_str().unwrap());
    }

    // Load environment - should auto-discover key via AGE_KEY_NAME from .env
    let loader = dotenvage::EnvLoader::with_manager(manager);
    let vars = loader.load_env_file(&env_file).await.unwrap();

    assert_eq!(vars.get("AGE_KEY_NAME"), Some(&"testapp".to_string()));
    assert_eq!(vars.get("API_KEY"), Some(&"supersecret".to_string()));
}

#[tokio::test]
#[serial]
async fn age_key_name_priority_order() {
    clear_key_env();

    let tmpdir = async_fs_io::TempDir::create(std::env::temp_dir())
        .await
        .unwrap();
    let state_dir = tmpdir.path().join("state");

    // Generate two different keys
    let manager1 = dotenvage::SecretManager::generate().unwrap();
    let manager2 = dotenvage::SecretManager::generate().unwrap();

    let key1_dir = state_dir.join("project1");
    ensure_dir(&key1_dir).await.unwrap();
    let key1_path = key1_dir.join("dotenvage.key");
    manager1.save_key(&key1_path).await.unwrap();

    // Priority 1: AGE_KEY env var should win
    let key1_content = read(&key1_path).await.unwrap();
    unsafe {
        env::set_var("AGE_KEY", key1_content.trim());
        env::set_var("AGE_KEY_NAME", "project2"); // Should be ignored
        env::set_var("XDG_STATE_HOME", state_dir.to_str().unwrap());
    }

    let loaded = dotenvage::SecretManager::load_key().await.unwrap();
    let plaintext = "test-123";
    let encrypted = manager1.encrypt_value(plaintext).unwrap();
    let decrypted = loaded.decrypt_value(&encrypted).unwrap();
    assert_eq!(plaintext, decrypted);

    // Should NOT be able to decrypt with manager2
    assert!(
        loaded
            .decrypt_value(&manager2.encrypt_value(plaintext).unwrap())
            .is_err()
            || loaded
                .decrypt_value(&manager2.encrypt_value(plaintext).unwrap())
                .unwrap()
                != plaintext
    );
}

#[tokio::test]
#[serial]
async fn age_key_name_empty_string_uses_default() {
    clear_key_env();

    let tmpdir = async_fs_io::TempDir::create(std::env::temp_dir())
        .await
        .unwrap();
    let state_dir = tmpdir.path().join("state");

    unsafe {
        env::set_var("XDG_STATE_HOME", state_dir.to_str().unwrap());
        env::set_var("AGE_KEY_NAME", "   "); // Whitespace only
    }

    // Empty/whitespace AGE_KEY_NAME should fall back to default
    let path = dotenvage::SecretManager::key_path_from_env_or_default();
    let expected = state_dir.join("dotenvage").join("dotenvage.key");
    assert_eq!(path, expected);
}
