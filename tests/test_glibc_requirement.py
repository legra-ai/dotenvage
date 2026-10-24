"""Require the published Linux ABI ceiling, rather than just printing it."""

import importlib.util
from pathlib import Path
import unittest

ROOT = Path(__file__).parents[1]


def verifier():
    spec = importlib.util.spec_from_file_location('verify_glibc', ROOT / 'scripts/verify-glibc.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class GlibcRequirementTests(unittest.TestCase):
    def test_rejects_symbols_newer_than_declared_ceiling(self):
        with self.assertRaisesRegex(ValueError, '2.34.*2.28'):
            verifier().require_glibc(iter(['symbol GLIBC_2.17', 'symbol GLIBC_2.34']), (2, 28))

    def test_compares_versions_numerically(self):
        self.assertEqual(verifier().require_glibc(iter(['symbol GLIBC_2.9', 'symbol GLIBC_2.28']),
                                                 (2, 28)), (2, 28))

    def test_rejects_patch_level_above_ceiling(self):
        with self.assertRaisesRegex(ValueError, '2.28.1'):
            verifier().require_glibc(iter(['symbol GLIBC_2.28.1']), (2, 28))

    def test_rejects_missing_glibc_symbols(self):
        with self.assertRaisesRegex(ValueError, 'No GLIBC'):
            verifier().require_glibc(iter(['invalid or unexpected binary']), (2, 28))

    def test_checks_every_symbol_version_on_a_line(self):
        with self.assertRaisesRegex(ValueError, '2.29'):
            verifier().require_glibc(iter(['GLIBC_2.2.5 GLIBC_2.29']), (2, 28))

    def test_workflow_gates_both_cli_and_napi(self):
        workflow = (ROOT / '.github/workflows/ci.yml').read_text()
        step = workflow.split('      - name: Verify glibc version requirement\n', 1)[1].split('\n      - ', 1)[0]
        self.assertIn('python3 release-controls/scripts/verify-glibc.py --maximum 2.28', step)
        self.assertIn('target/${{ matrix.platform.target }}/release/dotenvage', step)
        self.assertIn('npm/*.node', step)
        self.assertIn('python3 -m unittest discover -s release-controls/tests -p test_glibc_requirement.py', workflow)


if __name__ == '__main__':
    unittest.main()
