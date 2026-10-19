#!/usr/bin/env python3
"""Translate GNU driver arguments into Zig's supported command line."""

import os
import sys


def translate(arguments: list[str]) -> list[str]:
    """Remove GNU ld's optimization flag, which Zig ignores and diagnoses.

    Rust adds this flag to optimized GNU targets. Zig has no corresponding
    option; LLVM code generation and LTO optimization arguments are preserved.
    See ziglang/zig src/main.zig's linker_optimization handling.
    """
    if arguments and arguments[0] in ('cc', 'c++'):
        return [argument for argument in arguments if argument != '-Wl,-O1']
    return arguments


if __name__ == '__main__':
    os.execvp('zig', ['zig', *translate(sys.argv[1:])])
