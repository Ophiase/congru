# Long jobs

A job that may take more than about fifteen seconds (a build, a demo, a
benchmark) is never launched blind. The user must always know what the agent
is doing, why the focus is held, and how far along it is.

1. **Announce** it before launching: what it runs and a rough estimate of
   its duration.
2. **Split** it into steps short enough to report between them: one demo,
   one crate, one scene per command rather than one recipe that runs all.
   Some tools (Cline) cannot be interrupted, only queued: there, splitting is
   the only way the user keeps a hand on the run.
3. **Report** after each step: what finished, what it gave, what comes next.
4. **Show progress** on anything still long: run it in the background with
   its output in a log, and read the log at intervals, giving a count or a
   percentage (frame 45 / 90, scene 3 / 6) rather than silence.
5. A program the project owns that runs long prints its own progress, so
   that step 4 has something to read.
