"""Reject Linux release artifacts that exceed their declared glibc ceiling."""

import argparse
from collections.abc import Iterable, Iterator
from pathlib import Path
import re
import subprocess


def require_glibc(lines: Iterable[str], ceiling: tuple[int, ...]) -> tuple[int, ...]:
    """Stream symbol versions and enforce a numeric ABI ceiling."""
    highest = None
    for line in lines:
        for match in re.finditer(r'\bGLIBC_(\d+\.\d+(?:\.\d+)?)\b', line):
            version = tuple(map(int, match.group(1).split('.')))
            if highest is None or version > highest:
                highest = version
    if highest is None:
        raise ValueError('No GLIBC symbol requirements found')
    if highest + (0,) * (3 - len(highest)) > ceiling + (0,) * (3 - len(ceiling)):
        detected = '.'.join(map(str, highest))
        allowed = '.'.join(map(str, ceiling))
        raise ValueError(f'Required GLIBC {detected} exceeds declared ceiling {allowed}')
    return highest


def symbol_lines(binary: Path) -> Iterator[str]:
    """Stream objdump output and propagate inspection failures."""
    with subprocess.Popen(['objdump', '-T', str(binary)], stdout=subprocess.PIPE,
                          text=True) as process:
        yield from process.stdout
        result = process.wait()
        if result != 0:
            raise subprocess.CalledProcessError(result, process.args)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--maximum', required=True)
    parser.add_argument('binaries', nargs='+', type=Path)
    arguments = parser.parse_args()
    if not re.fullmatch(r'\d+\.\d+(?:\.\d+)?', arguments.maximum):
        parser.error('--maximum must be a numeric glibc version')
    ceiling = tuple(map(int, arguments.maximum.split('.')))
    for binary in arguments.binaries:
        highest = require_glibc(symbol_lines(binary), ceiling)
        print(f'{binary}: GLIBC {".".join(map(str, highest))} <= {arguments.maximum}')


if __name__ == '__main__':
    main()
