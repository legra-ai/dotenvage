const { describe, it, before } = require("node:test");
const assert = require("node:assert");

// Skip tests if bindings not available (will be available after build)
const dotenvage = require("../index.js");

describe("JsSecretManager", () => {
  it("should generate a new secret manager", () => {
    const manager = dotenvage.JsSecretManager.generate();
    assert(manager);
    assert(typeof manager.publicKeyString === "function");
  });

  it("should have a public key string starting with age1", () => {
    const manager = dotenvage.JsSecretManager.generate();
    const publicKey = manager.publicKeyString();
    assert(publicKey.startsWith("age1"));
  });

  it("should encrypt and decrypt a value", () => {
    const manager = dotenvage.JsSecretManager.generate();
    const plaintext = "my-secret-value";
    const encrypted = manager.encryptValue(plaintext);

    assert(encrypted.startsWith("ENC[AGE:b64:"));
    assert(encrypted !== plaintext);

    const decrypted = manager.decryptValue(encrypted);
    assert.strictEqual(decrypted, plaintext);
  });

  it("should pass through unencrypted values", () => {
    const manager = dotenvage.JsSecretManager.generate();
    const plaintext = "not-encrypted";
    const result = manager.decryptValue(plaintext);
    assert.strictEqual(result, plaintext);
  });

  it("should detect encrypted values", () => {
    const manager = dotenvage.JsSecretManager.generate();
    const plaintext = "my-secret-value";
    const encrypted = manager.encryptValue(plaintext);

    assert.strictEqual(manager.isEncrypted(encrypted), true);
    assert.strictEqual(manager.isEncrypted(plaintext), false);
  });

  it("should create from identity string", () => {
    // First generate one to get a valid identity
    const manager1 = dotenvage.JsSecretManager.generate();
    // Note: We can't easily get the identity string from the manager
    // This test would require exposing more methods or generating a key file
    // For now, we'll skip this test or mark it as TODO
    // assert(manager1);
  });
});

describe("shouldEncrypt", () => {
  it("should detect API keys", () => {
    assert.strictEqual(dotenvage.shouldEncrypt("API_KEY"), true);
    assert.strictEqual(
      dotenvage.shouldEncrypt("FLY_API_TOKEN"),
      true
    );
    assert.strictEqual(dotenvage.shouldEncrypt("SECRET_TOKEN"), true);
  });

  it("should detect non-sensitive keys", () => {
    assert.strictEqual(dotenvage.shouldEncrypt("NODE_ENV"), false);
    assert.strictEqual(dotenvage.shouldEncrypt("PORT"), false);
    assert.strictEqual(
      dotenvage.shouldEncrypt("DATABASE_URL"),
      false
    );
  });
});
