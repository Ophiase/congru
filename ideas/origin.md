# Origin

## The need

**[defined]** During an internship in medical computer vision, the author
worked on augmented-reality guidance for a shoulder operation. A headset had
to place a preoperative model of the bone over the real one. Two ways were
tried:

- **The whole model against the camera**: fine in a demonstration, weak in
  surgery, where only a small part of the bone is visible.
- **A few marks**: the surgeon marks some points on the bone, the camera
  finds them, and they are aligned rigidly onto the model. Practical, but
  sensitive to the first guess, to the noise of each mark, and to the
  viewpoint.

The second way is a registration problem with **few points, noisy points,
a partial view and a bad initial pose**: the hardest corner of the problem,
and the one where an error has a cost.

## The 2025 attempt

**[defined]** A first version, in Rust, started from a formal statement of
the problem: a stochastic observation model with three hypotheses and three
variants of increasing difficulty
([problem/](problem/observation-model.md)). It built data structures and the
noise generator, but no alignment, and stopped for lack of time.

What it got right, and is kept:

- **Start from the observation model**, not from an algorithm.
- **A generator with a known truth** to measure every method against.
- **Variants ordered by difficulty**, each with its own method.

What is dropped: the code, its layout, its dependencies.
