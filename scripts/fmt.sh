#!/bin/sh
# Use the pinned nightly formatter independently of the compiler toolchain.
script_directory=$(dirname "$0") || exit 1
repository_root=$(cd "$script_directory/.." && pwd) || exit 1
cd "$repository_root" || exit 1
if ! IFS= read -r formatter_toolchain < rustfmt-toolchain; then
    printf 'Cannot read rustfmt-toolchain\n' >&2
    exit 1
fi
exec cargo "+$formatter_toolchain" fmt --all -- "$@"
