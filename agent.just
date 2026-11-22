set positional-arguments := true

import ".agents/portable/just/harness.just"

[group("gate")]
[doc("The full local gate: check, guards, Python, C and licences; a missing tool is skipped.")]
[script("bash")]
ci:
    set -euo pipefail
    just check
    just guards
    if command -v uv >/dev/null; then just python-test; else echo "python-test skipped: no uv"; fi
    if command -v g++ >/dev/null; then just capi-example; else echo "capi-example skipped: no g++"; fi
    if command -v cargo-deny >/dev/null; then just deny; else echo "deny skipped: no cargo-deny"; fi

[group("code")]
[doc("The longest files of code, tests left out: green to 100 lines, yellow to 200, red past")]
[script("bash")]
longest count="20":
    set -euo pipefail
    git ls-files --cached --others --exclude-standard -- '*.rs' | while read -r file; do
        awk '/^#\[cfg\(test\)\]/ { exit } { lines++ } END { print lines + 0 }' "$file" | sed "s|$| $file|"
    done | sort -rn | head -n "$1" | while read -r lines file; do
        if ((lines > 200)); then colour=31; elif ((lines > 100)); then colour=33; else colour=32; fi
        printf '\e[%sm%5d\e[0m  %s\n' "$colour" "$lines" "$file"
    done
