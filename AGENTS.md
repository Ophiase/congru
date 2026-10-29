# AGENTS.md

Instructions for any coding agent in this repository. Claude Code reads them
through `.claude/CLAUDE.md`.

## What this project is

A rigid registration brick: align a point cloud or a few marks onto a point
cloud or a mesh, robust to noise, outliers and partial views, in Rust, called
from Python, C and C++. Everything starts in [ideas/](ideas/README.md).

## Language

English everywhere: code, comments, markdown, commits. The author may write
to you in French; answer in French, write files in English.

## Order of work

**Ideas before code.** A choice that is not in `ideas/` is not made yet:

1. Read [ideas/README.md](ideas/README.md) and [ideas/open.md](ideas/open.md).
2. A new concept, decision or doubt goes into `ideas/` first, one short file
   with its status tag, and the map stays current.
3. An open question waits for the author; do not settle it in code.
4. Code follows, then `just check` and `just guards`.

Read [.agents/references.md](.agents/references.md) and, when useful, the
reference projects whose paths are in `.agents/local.md` (gitignored). If it
is missing, ask for the paths and offer to write it from
[.agents/local.example.md](.agents/local.example.md).

## Ideas

- **One concept per file**, short. Add a file rather than grow one.
- Every claim carries its tag: **[defined]**, **[sketch]**, **[guess]**,
  **[to check]**, **[question]**, **[caution]**.
- A reference to the literature is **[to check]** until read again.

## Layout and languages

One Cargo workspace; `crates/` holds the core, the synthetic generator and
the skins ([layers](ideas/architecture/layers.md)). `just check` passes
before work is declared done.

Rust follows `.agents/portable/rules/rust.md`; Python follows
`.agents/portable/rules/python.md`. Folders of code follow
`.agents/rules/layout.md`: one concept each, seven entries at most, two
hundred lines per file. Read `.agents/portable/rules/comments.md` and
`.agents/rules/rust-docs.md` before writing code.

## Rules and checks

`.agents/portable/` is copied as is from the author's other repositories and
stays project-agnostic (`.agents/checks/portable.sh` guards it). Project
rules and checks go to `.agents/rules/` and `.agents/checks/`.
`.agents/checks/run.sh <check> <file>...` runs one; `just guards` runs them
all over the changed files.

## Tests

A test checks an algorithmic property against the truth of the synthetic
generator: a known motion recovered within a bound derived from the noise.
Never a number copied from a run.

## Done

A topic is done when the [review](.claude/skills/review/SKILL.md) skill
returns **ready**: the publication first, the code second.

## Branches

Work on `dev/<topic>`, never on `main`. When a topic is done and every check
passes, the [merge-master](.claude/skills/merge-master/SKILL.md) skill
merges it with `--no-ff` after a small version bump. Nothing is committed or
pushed without being asked.

## Downloads

Test scans only: single data files (`.ply`, `.obj`, `.xyz`) by direct URL,
into a gitignored `data/<source>/` with `SHA256SUMS` and a `SOURCE.md`
recording origin and licence. Never clone, never fetch an archive of
scripts, never execute what was downloaded.

## Never

- A network call the user did not ask for, in the library or its tools.
- Medical data, or the name of a company, client or professional project, in
  a tracked file or a commit.
- Anything from the projects of `.agents/local.md` but their principles: no
  name, no quotation, no code copied.
- Commit or push without being asked.
