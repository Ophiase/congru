# A believed pose must stay in doubt

**[sketch]** A pose that confirms itself was never searched again: a wrong first
search held for a whole film, at a wider box
([the table](box-as-test.md)).

- **Search again, at regular looks**, every six frames in the films, whatever the
  frame says.
- **A rival takes over only after winning twice running, at the same place**, by
  more than a tenth of confirmation. A single search won by chance on a noisy
  frame, and took a right pose away: one win cost 12 frames at the tight box.
- **The trajectory then starts over**: its velocity belonged to the wrong pose.

## What it gives

Frames within 2 % and frames believed and wrong, of 72, on the low-angle bracket:

| Box, seed | No doubt | One win | Two wins |
| --- | --- | --- | --- |
| Tight, 0 | 72, 0 | 60, 12 | **72, 0** |
| ±10 %, +40 %, 0 | 0, 72 | 48, 24 | **60, 12** |
| ±25 %, +100 %, 0 | 0, 72 | 48, 24 | **48, 24** |

- **The wrong frames are those before the second win**: at most two looks.
- **It costs about a third more time**: a search at each look.

## What it does not do

- **[caution]** A real twin stays: at the widest box the crate is believed wrong
  for the whole film. Shape cannot tell, and only a tighter box can.
- **[caution]** It brings nothing when a search cannot find the object: the
  close car and the flat gear refuse instead.
- **[question]** Off by default in the library: is a search at regular looks
  acceptable where time counts? It stays the caller's choice, `doubting_every`.
