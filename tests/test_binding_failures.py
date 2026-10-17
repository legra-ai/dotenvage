"""A missing native binding must fail each Node test entry point."""

from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).parents[1]


class BindingFailureTests(unittest.TestCase):
    def test_node_suites_fail_when_the_package_cannot_load(self):
        suites = sorted((ROOT / 'npm/__tests__').glob('*.test.js'))
        self.assertTrue(suites)
        for suite in suites:
            with self.subTest(suite=suite.name), tempfile.TemporaryDirectory() as directory:
                target = Path(directory) / '__tests__' / suite.name
                target.parent.mkdir()
                shutil.copyfile(suite, target)
                result = subprocess.run(['node', '--test', str(target)], cwd=directory,
                                        text=True, capture_output=True, check=False)
                self.assertNotEqual(result.returncode, 0, result.stdout + result.stderr)
                self.assertIn('Cannot find module', result.stdout + result.stderr)


if __name__ == '__main__':
    unittest.main()
