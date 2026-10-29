---
name: review
description: Review the current branch as the author would, verdict blocked until proven otherwise; with "auto", fix the findings and review again until nothing blocks. Run before declaring any topic done, and before merge-master.
---

# Review

The author reads the publication first (README, figures, `ideas/`), the code
later, by curiosity. A review checks both, in that order, and assumes the
work is not done until it shows otherwise.

## Gather

1. `git diff main...HEAD --stat`, then the diff itself.
2. `just check` and `just guards`: a failure is a blocker, stop there.
3. `just figures` and `just demo` when code behind a figure or the
   animation changed: the committed images must be the ones the code draws.

## Judge

Label each finding: **[B]** blocks the merge, **[W]** should be fixed now,
**[D]** may wait, with the reason it can wait.

### The publication

- [B] The README states something the code does not do, or a number no
  command reproduces.
- [B] A figure or animation is stale, hand-drawn, or unreadable in dark
  mode.
- [B] A decision taken in code that `ideas/` does not record.
- [W] A paragraph a newcomer cannot follow without reading the code.

### The code

- [B] A test that checks a residual, a count or a snapshot where the truth
  is known: it must check the pose against the truth.
- [B] A doc or comment that paraphrases, a `#[allow]`, an unnamed lossy
  cast, a panic path in a library.
- [B] Anything that names a private project or ties this one to the
  author's past employers.
- [W] A function past forty lines, a file past two hundred, a folder past
  seven entries, even when the checks did not see it.
- [W] A stringly-typed option, a tuple of three or more in a signature, a
  default restated in a binding.

## The last pass

Before the verdict, ask: **what would the author shout about after the
merge?** An ugly figure, a vague sentence in the README, a demo that only
works on one seed, a name that says nothing. Fix it or list it as [B].

## Verdict

**Blocked** while any [B] stands. With `auto`, fix every [B] and [W], run
the review again, and stop when the verdict is **ready**; then say what was
fixed and what was deferred, and why.
