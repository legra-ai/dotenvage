//! Public values must survive the writer, normal reader and update reader.
//! These exercise the existing synchronous filesystem boundary itself.

use dotenvage::{
    EnvLoader,
    NonSecret,
    SecretManager,
};
use tempfile::TempDir;

fn require_round_trip(value: &str) {
    let directory = TempDir::new().unwrap();
    let path = directory.path().join(".env");
    let loader = EnvLoader::with_manager(SecretManager::generate().unwrap());
    loader
        .set_write_var_in_file(NonSecret("PUBLIC_CONFIGURATION", value), &path)
        .unwrap();
    let loaded = loader.load_env_file(&path).unwrap();
    assert_eq!(
        loaded.get("PUBLIC_CONFIGURATION").map(String::as_str),
        Some(value)
    );

    // Updating another key must use the same quoting rules as normal loading.
    loader
        .set_write_var_in_file(NonSecret("OTHER_SETTING", "ready"), &path)
        .unwrap();
    let loaded = loader.load_env_file(&path).unwrap();
    assert_eq!(
        loaded.get("PUBLIC_CONFIGURATION").map(String::as_str),
        Some(value)
    );
}

#[test]
fn public_json_round_trips_without_extra_backslashes() {
    require_round_trip(r#"{"enabled":true}"#);
    require_round_trip(r#"{"service":"example","labels":["a \"quote\"","C:\\cache"]}"#);
}

#[test]
fn single_quoted_input_stays_literal_through_updates() {
    let directory = TempDir::new().unwrap();
    let path = directory.path().join(".env");
    let value = r#"C:\new\cache ${LITERAL_SETTING} "quoted" # text"#;
    std::fs::write(&path, format!("PUBLIC_CONFIGURATION='{value}'\n")).unwrap();
    let loader = EnvLoader::with_manager(SecretManager::generate().unwrap());
    assert_eq!(
        loader.load_env_file(&path).unwrap()["PUBLIC_CONFIGURATION"],
        value
    );
    loader
        .set_write_var_in_file(NonSecret("OTHER_SETTING", "ready"), &path)
        .unwrap();
    assert_eq!(
        loader.load_env_file(&path).unwrap()["PUBLIC_CONFIGURATION"],
        value
    );
}

#[test]
fn public_backslashes_round_trip_through_updates() {
    require_round_trip(r"directory C:\new\configuration with spaces");
}

#[test]
fn public_dollars_stay_literal_when_another_setting_is_updated() {
    assert!(std::env::var_os("DOTENVAGE_ROUNDTRIP_UNSET_421947").is_none());
    require_round_trip("${DOTENVAGE_ROUNDTRIP_UNSET_421947}");
}

#[test]
fn public_line_breaks_and_whitespace_round_trip() {
    // bounded: three fixed strings exercising ordinary quoted whitespace.
    for value in ["first\nsecond", "first\r\nsecond", "\t spaced 🦀 value \t"] {
        require_round_trip(value);
    }
}

#[test]
fn public_quote_and_comment_characters_are_values() {
    // bounded: five fixed values whose characters have dotenv syntax meaning.
    for value in [
        "'literal quotes'",
        "#literal",
        "one # two",
        "",
        "\"quoted\"",
    ] {
        require_round_trip(value);
    }
}

#[test]
fn unquoted_input_retains_its_literal_value_rules() {
    let loader = EnvLoader::with_manager(SecretManager::generate().unwrap());
    let values = loader.parse_and_decrypt(
        "WORDS=has spaces\nQUOTE=has\"quotes\nHASH=#literal\nDOLLAR=${DOTENVAGE_ROUNDTRIP_UNSET_421947}\n",
        std::path::Path::new("literal.env"),
    ).unwrap();
    assert_eq!(values.get("WORDS").map(String::as_str), Some("has spaces"));
    assert_eq!(values.get("QUOTE").map(String::as_str), Some("has\"quotes"));
    assert_eq!(values.get("HASH").map(String::as_str), Some("#literal"));
    assert_eq!(
        values.get("DOLLAR").map(String::as_str),
        Some("${DOTENVAGE_ROUNDTRIP_UNSET_421947}")
    );
}
