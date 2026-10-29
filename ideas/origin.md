# Origin

## The need

**[defined]** The author first met the problem in augmented reality: a
model had to be laid over a real object seen by a camera. Two ways were
tried:

- **The whole model against the image**: fine in a demonstration, weak in
  practice, where only a small part of the object is visible.
- **A few marks**: some points are marked on the object, the camera finds
  them, and they are aligned rigidly onto the model. Practical, but
  sensitive to the first guess, to the noise of each mark, and to the
  viewpoint.

The second way is a registration problem with **few points, noisy points,
a partial view and a bad initial pose**: the hardest corner of the problem,
and the one where an error has a cost. Robots calibrating against a part,
and headsets anchoring a model, share it.

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
