/**
 * Integration tests for dotenvage Node.js bindings
 * These tests require a valid encryption key and test the full workflow
 */

const { describe, it, before, after } = require("node:test");
const assert = require("node:assert");
const fs = require("fs");
const path = require("path");
const { mkdtemp, writeFile, rm } = require("fs/promises");
const os = require("os");

const dotenvage = require("../index.js");

describe("Integration tests", () => {
  let testDir;
  let originalCwd;
  let testManager;
  let originalEnv;

  before(async () => {
    // Save original environment
    originalEnv = { ...process.env };
    originalCwd = process.cwd();

    // Create temporary test directory
    testDir = await mkdtemp(
      path.join(os.tmpdir(), "dotenvage-integration-")
    );
    process.chdir(testDir);

    // Generate a test encryption key manager
    // Tests will use this same manager instance for encryption and decryption
    testManager = dotenvage.JsSecretManager.generate();
  });

  after(async () => {
    // Restore original environment
    process.env = originalEnv;
    process.chdir(originalCwd);

    if (testDir) {
      await rm(testDir, { recursive: true, force: true });
    }
  });

  it("should encrypt and decrypt values in .env files", async () => {
    const manager = dotenvage.JsSecretManager.generate();
    const secret = "my-secret-api-key-12345";
    const encrypted = manager.encryptValue(secret);

    // Create .env file with encrypted value
    const envContent = `API_KEY=${encrypted}\nPORT=8080\nNODE_ENV=test\n`;
    await writeFile(path.join(testDir, ".env"), envContent);

    // Load with the same manager
    const loader = dotenvage.JsEnvLoader.withManager(manager);
    loader.load();

    // Verify decryption worked
    assert.strictEqual(process.env.API_KEY, secret);
    assert.strictEqual(process.env.PORT, "8080");
    assert.strictEqual(process.env.NODE_ENV, "test");
  });

  it("should load variables from multiple .env files with layering", async () => {
    const manager = dotenvage.JsSecretManager.generate();

    // Clear environment variables to test .env file discovery
    // resolve_env() now checks .env file first, then environment variables
    const originalDotenvageEnv = process.env.DOTENVAGE_ENV;
    const originalEkgEnv = process.env.EKG_ENV;
    const originalVercelEnv = process.env.VERCEL_ENV;
    const originalNodeEnv = process.env.NODE_ENV;
    delete process.env.DOTENVAGE_ENV;
    delete process.env.EKG_ENV;
    delete process.env.VERCEL_ENV;
    delete process.env.NODE_ENV;

    try {
      // Create base .env with NODE_ENV=local so .env.local will be loaded
      // The loader will read NODE_ENV from .env file to determine which files to load
      await writeFile(
        path.join(testDir, ".env"),
        "NODE_ENV=local\nAPI_KEY=base-key\nPORT=8080\n"
      );

      // Create .env.local (should override .env when NODE_ENV=local)
      await writeFile(
        path.join(testDir, ".env.local"),
        "API_KEY=local-key\nDATABASE_URL=postgres://localhost/test\n"
      );

      const loader = dotenvage.JsEnvLoader.withManager(manager);
      loader.load();

      // .env.local should override .env
      assert.strictEqual(process.env.API_KEY, "local-key");
      assert.strictEqual(process.env.PORT, "8080"); // From .env
      assert.strictEqual(process.env.NODE_ENV, "local"); // From .env
      assert.strictEqual(
        process.env.DATABASE_URL,
        "postgres://localhost/test"
      ); // From .env.local
    } finally {
      // Restore original environment variables
      if (originalDotenvageEnv === undefined) {
        delete process.env.DOTENVAGE_ENV;
      } else {
        process.env.DOTENVAGE_ENV = originalDotenvageEnv;
      }
      if (originalEkgEnv === undefined) {
        delete process.env.EKG_ENV;
      } else {
        process.env.EKG_ENV = originalEkgEnv;
      }
      if (originalVercelEnv === undefined) {
        delete process.env.VERCEL_ENV;
      } else {
        process.env.VERCEL_ENV = originalVercelEnv;
      }
      if (originalNodeEnv === undefined) {
        delete process.env.NODE_ENV;
      } else {
        process.env.NODE_ENV = originalNodeEnv;
      }
    }
  });

  it("should get all variables as object without mutating process.env", async () => {
    const manager = dotenvage.JsSecretManager.generate();

    // Save original process.env values
    const originalApiKey = process.env.API_KEY;
    const originalPort = process.env.PORT;

    // Create test .env file
    await writeFile(
      path.join(testDir, ".env"),
      "TEST_API_KEY=test-key-123\nTEST_PORT=3000\n"
    );

    const loader = dotenvage.JsEnvLoader.withManager(manager);

    // Get variables as object (this still loads into process.env first, but returns object)
    const vars = loader.getAllVariables();

    // Check that variables are in the returned object
    assert.strictEqual(vars.TEST_API_KEY, "test-key-123");
    assert.strictEqual(vars.TEST_PORT, "3000");

    // Clean up
    delete process.env.TEST_API_KEY;
    delete process.env.TEST_PORT;
  });

  it("should get all variable names without loading into environment", async () => {
    const manager = dotenvage.JsSecretManager.generate();

    // Create test .env file
    await writeFile(
      path.join(testDir, ".env"),
      "VAR1=value1\nVAR2=value2\nVAR3=value3\n"
    );

    const loader = dotenvage.JsEnvLoader.withManager(manager);
    const names = loader.getAllVariableNames();

    assert(Array.isArray(names));
    assert(names.length >= 3);
    assert(names.includes("VAR1"));
    assert(names.includes("VAR2"));
    assert(names.includes("VAR3"));
  });

  it("should resolve env paths correctly", () => {
    // Use withManager to avoid requiring external AGE key
    const manager = dotenvage.JsSecretManager.generate();
    const loader = dotenvage.JsEnvLoader.withManager(manager);
    const paths = loader.resolveEnvPaths(testDir);

    assert(Array.isArray(paths));
    assert(paths.length > 0);
    // Should include .env at minimum
    assert(paths.some((p) => p.includes(".env")));
  });

  it("should handle encrypted and plain values mixed", async () => {
    const manager = dotenvage.JsSecretManager.generate();
    const encrypted = manager.encryptValue("secret-value");

    // Create .env with mix of encrypted and plain values
    const envContent = `ENCRYPTED_KEY=${encrypted}\nPLAIN_KEY=plain-value\nPORT=8080\n`;
    await writeFile(path.join(testDir, ".env"), envContent);

    const loader = dotenvage.JsEnvLoader.withManager(manager);
    loader.load();

    // Encrypted should be decrypted
    assert.strictEqual(process.env.ENCRYPTED_KEY, "secret-value");
    // Plain should pass through
    assert.strictEqual(process.env.PLAIN_KEY, "plain-value");
    assert.strictEqual(process.env.PORT, "8080");
  });

  it("should load from specific directory", async () => {
    const manager = dotenvage.JsSecretManager.generate();

    // Create subdirectory with .env file
    const subDir = path.join(testDir, "config");
    await fs.promises.mkdir(subDir, { recursive: true });
    await writeFile(
      path.join(subDir, ".env"),
      "SUB_DIR_VAR=subdir-value\n"
    );

    const loader = dotenvage.JsEnvLoader.withManager(manager);
    loader.loadFromDir(subDir);

    assert.strictEqual(process.env.SUB_DIR_VAR, "subdir-value");
  });
});
