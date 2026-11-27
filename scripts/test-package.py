"""Build and test the actual crates.io archive outside the Git checkout."""

import os
from pathlib import Path
import subprocess
import tarfile
import tempfile
import tomllib


def pinned_toolchain(root):
    """The toolchain rust-toolchain.toml pins for the checkout.

    The archive is built and tested outside the checkout, where rustup no
    longer sees that file and would fall back to the runner's default
    toolchain, so the pin is passed on explicitly.
    """
    pin = tomllib.loads((root / "rust-toolchain.toml").read_text(encoding="utf-8"))
    channel = pin.get("toolchain", {}).get("channel")
    if not channel:
        raise ValueError("rust-toolchain.toml must pin a toolchain channel")
    return channel


def main():
    root = Path(__file__).resolve().parents[1]
    toolchain = dict(os.environ, RUSTUP_TOOLCHAIN=pinned_toolchain(root))
    manifest = tomllib.loads((root / "Cargo.toml").read_text(encoding="utf-8"))
    stem = f"dotenvage-{manifest['package']['version']}"
    tests = subprocess.check_output(
        ["git", "ls-files", "tests/*.rs"], cwd=root, text=True
    ).splitlines()
    if not tests:
        raise ValueError("The published crate must contain integration tests")
    examples = subprocess.check_output(
        ["git", "ls-files", "examples/*.rs"], cwd=root, text=True
    ).splitlines()
    with tempfile.TemporaryDirectory(prefix="dotenvage-package-") as directory:
        target = Path(directory)
        subprocess.run(
            ["cargo", "package", "--locked", "--package", "dotenvage", "--no-verify",
             "--target-dir", str(target)], cwd=root, env=toolchain, check=True,
        )
        with tarfile.open(target / "package" / f"{stem}.crate") as archive:
            members = set(archive.getnames())
            for name in [*tests, *examples, "LICENSE-MIT", "LICENSE-APACHE"]:
                if f"{stem}/{name}" not in members:
                    raise ValueError(f"Published archive omits {name}")
            for name in ["build.rs", ".cargo/config.toml"]:
                if f"{stem}/{name}" in members:
                    raise ValueError(f"Published archive includes repository tooling: {name}")
            archive.extractall(target / "unpacked", filter="data")
        environment = dict(toolchain, RUST_TEST_THREADS="1")
        for selection in [["--lib", "--bins", "--tests"], ["--doc"]]:
            subprocess.run(
                ["cargo", "test", "--locked", *selection,
                 "--target-dir", str(root / "target/package-tests")],
                cwd=target / "unpacked" / stem, env=environment, check=True,
            )


if __name__ == "__main__":
    main()
