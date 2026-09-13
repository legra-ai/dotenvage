"""Build and test the actual crates.io archive outside the Git checkout."""

import os
from pathlib import Path
import subprocess
import tarfile
import tempfile
import tomllib


def main():
    root = Path(__file__).resolve().parents[1]
    manifest = tomllib.loads((root / "Cargo.toml").read_text())
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
             "--target-dir", str(target)], cwd=root, check=True,
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
        environment = dict(os.environ, RUST_TEST_THREADS="1")
        for selection in [["--lib", "--bins", "--tests"], ["--doc"]]:
            subprocess.run(
                ["cargo", "test", "--locked", *selection,
                 "--target-dir", str(root / "target/package-tests")],
                cwd=target / "unpacked" / stem, env=environment, check=True,
            )


if __name__ == "__main__":
    main()
