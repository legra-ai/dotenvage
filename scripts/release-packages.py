"""Persist original release archives and verify registry identity on every retry."""

import base64
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
from urllib.error import HTTPError
from urllib.request import urlopen
import zipfile


def digest(path, algorithm="sha256"):
    with path.open("rb") as source:
        return hashlib.file_digest(source, algorithm).digest()


def registry_json(url):
    try:
        with urlopen(url, timeout=60) as response:
            return json.load(response)
    except HTTPError as error:
        if error.code == 404:
            return None
        raise


def npm_exists(version, archive):
    metadata = registry_json(f"https://registry.npmjs.org/@dotenvage%2fnode/{version}")
    if metadata is None:
        return False
    expected = "sha512-" + base64.b64encode(digest(archive, "sha512")).decode()
    if metadata["dist"]["integrity"] != expected:
        raise ValueError("npm package identity differs from the original release archive")
    return True


def missing_wheels(version, wheels):
    metadata = registry_json(f"https://pypi.org/pypi/dotenvage/{version}/json")
    published = {entry["filename"]: entry["digests"]["sha256"]
                 for entry in metadata["urls"]} if metadata else {}
    missing = []
    for wheel in wheels:
        if wheel.name not in published:
            missing.append(wheel)
        elif published[wheel.name] != digest(wheel).hex():
            raise ValueError(f"PyPI wheel identity differs: {wheel.name}")
    return missing


def write_bundle(bundle, version, revision, packages):
    manifest = {"version": version, "revision": revision, "files": {
        package.name: digest(package).hex() for package in packages
    }}
    if len(manifest["files"]) != len(packages):
        raise ValueError("Duplicate package filenames")
    with zipfile.ZipFile(bundle, "x", compression=zipfile.ZIP_STORED) as archive:
        archive.writestr("manifest.json", json.dumps(manifest, sort_keys=True))
        for package in packages:
            archive.write(package, package.name)


def read_bundle(bundle, version, revision, output):
    with zipfile.ZipFile(bundle) as archive:
        manifest = json.loads(archive.read("manifest.json"))
        if (manifest["version"], manifest["revision"]) != (version, revision):
            raise ValueError("Release bundle version/revision mismatch")
        names = archive.namelist()
        if len(names) != len(set(names)) or set(names) != {"manifest.json", *manifest["files"]}:
            raise ValueError("Release bundle contains unexpected files")
        output.mkdir()
        for name, expected in manifest["files"].items():
            if Path(name).name != name or name in (".", ".."):
                raise ValueError("Invalid release package filename")
            target = output / name
            with archive.open(name) as source, target.open("xb") as destination:
                shutil.copyfileobj(source, destination)
            if digest(target).hex() != expected:
                raise ValueError(f"Release bundle package identity mismatch: {name}")


def stage(version, revision):
    """One immutable asset survives lost runners and nondeterministic rebuilds."""
    tag = f"v{version}"
    bundle = Path("release-packages.zip")
    assets = json.loads(subprocess.check_output(
        ["gh", "release", "view", tag, "--json", "assets"], text=True
    ))["assets"]
    if not any(asset["name"] == bundle.name for asset in assets):
        result = json.loads(subprocess.check_output(
            ["npm", "pack", "--json", "--ignore-scripts"], cwd="npm", text=True
        ))
        if len(result) != 1:
            raise ValueError("Expected one npm archive")
        wheels = sorted(Path("python-wheels").glob("*.whl"))
        if len(wheels) != 5:
            raise ValueError("Expected exactly five Python platform wheels")
        write_bundle(bundle, version, revision, [Path("npm") / result[0]["filename"], *wheels])
        # No --clobber: an existing release asset must never change identity.
        subprocess.run(["gh", "release", "upload", tag, str(bundle)], check=True)
    else:
        subprocess.run(["gh", "release", "download", tag, "--pattern", bundle.name], check=True)
    read_bundle(bundle, version, revision, Path("release-packages"))


def output(name, value):
    with open(os.environ["GITHUB_OUTPUT"], "a") as stream:
        stream.write(f"{name}={value}\n")


def main():
    command = sys.argv[1]
    version = os.environ["RELEASE_VERSION"]
    if command == "stage":
        stage(version, os.environ["RELEASE_REVISION"])
        return
    directory = Path("release-packages")
    if command == "check-npm":
        archives = list(directory.glob("*.tgz"))
        if len(archives) != 1:
            raise ValueError("Expected exactly one npm archive")
        output("npm_missing", str(not npm_exists(version, archives[0])).lower())
        output("npm_archive", str(archives[0]))
    elif command in ("filter-python", "verify-python"):
        wheels = sorted(directory.glob("*.whl"))
        if len(wheels) != 5:
            raise ValueError("Expected exactly five Python platform wheels")
        missing = missing_wheels(version, wheels)
        if command == "verify-python":
            if missing:
                raise ValueError(f"PyPI release is incomplete: {[wheel.name for wheel in missing]}")
            return
        pending = Path("pending-wheels")
        pending.mkdir()
        for wheel in missing:
            shutil.copyfile(wheel, pending / wheel.name)
        output("python_missing", str(bool(missing)).lower())
    else:
        raise ValueError(f"Unknown release operation: {command}")


if __name__ == "__main__":
    main()
