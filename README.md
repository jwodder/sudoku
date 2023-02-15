[![Project Status: Concept – Minimal or no implementation has been done yet, or the repository is only intended to be a limited example, demo, or proof-of-concept.](https://www.repostatus.org/badges/latest/concept.svg)](https://www.repostatus.org/#concept)
[![CI Status](https://github.com/jwodder/sudoku/actions/workflows/test.yml/badge.svg)](https://github.com/jwodder/sudoku/actions/workflows/test.yml)
[![codecov.io](https://codecov.io/gh/jwodder/sudoku/branch/master/graph/badge.svg)](https://codecov.io/gh/jwodder/sudoku)
[![MIT License](https://img.shields.io/github/license/jwodder/sudoku.svg)](https://opensource.org/licenses/MIT)

This is a Rust library and command-line program for solving a classic 9×9
Sudoku puzzle using a basic backtracking algorithm.

Usage
=====

    sudoku [--pretty|-P] [<infile>]

Solve the Sudoku puzzle contained in the given file, or passed on standard
input if no filename is given.  The puzzle must be in the form of nine lines of
nine values each, where "unfilled" cells are represented by either `0` or any
non-digit, non-space character; horizontal whitespace and blank lines are
ignored.

By default, the solution is output as nine lines of nine numbers with no
horizontal whitespace or other styling, but supplying the `--pretty` or `-P`
option will cause the solution to be rendered with borders around the grid &
between regions and with spaces between adjacent values.

If the given puzzle has multiple solutions, one solution will be output, but
which one is unspecified.  If the puzzle does not have a solution, "`No
solution`" will be printed to standard error, and the program will exit
nonzero.
