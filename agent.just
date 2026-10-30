set positional-arguments := true

root := justfile_directory()

[group("gate")]
[doc("Everything that must pass before declaring work done: check, then guards.")]
gate:
    just --justfile "{{root}}/justfile" --working-directory "{{root}}" check
    just --justfile "{{root}}/justfile" --working-directory "{{root}}" guards

[group("gate")]
[doc("The portable checks over the files the working tree changed.")]
guards:
    just --justfile "{{root}}/justfile" --working-directory "{{root}}" guards

[group("gate")]
[doc("The full local gate: check, guards, Python, C and licences; a missing tool is skipped.")]
[script("bash")]
ci:
    set -euo pipefail
    just --justfile "{{root}}/justfile" --working-directory "{{root}}" check
    just --justfile "{{root}}/justfile" --working-directory "{{root}}" guards
    if command -v uv >/dev/null; then
        just --justfile "{{root}}/justfile" --working-directory "{{root}}" python-test
    else
        echo "python-test skipped: no uv"
    fi
    if command -v g++ >/dev/null; then
        just --justfile "{{root}}/justfile" --working-directory "{{root}}" capi-example
    else
        echo "capi-example skipped: no g++"
    fi
    if command -v cargo-deny >/dev/null; then
        just --justfile "{{root}}/justfile" --working-directory "{{root}}" deny
    else
        echo "deny skipped: no cargo-deny"
    fi

[group("gate")]
[doc("Branch, last commit and working tree: what a review reads first.")]
status:
    @git branch --show-current
    @git log --oneline -1
    @git status --short

[group("gate")]
[doc("What changed since main, then the working tree.")]
[script("bash")]
brief:
    set -euo pipefail
    git --no-pager diff main...HEAD --stat
    git status --short

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
