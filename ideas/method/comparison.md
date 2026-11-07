# Compared on the same frames

**[sketch]** A library that does not say where others win is not believed. The
same frames go through congru, [Open3D](https://www.open3d.org/) and
[small_gicp](https://github.com/koide3/small_gicp): [the
table](../../docs/benchmark/comparison.md).

- **Fair by construction**: same frames, same gate, same iteration cap, one
  thread, each library at its own defaults, none tuned.
- **The same accuracy**: 95 to 97 % for the point-to-plane methods; congru leads
  by a little on the hardest line, by 81 % to 75 and 69.
- **On points, Open3D is about three times faster**: 5.9 ms against 20.7 once
  congru stops as early as small_gicp, 18.4 ms. Its own default, steps of 1e-7,
  costs 51 ms for no gain here. **[question]** Should the default stop earlier?
- **On a [distance field](representations.md), congru is the fastest**: 3.7 ms,
  at 96 % as Open3D. **[caution]** It needs the model's mesh and about 250 ms once
  to build the field, where the others were given samples; Open3D may have ways
  this script does not use.
- **The others stay outside the library**: a script in `experiments/comparison`,
  run through a throwaway environment.
- **[caution]** One model, simulated frames, 16 seeds: this says little of real
  scans, and the times come from one machine.
- **[to check]** The other libraries' options could be better chosen by someone
  who knows them; the script is there to be corrected.
