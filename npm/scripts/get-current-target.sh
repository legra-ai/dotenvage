#!/bin/bash
# Get the current platform's Rust target triple
# This is used by the build scripts to only build for the current platform

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

# Map platform and architecture to Rust target triple
if [[ "$UNAME_SYS" == "darwin" ]]; then
  if [[ "$NATIVE_ARCH" == "aarch64" ]]; then
    echo "aarch64-apple-darwin"
  else
    echo "unsupported: Intel macOS (x86_64-apple-darwin) is no longer supported" >&2
    exit 1
  fi
elif [[ "$UNAME_SYS" == "linux" ]]; then
  if [[ "$NATIVE_ARCH" == "aarch64" ]]; then
    echo "aarch64-unknown-linux-gnu"
  else
    echo "x86_64-unknown-linux-gnu"
  fi
elif [[ "$UNAME_SYS" == *"mingw"* ]] || [[ "$UNAME_SYS" == *"msys"* ]] || [[ "$UNAME_SYS" == *"cygwin"* ]]; then
  if [[ "$NATIVE_ARCH" == "aarch64" ]]; then
    echo "aarch64-pc-windows-msvc"
  else
    echo "x86_64-pc-windows-msvc"
  fi
else
  echo "unknown-unknown-unknown"
  exit 1
fi
