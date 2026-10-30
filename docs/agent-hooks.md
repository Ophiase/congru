# Agents: rules, hooks and the gate

What each tool reads, what it can run by itself, and the one thing that works
everywhere.

| Tool | Rules it reads | Lifecycle hooks | Adapter here |
| --- | --- | --- | --- |
| Claude Code | `AGENTS.md` through `.claude/CLAUDE.md` | `settings.json` on edit, stop and write | `.claude/hooks/` |
| Cline, VS Code | `AGENTS.md`, `.clinerules/`, `.cline/rules/` | None: hooks are SDK plugins, and plugins apply to the CLI, SDK and Kanban, not the extension | `.clinerules/agents.md` |
| Cline CLI, SDK, Kanban | plugin rules | `beforeRun`, `afterRun`, `beforeModel`, `afterModel`, `beforeTool`, `afterTool`, `onEvent`, in a TypeScript `AgentPlugin`; project plugins in `.cline/plugins/` | a plugin may run `just guards` |
| Kilo Code | `AGENTS.md`, `.kilocode/rules/` (legacy) or `.kilo/rules/` through `kilo.jsonc` | None documented; customize offers rules, instructions, workflows and skills | `.kilocode/rules/agents.md` |
| Any tool with git | — | A `pre-commit` hook | `.githooks/pre-commit` |

## The gate

`just agent gate` runs `just check` then `just guards`; `just agent ci` adds
the Python, C and licence checks when their tools are installed.
`just install-hooks` sets `core.hooksPath` to `.githooks`, so every commit
runs `just guards` locally: no event API, no paid runner.

## Why the hook is the floor, not the ceiling

A tool's event names change; the shell does not. The checks in
`.agents/checks/` are the source, and every adapter above only names them.

## Sources

- [Cline hooks](https://docs.cline.bot/customization/hooks): "See details
  under SDK Plugins"; plugins apply to the SDK, CLI and Kanban.
- [Cline SDK plugins](https://docs.cline.bot/sdk/plugins): the seven
  lifecycle hooks and the `AgentPlugin` shape.
- [Cline rules](https://docs.cline.bot/customization/cline-rules):
  `AGENTS.md`, `.clinerules/`, `.cline/rules/`.
- [Kilo agents.md](https://kilo.ai/docs/customize/agents-md): root
  `AGENTS.md`, per-directory files, `CLAUDE.md`.
- [Kilo custom rules](https://kilo.ai/docs/customize/custom-rules):
  `.kilocode/rules/` is legacy-compatible; `.kilo/rules/` through
  `kilo.jsonc`.
