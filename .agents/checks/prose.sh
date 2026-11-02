#!/usr/bin/env bash
# Usage: prose.sh <file>...
# Markdown is structured reasoning, not prose: reports a paragraph of more
# than four lines and any diagram drawn in a text code block (Mermaid instead).
set -uo pipefail

status=0
for file in "$@"; do
  [[ $file == *.md && -f $file ]] || continue
  awk -v file="$file" -v max=4 '
    function flush() {
      if (count > max) { printf "%s:%d: PARAGRAPH %d lines; split into bullets or a table\n", file, start, count; bad = 1 }
      count = 0
    }
    /^```/ {
      flush()
      if (!fenced && $0 ~ /^```text/) { printf "%s:%d: ASCII-DIAGRAM draw it in Mermaid or as a list\n", file, NR; bad = 1 }
      fenced = !fenced; next
    }
    fenced { next }
    /^[[:space:]]*$/ || /^[[:space:]]*([-*+|>#]|[0-9]+\.)/ || /^[[:space:]]{2,}/ { flush(); next }
    { if (count == 0) start = NR; count++ }
    END { flush(); exit bad }
  ' "$file" || status=1
done
exit $status
