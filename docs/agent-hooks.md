# Agents: rules, hooks and the gate

What each tool reads, what it can run by itself, and the one thing that works
everywhere.

| Tool | Rules it reads | Lifecycle hooks | Adapter here |
| --- | --- | --- | --- |
| Claude Code | `AGENTS.md` through `.claude/CLAUDE.md` | `settings.json` on edit, stop and write | `.claude/hooks/` |
| Cline, VS Code | `AGENTS.md`, `.clinerules/`, `.cline/rules/` | `.clinerules/hooks/<Event>`: see below | `.clinerules/hooks/PostToolUse`, `TaskComplete` |
| Cline CLI, SDK, Kanban | plugin rules | `beforeRun`, `afterRun`, `beforeModel`, `afterModel`, `beforeTool`, `afterTool`, `onEvent`, in a TypeScript `AgentPlugin`; project plugins in `.cline/plugins/` | a plugin may run `just guards` |
| Kilo Code | `AGENTS.md`, `.kilocode/rules/` (legacy) or `.kilo/rules/` through `kilo.jsonc` | None documented; customize offers rules, instructions, workflows and skills | `.kilocode/rules/agents.md` |
| Any tool with git | — | A `pre-commit` hook | `.githooks/pre-commit` |

## Cline hooks (extension 4.1.26)

The public Hooks page points at SDK plugins; the installed extension also
runs a script named after the event in `.clinerules/hooks/`:

`TaskStart`, `TaskResume`, `TaskCancel`, `TaskComplete`, `PreToolUse`,
`PostToolUse`, `UserPromptSubmit`, `Notification`, `PreCompact`.

The script is executable, has no extension on Linux, reads one JSON object
on stdin and writes one on stdout. The event object is named after the hook:
`postToolUse.toolName`, `postToolUse.parameters`, `taskComplete.taskMetadata`,
and so on. The answer is `{"cancel": bool, "contextModification": string,
"errorMessage": string}`; `cancel: true` stops the tool or the task, and
`contextModification` adds text the model sees. Cline kills a hook after
30 seconds, so the hooks here run `just guards`, never the full `gate`.

The two files in `.clinerules/hooks/` do that: `PostToolUse` reports a
failure as context, `TaskComplete` keeps the task open with the report.
They are symlinks to the portable templates in
`.agents/portable/hooks/cline/`; another repository copies the templates and
keeps the same layout.

## The gate

`just agent gate` runs `just check` then `just guards`; `just agent ci` adds
the Python, C and licence checks when their tools are installed.
`just install-hooks` sets `core.hooksPath` to `.githooks`, so every commit
runs `just guards` locally: no event API, no paid runner.

## Why the hook is the floor, not the ceiling

A tool's event names change; the shell does not. The checks in
`.agents/checks/` are the source, and every adapter above only names them.

## Sources

- [Cline hooks](https://docs.cline.bot/customization/hooks): the public page
  points at SDK plugins; the installed 4.1.26 extension implements
  `.clinerules/hooks/`.
- [Cline SDK plugins](https://docs.cline.bot/sdk/plugins): the seven
  lifecycle hooks and the `AgentPlugin` shape.
- [Cline rules](https://docs.cline.bot/customization/cline-rules):
  `AGENTS.md`, `.clinerules/`, `.cline/rules/`.
- [Kilo agents.md](https://kilo.ai/docs/customize/agents-md): root
  `AGENTS.md`, per-directory files, `CLAUDE.md`.
- [Kilo custom rules](https://kilo.ai/docs/customize/custom-rules):
  `.kilocode/rules/` is legacy-compatible; `.kilo/rules/` through
  `kilo.jsonc`.
