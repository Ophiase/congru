# The aligner contract

**[defined]** Every alignment method, in the library or written by a user,
is one thing: an **`Aligner`**. It owns its model, is handed one frame at a
time, and answers with the model's pose in the sensor's frame.

```rust
pub trait Aligner {
    fn name(&self) -> &str;
    fn align(&mut self, frame: &Frame<'_>) -> Result<Estimate, Error>;
    fn reset(&mut self) {}
}
```

| Piece | Meaning |
| --- | --- |
| `Frame { scan, guess }` | The points the sensor returned, in its frame; a pose to start from, when the caller has one (a detector, an encoder, the robot's kinematics) |
| `Estimate { pose, trusted, observability }` | The pose mapping the model's frame to the sensor's; whether the method stands behind it; which motions it could not see |
| `&mut self` | A method may remember: a tracker keeps its filter between frames, a one-shot method keeps nothing |
| `reset` | Forget the past, as after a cut in the video |

## Why this shape

- **One signature for one-shot and tracking.** The benchmark runs every
  method through the same loop; a one-shot method simply ignores the past,
  a tracker ignores the guess after its first frame.
- **The model is bound at construction**, so building a search index or a
  sampled mesh happens once, outside the timed loop.
- **`trusted` is part of the answer.** A method that knows it failed must
  say so; the benchmark counts a confident wrong pose as worse than an
  admitted failure ([measuring success](../problem/measuring-success.md)).
- **No generic parameters**: `Box<dyn Aligner>` works, so a registry of
  methods, a plugin or a binding can hold any of them.

## Adding a method

Implement the trait, add one line to the method registry of the benchmark,
run `just bench`: the method gets its row in every table of
[the results](../../docs/benchmark/results.md).

**[caution]** The contract is unstable until the author declares 1.0.
