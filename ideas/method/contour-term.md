# A contour term, measured and left out

**[sketch]** The edge of a silhouette, where the image jumps away too, says what
a flat face cannot: where the object stops along itself. So its rows join the
[information of a pose](pose-information.md).

## Measured

**[caution]** Used only to rank near poses, never to believe one, against the same
run without it. It stays out of the code; the last version is on the branch
`dev/contour-term`.

| Case | Without | With | Used only where faces leave a direction free |
| --- | --- | --- | --- |
| Gear from above, of 72 | 72 | 72 | 72 |
| Its median error | 0.58 % | 0.49 % | 0.49 % |
| Room, of 90 | 81 | 84 | 82 |
| In a room, of 90 | 74 | 67 | 66 |
| Shelf of mugs, of 72 | 64 | 58 | 58 |

## Why it loses

Any wrong pose touching a depth edge gains from it: in clutter, edges are
everywhere, and the geometric mean rises fastest on what had nothing.

## Why it is no longer wanted

- **Its gain on flat views**, 29 to 72 at first, came mostly from
  [a flat view being read at all](flat-view-reading.md).
- **What remains** is a smaller error on views already held, and
  [a short alignment](what-answers.md) gives that without the loss.
