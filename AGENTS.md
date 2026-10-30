# AGENTS.md

Instructions for any coding agent in this repository. Claude Code reads them
through `.claude/CLAUDE.md`.

Harmostes, the agent who works here, leaves a word to visiting agents at the
end of the README.

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
reference projects whose paths are in `.agents/local/projects.yaml`
(gitignored). If it is missing, ask for the paths and offer to write it from
[.agents/local.example.yaml](.agents/local.example.yaml).

## Good practices

The project's own order of what to copy first is in
[.agents/masterclass-bootstrap.md](.agents/masterclass-bootstrap.md); the
portable rules stay in `.agents/portable/`. Before reusing another project,
read its `.agents/guard.yaml` and then `.agents/local/guard.yaml`
([guards](.agents/portable/rules/guards.md)).

## Ideas

- **One concept per file**, short. Add a file rather than grow one.
- Every claim carries its tag: **[defined]**, **[sketch]**, **[guess]**,
  **[to check]**, **[question]**, **[caution]**.
- A reference to the literature is **[to check]** until read again.
- A known method, algorithm or concept is a link, to Wikipedia or to the
  paper (its DOI or arXiv page), at its first mention in a file: the reader
  delegates its understanding to the source instead of to our paraphrase.
  Reference-style links at the end of the section keep tables readable.

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

## Any agent, not only Claude Code

The harness is the recipes and the checks, never one tool's events.
`just agent gate` runs `just check` then `just guards`; `.agents/checks/run.sh
<check> <file>...` runs one check directly. The root justfile has a module
`agent`; `agent.just` imports the portable harness from
`.agents/portable/just/harness.just`, which calls the project's `check` and
`guards`. An agent that can run a shell runs the same gate.

Cline, Cursor, Kilo Code and Copilot read this file. Their own rule folders
(`.clinerules/`, `.cursor/rules/`, `.kilocode/rules/`, `.github/`) hold one
pointer each, never a copy. Cline runs `.clinerules/hooks/<Event>` and Cursor
runs `.cursor/hooks.json`; Kilo documents no lifecycle hook but takes
permissions from `.kilo/kilo.jsonc`. So the adapters are the Cline and Cursor
hooks, the existing Claude Code hooks, a Kilo permission file, and a git
hook: `just install-hooks` once, then every commit runs `just guards`.

`.claude/` is one adapter. Its `settings.json` permissions and its `hooks/`
fire only in Claude Code; elsewhere they are inert. `.claude/CLAUDE.md`
imports this file with `@`, a Claude Code convention; for every other tool,
this file is the source. The portable contract is in
[.agents/portable/rules/agent-harness.md](.agents/portable/rules/agent-harness.md);
the project matrix is in [docs/agent-hooks.md](docs/agent-hooks.md).

## Tests

A test checks an algorithmic property against the truth of the synthetic
generator: a known motion recovered within a bound derived from the noise.
Never a number copied from a run.

## Done

A topic is done when the [review](.claude/skills/review/SKILL.md) skill
returns **ready**: the publication first, the code second.

## Continuous integration

Workflows live in `.github/workflows/` and run **only when started by
hand** (`workflow_dispatch`): nothing runs, and nothing is paid for, on a
push or a pull request. The same `just` recipes run on a laptop.

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
- Anything from the projects of `.agents/local/projects.yaml` but their
  principles: no name, no quotation, no code copied.
- Reuse another project beyond what its `.agents/guard.yaml` allows.
- Commit or push without being asked.
