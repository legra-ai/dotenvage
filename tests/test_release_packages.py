"""Exercise partial publication recovery and reject mismatched packages."""

import base64
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import tempfile
import zipfile
import unittest
from unittest.mock import patch
from urllib.error import HTTPError

SPEC = importlib.util.spec_from_file_location(
    "release_packages", Path(__file__).parents[1] / "scripts/release-packages.py"
)
release = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(release)


class RegistryTests(unittest.TestCase):
    def setUp(self):
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        self.package = Path(directory.name) / "package.tgz"
        self.package.write_bytes(b"validated package")

    def test_npm_duplicate_requires_identical_archive(self):
        integrity = "sha512-" + base64.b64encode(
            hashlib.sha512(self.package.read_bytes()).digest()
        ).decode()
        with patch.object(release, "registry_json", return_value={"dist": {"integrity": integrity}}):
            self.assertTrue(release.npm_exists("1.2.3", self.package))
        with patch.object(release, "registry_json", return_value={"dist": {"integrity": "sha512-wrong"}}):
            with self.assertRaisesRegex(ValueError, "npm.*identity"):
                release.npm_exists("1.2.3", self.package)

    def test_missing_npm_package_needs_publication(self):
        with patch.object(release, "registry_json", return_value=None):
            self.assertFalse(release.npm_exists("1.2.3", self.package))

    def test_registry_only_treats_404_as_absent(self):
        for code in [401, 403, 429, 500]:
            with self.subTest(code=code), patch.object(
                release, "urlopen", side_effect=HTTPError("https://registry", code, "error", {}, None)
            ):
                with self.assertRaises(HTTPError):
                    release.registry_json("https://registry")

    def test_partial_python_release_returns_only_missing_wheels(self):
        wheel = self.package.with_name("dotenvage-1.2.3-platform.whl")
        wheel.write_bytes(b"wheel")
        other = self.package.with_name("dotenvage-1.2.3-other.whl")
        other.write_bytes(b"other wheel")
        published = {"urls": [{"filename": wheel.name, "digests": {
            "sha256": hashlib.sha256(wheel.read_bytes()).hexdigest()
        }}]}
        with patch.object(release, "registry_json", return_value=published):
            self.assertEqual(release.missing_wheels("1.2.3", [wheel, other]), [other])
            wheel.write_bytes(b"different source")
            with self.assertRaisesRegex(ValueError, "PyPI.*identity"):
                release.missing_wheels("1.2.3", [wheel, other])

    def test_bundle_rejects_wrong_revision(self):
        bundle = self.package.with_name("release-packages.zip")
        release.write_bundle(bundle, "1.2.3", "a" * 40, [self.package])
        with self.assertRaisesRegex(ValueError, "revision"):
            release.read_bundle(bundle, "1.2.3", "b" * 40, self.package.parent / "wrong")
        output = self.package.parent / "restored"
        release.read_bundle(bundle, "1.2.3", "a" * 40, output)
        self.assertEqual((output / self.package.name).read_bytes(), self.package.read_bytes())


    def test_bundle_rejects_changed_archive_bytes(self):
        bundle = self.package.with_name("release-packages.zip")
        release.write_bundle(bundle, "1.2.3", "a" * 40, [self.package])
        with zipfile.ZipFile(bundle) as archive:
            manifest = archive.read("manifest.json")
        with zipfile.ZipFile(bundle, "w") as archive:
            archive.writestr("manifest.json", manifest)
            archive.writestr(self.package.name, b"unrelated build")
        with self.assertRaisesRegex(ValueError, "identity mismatch"):
            release.read_bundle(bundle, "1.2.3", "a" * 40, self.package.parent / "invalid")

    def test_retry_uses_persisted_archives_without_repacking(self):
        bundle = self.package.with_name("release-packages.zip")
        release.write_bundle(bundle, "1.2.3", "a" * 40, [self.package])
        self.package.write_bytes(b"new rebuild with different bytes")
        original = Path.cwd()
        try:
            os.chdir(self.package.parent)
            with patch.object(release.subprocess, "check_output", return_value=json.dumps({
                "assets": [{"name": bundle.name}]
            })) as query, patch.object(release.subprocess, "run") as command:
                release.stage("1.2.3", "a" * 40)
                query.assert_called_once()
                command.assert_called_once_with(
                    ["gh", "release", "download", "v1.2.3", "--pattern", bundle.name], check=True
                )
                self.assertEqual(Path("release-packages/package.tgz").read_bytes(), b"validated package")
        finally:
            os.chdir(original)


if __name__ == "__main__":
    unittest.main()
