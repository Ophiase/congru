# Reference projects — how the author works

Read these before proposing a structure, a convention or a style. Their
paths are in `.agents/local.md` (gitignored; template:
[local.example.md](local.example.md)). Read only; take the **method**, never
the content.

**Absolute rule**: these projects are private. Never name them, quote them
or describe their content in a tracked file or a commit. Refer to them by
role only.

## Sibling reference

A Rust monorepo of the author, the most recent state of the conventions:
`ideas/` as the core, `.agents/portable/` copied from it, workspace lints,
`justfile` modules, hooks, the merge-master skill.

## Research reference

- `ideas/` first: short files, one notion each, a map, status tags.
- A report in **Typst**, figures drawn natively from data.
- Python to check by brute force, never to explain.

## Industrial Python reference

- `AGENTS.md` for every agent, `.claude/CLAUDE.md` importing it.
- Written architecture decisions, `justfile`, strict tooling (uv, ruff),
  changelog and version. The Python skin follows it.

## Founding case

A private report where the need first appeared. It may hold confidential
content: read locally only, take the need, never the content, and never
link this project to it in a tracked file.
