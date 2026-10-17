/**
 * Basic tests for dotenvage Node.js bindings
 * Run with: npm test or node --test
 */

const { test } = require("node:test");
const assert = require("node:assert");

const dotenvage = require("../index.js");

test("SecretManager.generate() creates a new manager", () => {
  const manager = dotenvage.JsSecretManager.generate();
  assert(manager, "Manager should be created");
  const publicKey = manager.publicKeyString();
  assert(
    publicKey.startsWith("age1"),
    "Public key should start with 'age1'"
  );
});

test("SecretManager encrypt/decrypt round-trip", () => {
  const manager = dotenvage.JsSecretManager.generate();
  const plaintext = "my-secret-value-12345";

  const encrypted = manager.encryptValue(plaintext);
  assert(encrypted.startsWith("ENC[AGE:b64:"), "Should be encrypted");

  const decrypted = manager.decryptValue(encrypted);
  assert.strictEqual(
    decrypted,
    plaintext,
    "Decrypted should match original"
  );
});

test("SecretManager.isEncrypted() detects encrypted values", () => {
  const manager = dotenvage.JsSecretManager.generate();
  const plaintext = "my-secret-value";
  const encrypted = manager.encryptValue(plaintext);

  assert.strictEqual(manager.isEncrypted(encrypted), true);
  assert.strictEqual(manager.isEncrypted(plaintext), false);
});

test("shouldEncrypt() detects keys that should be encrypted", () => {
  assert.strictEqual(dotenvage.shouldEncrypt("API_KEY"), true);
  assert.strictEqual(dotenvage.shouldEncrypt("FLY_API_TOKEN"), true);
  assert.strictEqual(dotenvage.shouldEncrypt("SECRET"), true);
  assert.strictEqual(dotenvage.shouldEncrypt("NODE_ENV"), false);
  assert.strictEqual(dotenvage.shouldEncrypt("PORT"), false);
  assert.strictEqual(dotenvage.shouldEncrypt("DATABASE_URL"), false); // URLs are not encrypted by default
});

test("EnvLoader.new() creates a loader", () => {
  try {
    const loader = dotenvage.JsEnvLoader.new();
    assert(loader, "Loader should be created");
  } catch (error) {
    // This will fail if no key is available, which is expected in CI
    assert(
      error.message.includes("key") || error.message.includes("AGE"),
      "Should fail with key-related error if no key available"
    );
  }
});

test("EnvLoader.resolveEnvPaths() returns array of paths", () => {
  try {
    const loader = dotenvage.JsEnvLoader.new();
    const paths = loader.resolveEnvPaths(".");
    assert(Array.isArray(paths), "Should return an array");
    assert(paths.length > 0, "Should have at least .env path");
    assert(
      paths.includes(".env") || paths.some((p) => p.includes(".env")),
      "Should include .env"
    );
  } catch (error) {
    // new() requires a key, but if it succeeds, resolveEnvPaths works
    // If new() fails due to missing key, that's expected in CI environments
    if (
      error.message.includes("key") ||
      error.message.includes("AGE")
    ) {
      // This is expected in CI - test passes by skipping
      return;
    }
    // Otherwise re-throw unexpected errors
    throw error;
  }
});

// Integration test (requires valid key and .env files)
test(
  "EnvLoader.load() loads variables into process.env",
  { skip: true },
  () => {
    // This test is skipped by default as it requires:
    // 1. A valid encryption key
    // 2. .env files in the test directory
    // To run: Set DOTENVAGE_AGE_KEY and create test .env files
    const loader = dotenvage.JsEnvLoader.new();
    loader.load();
    const names = loader.getAllVariableNames();
    assert(
      Array.isArray(names),
      "Should return array of variable names"
    );
  }
);
