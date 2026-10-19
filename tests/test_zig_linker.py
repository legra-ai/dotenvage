"""Exercise argument translation at the Rust-to-Zig boundary."""

import importlib.util
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).parents[1]
ADAPTER = ROOT / 'scripts/zig-cc.py'


class ZigLinkerTests(unittest.TestCase):
    def translate(self, arguments):
        spec = importlib.util.spec_from_file_location('zig_cc', ADAPTER)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        return module.translate(arguments)

    def test_removes_only_unsupported_gnu_linker_optimization(self):
        self.assertEqual(self.translate(['cc', '-O3', '-Wl,-O1', '-Wl,--gc-sections', 'main.o']),
                         ['cc', '-O3', '-Wl,--gc-sections', 'main.o'])

    def test_preserves_other_commands_and_arguments(self):
        for arguments in [['version'], ['ar', 'rcs', 'lib.a', 'main.o'],
                          ['cc', '-Wl,--fatal-warnings', '-Wl,-O2', '-O1']]:
            self.assertEqual(self.translate(arguments), arguments)

    def test_all_linux_builds_use_the_same_adapter(self):
        workflow = (ROOT / '.github/workflows/ci.yml').read_text()
        self.assertIn('CARGO_ZIGBUILD_ZIG_PATH: ${{ github.workspace }}/scripts/zig-cc.py', workflow)
        self.assertIn('CARGO_ZIGBUILD_ZIG_COMMAND: ${{ github.workspace }}/scripts/zig-cc.py', workflow)
        self.assertIn('exec "$GITHUB_WORKSPACE/scripts/zig-cc.py" cc', workflow)
        self.assertIn('python3 -m unittest discover -s tests -p test_zig_linker.py', workflow)

    @unittest.skipUnless(shutil.which('zig'), 'Zig is installed in Linux artifact jobs')
    def test_real_compiler_errors_still_fail(self):
        result = subprocess.run([str(ADAPTER), 'cc', '-target', 'invalid-target'],
                                text=True, capture_output=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('error', result.stderr.lower())

    @unittest.skipUnless(shutil.which('zig'), 'Zig is installed in Linux artifact jobs')
    def test_real_release_link_has_no_diagnostics(self):
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory) / 'main.c'
            source.write_text('int main(void) { return 0; }\n')
            result = subprocess.run([str(ADAPTER), 'cc', '-target', 'aarch64-linux-gnu.2.28',
                                     '-Wl,-O1', str(source), '-o', str(Path(directory) / 'main')],
                                    text=True, capture_output=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(result.stderr, '')


if __name__ == '__main__':
    unittest.main()
