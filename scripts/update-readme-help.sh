#!/usr/bin/env bash
# Rewrites the `tataki --help` transcript in README.md from the built binary.
# Run it after changing anything clap prints, and commit the result; the CI
# checks that the committed README matches what this script produces.
set -euo pipefail

cd "$(dirname "$0")/.."

help_file=$(mktemp)
trap 'rm -f "$help_file"' EXIT

cargo run --quiet -- --help > "$help_file"

HELP_FILE="$help_file" python3 - <<'PY'
import io
import os
import re

# Both ends are stripped because Cargo.toml carries no `description`, so
# clap renders an empty `about` and the help opens with a blank line.
help_text = io.open(os.environ["HELP_FILE"], encoding="utf-8").read().strip("\n")

path = "README.md"
readme = io.open(path, encoding="utf-8").read()

block = (
    "<!-- BEGIN help -->\n"
    "```shell\n"
    "$ tataki --help\n"
    f"{help_text}\n"
    "```\n"
    "<!-- END help -->"
)

# The replacement is a function because re.sub reads backslashes in a string
# replacement as group references, and the help text is not escaped for that.
readme, count = re.subn(
    r"<!-- BEGIN help -->.*?<!-- END help -->",
    lambda _match: block,
    readme,
    flags=re.DOTALL,
)
if count != 1:
    raise SystemExit(f"expected exactly one marker pair in {path}, found {count}")

io.open(path, "w", encoding="utf-8").write(readme)
PY
