#!/usr/bin/env node

/**
 * Node.js CLI wrapper for dotenvage
 *
 * This wrapper attempts to use the native Rust binary if available,
 * otherwise falls back to using the NAPI bindings to implement CLI functionality.
 */

const { existsSync } = require("fs");
const { join } = require("path");
const { spawn, execSync } = require("child_process");
const { platform, arch } = require("os");

// Determine the binary name based on platform
function getBinaryName() {
  const plat = platform();
  const architecture = arch();

  // Map Node.js arch to Rust target arch
  const archMap = {
    x64: "x86_64",
    arm64: "aarch64",
    ia32: "i686",
  };

  const rustArch = archMap[architecture] || architecture;

  if (plat === "darwin" && architecture === "x64") {
    console.error(
      "dotenvage: unsupported platform: Intel macOS (darwin-x64) is no longer supported; use Apple Silicon (darwin-arm64)"
    );
    process.exit(1);
  }

  if (plat === "win32") {
    return `dotenvage-${rustArch}-pc-windows-msvc.exe`;
  } else if (plat === "darwin") {
    return `dotenvage-${rustArch}-apple-darwin`;
  } else {
    return `dotenvage-${rustArch}-unknown-linux-gnu`;
  }
}

// Try to find the native binary
function findNativeBinary() {
  // Check in common locations
  const possiblePaths = [
    // In the npm package directory
    join(__dirname, "..", "bin", getBinaryName()),
    // In node_modules/.bin
    join(__dirname, "..", "..", "..", "bin", getBinaryName()),
    // Global installation
    join(
      process.env.HOME || process.env.USERPROFILE || "",
      ".cargo",
      "bin",
      "dotenvage"
    ),
    join(
      process.env.HOME || process.env.USERPROFILE || "",
      ".local",
      "bin",
      "dotenvage"
    ),
    // PATH
    "dotenvage",
  ];

  for (const path of possiblePaths) {
    try {
      if (path === "dotenvage") {
        // Check if it's in PATH
        execSync(`which ${path}`, { stdio: "ignore" });
        return path;
      } else if (existsSync(path)) {
        return path;
      }
    } catch (e) {
      // Continue searching
    }
  }

  return null;
}

// Fallback to NAPI implementation
function useNapiImplementation() {
  try {
    const dotenvage = require("../index.js");

    // Parse command line arguments
    const args = process.argv.slice(2);
    const command = args[0];

    switch (command) {
      case "keygen":
      case "gen":
        {
          const manager = dotenvage.JsSecretManagerGenerate();
          console.log(`✓ Private key generated`);
          console.log(
            `Public recipient: ${manager.publicKeyString()}`
          );
          console.log(
            `Note: Use DOTENVAGE_AGE_KEY or AGE_KEY environment variable to set the key`
          );
        }
        break;

      case "get":
        {
          const key = args[1];
          if (!key) {
            console.error("Error: key name required");
            process.exit(1);
          }

          const loader = dotenvage.JsEnvLoaderNew();
          loader.load();

          const value = process.env[key];
          if (value === undefined) {
            console.error(
              `Error: key '${key}' not found in any .env* file`
            );
            process.exit(1);
          }

          console.log(value);
        }
        break;

      case "list":
        {
          const verbose =
            args.includes("--verbose") || args.includes("-v");
          const json = args.includes("--json");
          const plain = args.includes("--plain");
          const fileIndex = args.indexOf("--file");
          const file = fileIndex >= 0 ? args[fileIndex + 1] : null;

          const loader = dotenvage.JsEnvLoaderNew();
          let names;

          if (file) {
            loader.loadFromDir(process.cwd());
            names = loader.getAllVariableNamesFromDir(process.cwd());
          } else {
            loader.load();
            names = loader.getAllVariableNames();
          }

          if (json) {
            const output = {};
            for (const name of names) {
              const value = process.env[name];
              if (value) {
                output[name] = verbose
                  ? { value, encrypted: false }
                  : { encrypted: false };
              }
            }
            console.log(JSON.stringify(output, null, 2));
          } else {
            for (const name of names) {
              if (verbose) {
                const value = process.env[name];
                const lockIcon = "  "; // Can't easily detect encryption without manager
                console.log(`${lockIcon} ${name} = ${value}`);
              } else if (plain) {
                console.log(name);
              } else {
                console.log(`  ${name}`);
              }
            }
          }
        }
        break;

      case "dump":
        {
          const exportFlag =
            args.includes("--export") || args.includes("-e");
          const fileIndex = args.indexOf("--file");
          const file = fileIndex >= 0 ? args[fileIndex + 1] : null;

          const loader = dotenvage.JsEnvLoaderNew();

          if (file) {
            loader.loadFromDir(process.cwd());
          } else {
            loader.load();
          }

          const vars = loader.getAllVariables();
          const prefix = exportFlag ? "export " : "";

          for (const [key, value] of Object.entries(vars)) {
            // Simple escaping for values
            const needsQuotes =
              value.includes(" ") ||
              value.includes("$") ||
              value.includes("`");
            if (needsQuotes) {
              const escaped = value
                .replace(/"/g, '\\"')
                .replace(/\$/g, "\\$");
              console.log(`${prefix}${key}="${escaped}"`);
            } else {
              console.log(`${prefix}${key}=${value}`);
            }
          }
        }
        break;

      default:
        console.error(`Error: Unknown command '${command}'`);
        console.error(`
Usage: dotenvage <command> [options]

Commands:
  keygen, gen          Generate a new encryption key pair
  get <key>            Get a decrypted secret value
  list                 List environment variables
  dump                 Dump all decrypted env vars

Note: This is a limited implementation using NAPI bindings.
For full functionality, install the native Rust binary:
  cargo install dotenvage
        `);
        process.exit(1);
    }
  } catch (error) {
    console.error("Error:", error.message);
    console.error(`
Note: Native bindings not available. Install the native Rust binary:
  cargo install dotenvage
    `);
    process.exit(1);
  }
}

// Main entry point
function main() {
  const binary = findNativeBinary();

  if (binary) {
    // Use native binary
    const child = spawn(binary, process.argv.slice(2), {
      stdio: "inherit",
      shell: false,
    });

    child.on("error", (error) => {
      console.error(`Error running dotenvage: ${error.message}`);
      console.error("Falling back to NAPI implementation...");
      useNapiImplementation();
    });

    child.on("exit", (code) => {
      process.exit(code || 0);
    });
  } else {
    // Fallback to NAPI implementation
    useNapiImplementation();
  }
}

if (require.main === module) {
  main();
}
