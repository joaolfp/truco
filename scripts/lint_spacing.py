#!/usr/bin/env python3
"""Spacing lint for Rust sources: keeps blocks from being glued to the code
around them. rustfmt does not enforce blank lines between statements, so this
fills the gap.

Rules (only lines at the same indentation are compared):
  1. A block statement (`for`, `while`, `loop`, `if`, `match`, `unsafe`) needs
     a blank line before it, unless it opens its enclosing block.
  2. Code after a closing `}` / `};` needs a blank line, unless it closes the
     enclosing block too, continues the expression (`else`, `.method()`, ...)
     or is the next arm of a `match`.

Usage:
  python3 scripts/lint_spacing.py          # check, exit 1 on violations
  python3 scripts/lint_spacing.py --fix    # insert the missing blank lines
"""

import re
import sys
from pathlib import Path

ROOTS = ["src", "tests", "build.rs"]

BLOCK_START = re.compile(r"(for|while|loop|if|match|unsafe)\b")
MATCH_OPEN = re.compile(r"\bmatch\b.*\{$")
# Previous line endings after which a block statement is not a new statement.
OPENERS = ("{", "(", "[", ",", "=", "=>", "|", "&&", "||")
# Line starts that continue the expression or close the enclosing block.
CONTINUATIONS = ("}", ")", "]", ".", "?", "else", ",", "=>", "&&", "||")


def indent(line: str) -> str:
	return line[: len(line) - len(line.lstrip())]


def is_comment_or_attr(text: str) -> bool:
	return text.startswith(("//", "#[", "#!["))


def needs_blank_before(lines: list[str], i: int) -> bool:
	"""Rule 1: is line `i` a block statement glued to the previous one?"""
	text = lines[i].strip()
	if not BLOCK_START.match(text):
		return False

	# Comments and attributes belong to the statement below them.
	j = i - 1
	while j >= 0 and lines[j].strip() and is_comment_or_attr(lines[j].strip()):
		if indent(lines[j]) != indent(lines[i]):
			return False
		j -= 1

	if j < 0 or not lines[j].strip():
		return False

	prev = lines[j]
	prev_text = prev.strip()
	if indent(prev) != indent(lines[i]):
		return False

	if prev_text.endswith(OPENERS) or prev_text in ("}", "};"):
		# `}` is handled by rule 2 (reported once, at the closing brace).
		return False

	return True


def needs_blank_after(lines: list[str], i: int) -> bool:
	"""Rule 2: is the code after the `}` on line `i` glued to it?"""
	if lines[i].strip() not in ("}", "};"):
		return False

	if i + 1 >= len(lines):
		return False

	nxt = lines[i + 1]
	nxt_text = nxt.strip()
	if not nxt_text or indent(nxt) != indent(lines[i]):
		return False

	if nxt_text.startswith(CONTINUATIONS) or "=>" in nxt_text:
		return False

	# Inside a `match`, the next line starts another arm (possibly a
	# multi-line pattern), so it stays attached to the previous one.
	return not inside_match(lines, i)


def inside_match(lines: list[str], i: int) -> bool:
	"""Whether line `i` sits directly inside the arms of a `match`."""
	own = len(indent(lines[i]))
	for j in range(i - 1, -1, -1):
		text = lines[j].strip()
		if text and len(indent(lines[j])) < own:
			return bool(MATCH_OPEN.search(text))

	return False


def blank_line_positions(lines: list[str]) -> list[int]:
	"""Indices before which a blank line must be inserted."""
	positions = set()
	for i in range(len(lines)):
		if needs_blank_after(lines, i):
			positions.add(i + 1)

		if needs_blank_before(lines, i):
			j = i
			while j > 0 and is_comment_or_attr(lines[j - 1].strip()):
				j -= 1

			positions.add(j)

	return sorted(positions)


def rust_files() -> list[Path]:
	files = []
	for root in ROOTS:
		path = Path(root)
		if path.is_file():
			files.append(path)
		elif path.is_dir():
			files.extend(sorted(path.rglob("*.rs")))

	return files


def main() -> int:
	fix = "--fix" in sys.argv[1:]
	violations = 0
	for path in rust_files():
		lines = path.read_text().split("\n")
		positions = blank_line_positions(lines)
		if not positions:
			continue

		violations += len(positions)
		if fix:
			for pos in reversed(positions):
				lines.insert(pos, "")

			path.write_text("\n".join(lines))
		else:
			for pos in positions:
				print(f"{path}:{pos + 1}: missing blank line before `{lines[pos].strip()}`")

	if violations and not fix:
		print(f"\n{violations} spacing issue(s). Run `mise run lint-spacing-fix` to fix them.")
		return 1

	if fix and violations:
		print(f"Inserted {violations} blank line(s).")

	return 0


if __name__ == "__main__":
	sys.exit(main())
