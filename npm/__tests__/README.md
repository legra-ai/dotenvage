# Test Suite

Tests for the Node.js bindings of dotenvage.

## Running Tests

### Prerequisites

1. Build the native bindings first:

   From the `npm/` directory:

   ```bash
   cd npm
   npm run build
   ```

   Or from the project root:

   ```bash
   npm run npm:build
   ```

2. (Optional) For integration tests that require external encryption
   keys, set:

   ```bash
   export DOTENVAGE_AGE_KEY="AGE-SECRET-KEY-1..."
   ```

### Run All Tests

```bash
npm test
```

### Run Specific Test Suites

```bash
# Unit tests only (don't require encryption key)
npm run test:unit

# Integration tests (test full workflow)
npm run test:integration

# All tests
npm run test:all
```

## Test Structure

- **`basic.test.js`** - Basic functionality tests (can run without
  key)
- **`secret-manager.test.js`** - SecretManager API tests
- **`env-loader.test.js`** - EnvLoader API tests
- **`integration.test.js`** - Full workflow integration tests

## Test Utilities

The `test-utils.js` file provides helper functions for:

- Creating temporary test directories
- Creating test .env files
- Setting up test managers and loaders
- Cleaning up test artifacts

## Writing New Tests

When writing new tests:

1. Use the test utilities from `test-utils.js` when possible
2. Clean up test directories and environment variables in `after`
   hooks
3. Handle cases where encryption keys might not be available
   gracefully
4. Use `JsSecretManager.generate()` and `JsEnvLoader.withManager()`
   for tests that don't require external keys

## Example Test

```javascript
const { describe, it, before, after } = require("node:test");
const assert = require("node:assert");
const {
  createTestDir,
  cleanupTestDir,
  setupTestEnvironment,
} = require("./test-utils");

const dotenvage = require("../index.js");

describe("My Feature", () => {
  let testDir;

  before(async () => {
    testDir = await createTestDir();
  });

  after(async () => {
    await cleanupTestDir(testDir);
  });

  it("should do something", async () => {
    const { manager, loader } = await setupTestEnvironment(
      dotenvage,
      testDir,
      {
        ".env": "TEST_VAR=value\n",
      }
    );

    loader.load();
    assert.strictEqual(process.env.TEST_VAR, "value");
  });
});
```
