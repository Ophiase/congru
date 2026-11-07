# A box tests a pose

**[sketch]** The noisy box stands for a 2D detector, the one thing taken from the
truth. It proposes, and must also forbid.

- **Why**: [a search judges on the whole frame](ray-evidence.md), so a neighbouring
  crate, the same shape, outscored the target: 72 to 0 of 72.

**A pose stands only with most of the object's points in the box**, three quarters.
A rectangle against a pose: nothing is cut from the data.

## How much the box matters

Frames within the bound, on one seed. Shift and growth are shares of the object's
size. The first line is the usual box.

| Box: shift, growth | Room | In a room | Crate among crates | Twin brackets | Part on a table |
| --- | --- | --- | --- | --- | --- |
| 0.05, 0.2 | 90 | 75 | 72 | 72 | 72 |
| 0.25, 0.6 | 89 | 78 | 72 | 72 | 72 |
| None, 90 or 36 frames | 88 of 90 | **74 of 90** | 0 of 36 | 0 of 36 | 36 of 36 |

- **Without any box, nothing from the truth**, all in a room gives 74, 71, 69 on
  three seeds: the object is found by shape, once a search asks 0.3 of agreement.
- **Twins and alike crates cannot be told apart by shape**: that is no failure,
  and what a box is for.

`CONGRU_BOX=shift,growth` sets it. The scenes ran 36 frames, without a box a search
costs seconds.

## All fourteen scenes

**[caution]** The usual box is kind: 10 % per side, 5 % off, around the whole
silhouette even where it is hidden, at every frame. The table above read "a sloppy
box costs nothing" from five cases; on all the scenes that does not hold.

Frames within 2 % of the size, of 72, on seeds 0 and 1, at 0.1.25:

| Scene | 0.05, 0.2 | 0.1, 0.4 | 0.25, 1.0 |
| --- | --- | --- | --- |
| Bracket at a low angle | 72, 72 | 0, 72 | 0, 70 |
| Car close by | 48, 42 | 0, 42 | 0, 42 |
| Crate among crates | 72, 72 | 72, 72 | 0, 0 |
| Gear from above | 72, 72 | 72, 72 | 0, 0 |
| Bracket behind a crate | 66, 60 | 54, 60 | 66, 66 |
| Car on a road | 54, 72 | 54, 41 | 48, 42 |
| Shelf of mugs | 72, 72 | 71, 72 | 72, 60 |
| The seven others | 71 or 72 | 72 | 72 |

- **The crate and the flat gear need a tight box**: both seeds fall together.
- **The low angle and the close car depend on the seed**: all or nothing, so the
  first search decides. The bracket then stays 11.6 % off for the whole film: a
  wrong pose is believed and never doubted.
- **Believed and wrong**, seed 0: the low-angle bracket gives 72 of 72 at both wider
  boxes, the crate 72 of 72 at the widest. The close car and the flat gear refuse
  instead, which is the better failure. The usual box has 12, on the car on a road.
- **[to check]** Two seeds show the failures, not their rate.
- **[to check]** Still from the truth: no box yet leaves out a hidden part, or
  misses a frame.
