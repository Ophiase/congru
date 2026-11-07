# When the object is not quite the model

**[sketch]** A rigid answer on an object a little off: how far it holds, and what
a scale buys. [The curves](../../docs/benchmark/breaking.md), on a whole clean view
from a close guess.

| Trouble | Rigid | With a [similarity] |
| --- | --- | --- |
| 10 % larger | 0.51°, centre 11.5 mm off | 0.04°, centre 0.1 mm off, scale right |
| Bent by 16 mm | 0.35°, centre 4.9 mm off | The same: a scale cannot unbend |

- **A wrong scale hurts the place, hardly the turn**: an offset that looks like
  a bias, with a small rotation error to reassure.
- **A scale is cheap**: one closed form in each step.
- **A slight bend degrades smoothly**: a rigid fit is a fair answer, no need yet
  for [bending modes](../problem/hypothesis-ladder.md).

## Open

- **[question]** Should `Icp` itself offer a scale? `fit_similarity` is in the
  library; the loop is still in the benchmark.
- **[to check]** Partial views: a scale and a distance trade off when little is
  seen. These curves see the whole object.

[similarity]: https://doi.org/10.1109/34.88573
