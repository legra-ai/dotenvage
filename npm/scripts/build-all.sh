#!/bin/bash
# Build NAPI bindings for all configured targets
# Note: Cross-compilation requires proper toolchains. This script attempts
# to build for all targets but gracefully handles failures.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
NPM_DIR="$(dirname "$SCRIPT_DIR")"
CARGO_TOML="$NPM_DIR/dotenvage-napi/Cargo.toml"

# Detect current platform
UNAME_SYS="$(uname -s | tr '[:upper:]' '[:lower:]')"
UNAME_MACHINE="$(uname -m | tr '[:upper:]' '[:lower:]')"

# Determine native architecture
if [[ "$UNAME_MACHINE" == "x86_64" ]] || [[ "$UNAME_MACHINE" == "amd64" ]]; then
  NATIVE_ARCH="x86_64"
elif [[ "$UNAME_MACHINE" == "aarch64" ]] || [[ "$UNAME_MACHINE" == "arm64" ]]; then
  NATIVE_ARCH="aarch64"
else
  NATIVE_ARCH="unknown"
fi

# All possible targets from package.json napi.targets
ALL_TARGETS=(
  "x86_64-unknown-linux-gnu"
  "x86_64-unknown-linux-musl"
  "aarch64-unknown-linux-gnu"
  "aarch64-unknown-linux-musl"
  "aarch64-apple-darwin"
  "x86_64-pc-windows-msvc"
  "aarch64-pc-windows-msvc"
  "i686-pc-windows-msvc"
)

# Filter targets based on current platform
TARGETS=()

if [[ "$UNAME_SYS" == "darwin" ]]; then
  # On macOS, only build native darwin targets
  # Linux targets require cross-compilation toolchains (not available by default)
  # Windows targets cannot be cross-compiled from macOS
  echo "Platform: macOS ($NATIVE_ARCH)"
  for target in "${ALL_TARGETS[@]}"; do
    if [[ "$target" == *"apple-darwin"* ]]; then
      TARGETS+=("$target")
    fi
  done
elif [[ "$UNAME_SYS" == "linux" ]]; then
  # On Linux, build native Linux targets
  echo "Platform: Linux ($NATIVE_ARCH)"
  for target in "${ALL_TARGETS[@]}"; do
    if [[ "$target" == *"linux"* ]]; then
      TARGETS+=("$target")
    fi
  done
elif [[ "$UNAME_SYS" == *"mingw"* ]] || [[ "$UNAME_SYS" == *"msys"* ]] || [[ "$UNAME_SYS" == *"cygwin"* ]]; then
  # On Windows, build native Windows targets
  echo "Platform: Windows ($NATIVE_ARCH)"
  for target in "${ALL_TARGETS[@]}"; do
    if [[ "$target" == *"windows"* ]] || [[ "$target" == *"msvc"* ]]; then
      TARGETS+=("$target")
    fi
  done
else
  # Unknown platform, try to build all (will fail gracefully)
  echo "Platform: Unknown ($UNAME_SYS), attempting all targets"
  TARGETS=("${ALL_TARGETS[@]}")
fi

# Allow override via environment variable (for CI or advanced users)
if [[ -n "$NAPI_BUILD_TARGETS" ]]; then
  echo "Using targets from NAPI_BUILD_TARGETS: $NAPI_BUILD_TARGETS"
  IFS=',' read -ra TARGETS <<< "$NAPI_BUILD_TARGETS"
fi

if [[ ${#TARGETS[@]} -eq 0 ]]; then
  echo "❌ No targets selected for platform: $UNAME_SYS"
  exit 1
fi

echo "Building for ${#TARGETS[@]} target(s): ${TARGETS[*]}"
echo ""

cd "$NPM_DIR" || {
  echo "❌ Failed to change to directory: $NPM_DIR"
  exit 1
}

BUILT=0
FAILED=0

# Disable exit on error for the build loop (we want to continue on failures)
set +e

for target in "${TARGETS[@]}"; do
  echo "Building for target: $target"
  if napi build \
    --manifest-path "$CARGO_TOML" \
    --output-dir . \
    --platform \
    --release \
    --target "$target" 2>&1; then
    ((BUILT++))
    echo "✅ Successfully built for $target"
  else
    ((FAILED++))
    echo "⚠️  Failed to build for $target (cross-compilation toolchain may be required)"
  fi
  echo ""
done

# Re-enable exit on error for final checks
set -e

echo "Build summary: $BUILT succeeded, $FAILED failed"
if [ $BUILT -eq 0 ]; then
  echo "❌ No targets built successfully"
  exit 1
fi
