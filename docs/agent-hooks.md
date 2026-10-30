# Agents: rules, hooks and the gate

What each tool reads, what it can run by itself, and the one thing that works
everywhere.

| Tool | Rules it reads | Lifecycle hooks | Adapter here |
| --- | --- | --- | --- |
| Claude Code | `AGENTS.md` through `.claude/CLAUDE.md` | `settings.json` on edit, stop and write | `.claude/hooks/` |
| Cline, VS Code | `AGENTS.md`, `.clinerules/`, `.cline/rules/` | `.clinerules/hooks/<Event>`: see below | `.clinerules/hooks/PostToolUse`, `TaskComplete`, `PreToolUse` |
| Cline CLI, SDK, Kanban | plugin rules | `beforeRun`, `afterRun`, `beforeModel`, `afterModel`, `beforeTool`, `afterTool`, `onEvent`, in a TypeScript `AgentPlugin`; project plugins in `.cline/plugins/` | a plugin may run `just guards` |
| Cursor | `AGENTS.md`, `.cursor/rules/*.mdc` | `.cursor/hooks.json`: see below | `.cursor/hooks/beforeShellExecution`, `stop` |
| Kilo Code | `AGENTS.md`, `.kilocode/rules/` (legacy) or `.kilo/rules/` through `kilo.jsonc` | None documented; `permission` in `.kilo/kilo.jsonc` | `.kilocode/rules/agents.md`, `.kilo/kilo.jsonc` |
| GitHub Copilot | `.github/copilot-instructions.md`, `AGENTS.md` | Not wired | `.github/copilot-instructions.md` |
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

The files in `.clinerules/hooks/` do that: `PostToolUse` reports a failure
as context, `TaskComplete` keeps the task open with the report, and
`PreToolUse` blocks the commands the policy denies (Cline has no ask
decision). They are symlinks to the portable templates in
`.agents/portable/hooks/cline/`; another repository copies the templates and
keeps the same layout.

## Cursor hooks

Cursor reads `.cursor/hooks.json` at the project root. The two hooks here are
`beforeShellExecution` (allow, ask or deny a command; `failClosed`) and
`stop` (`followup_message` keeps the agent working while `just guards`
fails). The scripts are symlinks to `.agents/portable/hooks/cursor/`; Cursor
runs them from the project root.

## Permissions

| Tool | Project file | allow | ask | deny |
| --- | --- | --- | --- | --- |
| Claude Code | `.claude/settings.json` | yes | yes | yes |
| Cline | `.clinerules/hooks/PreToolUse` | yes | no, the UI | yes |
| Cursor | `.cursor/hooks.json` + `beforeShellExecution` | yes | yes | yes |
| Kilo | `.kilo/kilo.jsonc` | yes | yes | yes |
| Copilot | `.github/copilot-instructions.md` | instructions only | no | no |

The allow and ask halves are each tool's UI; the deny half is a file we can
version.

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
- [Kilo permissions](https://kilo.ai/docs/getting-started/settings/auto-approving-actions):
  `permission` in `.kilo/kilo.jsonc`, with allow, ask and deny.
- [Cursor hooks](https://cursor.com/docs/hooks): `.cursor/hooks.json`,
  `beforeShellExecution` and `stop`.
- [Copilot instructions](https://docs.github.com/en/copilot/how-tos/configure-custom-instructions/add-repository-instructions):
  `.github/copilot-instructions.md`.
