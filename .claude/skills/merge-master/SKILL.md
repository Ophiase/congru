---
name: merge-master
description: Merge a finished dev/<topic> branch into main as the only developer, with a small version bump, a version tag and the branch deleted. Run by the agent itself when a topic is done.
---

# Merge master

For a project with **one developer and no pull requests**. The day several
people work on it, this skill is obsolete: switch to pull requests and
reviews, and delete it.

## When

A topic on `dev/<topic>` is done and every check passes. The agent runs
this on its own; it never pushes.

## Steps

1. **Checks**: `just check` and `just guards`.
2. **Confidentiality**: `git grep -i` for the names kept in
   `.agents/local.md` finds nothing, nor does `git log main..HEAD -p`.
3. **A small bump**, on the branch: the patch number of `VERSION` and of
   `workspace.package.version` in `Cargo.toml`, together (0.1.0 → 0.1.1),
   the minor one only when a set of features lands; `just guards` checks
   they agree. Write two to five lines under the
   new version in `CHANGELOG.md`, the top of `Unreleased` moved there. Keep
   each release short.
4. **Merge**: `git switch main && git merge --no-ff dev/<topic>`, the
   message naming the topic.
5. **Tag**: `git tag -a v<version> -m "<the topic, one line>"`.
6. **Clean up**: `git branch -d dev/<topic>`.
