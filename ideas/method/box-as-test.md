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
| 0.05, 0.2 | 90 | 78 | 72 | 72 | 72 |
| 0.25, 0.6 | 89 | 77 | 72 | 72 | 72 |
| None, 90 or 36 frames | 88 of 90 | **0 of 90** | 0 of 36 | 0 of 36 | 36 of 36 |

- **A sloppy box costs little**, once the test reads the share of the object rather
  than its middle.
- **Without a box, the room alone holds, and the room with every effect does not.**
  Twins and alike crates cannot be told apart by shape: that is no failure.

`CONGRU_BOX=shift,growth` sets it.
