#!/bin/zsh
set -eu
TASK_DIR="${0:A:h}"
if (( $# == 0 )); then
  INPUT_FILE=$(/usr/bin/osascript -e 'POSIX path of (choose file with prompt "Choose a Markdown file")') || exit 0
  set -- "$INPUT_FILE"
fi
THEME=$(/usr/bin/osascript -e 'set picked to choose from list {"Editorial", "Classic"} with prompt "Choose a PDF theme" default items {"Editorial"}' -e 'if picked is false then return ""' -e 'return item 1 of picked') || exit 0
[[ -n "$THEME" ]] || exit 0
for INPUT_FILE in "$@"; do
  "$TASK_DIR/mdpdf" "$INPUT_FILE" --theme "${THEME:l}" --open
done
