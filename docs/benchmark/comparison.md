# Compared on the same frames

Written by `just comparison`. Same frames, same 3 cm gate, at most 50 iterations,
one thread, each library at its own defaults otherwise, not tuned; a bracket, 16
seeds per line.

- **Success**: within 2° and 2 % of the size of the truth.
- **Median time**: the alignment alone, its target already prepared.
- **Stopped as small_gicp**: congru's default stops at steps of 1e-7, much later
  than the others; this column stops at 0.1° and 1 mm.
- **On a distance field**: stopped the same, the mesh read as a field of 2 mm; the
  others get samples every 3 mm.
- **Mean over all**: the last line weighs each situation the same.

| Situation | congru, point to plane | congru, stopped as small_gicp | congru, on a distance field | Open3D, point to plane | small_gicp, point to plane | small_gicp, GICP |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| a guess 10° off | 100 %, 0.04° | 100 %, 0.05° | 100 %, 0.04° | 100 %, 0.05° | 100 %, 0.04° | 100 %, 0.08° |
| a guess 30° off | 100 %, 0.04° | 100 %, 0.05° | 100 %, 0.04° | 100 %, 0.04° | 100 %, 0.04° | 100 %, 0.08° |
| 5 mm of noise | 100 %, 0.48° | 100 %, 0.44° | 100 %, 0.54° | 100 %, 0.48° | 100 %, 0.40° | 75 %, 1.48° |
| 30 % strays | 100 %, 0.12° | 100 %, 0.12° | 100 %, 0.12° | 100 %, 0.12° | 100 %, 0.13° | 100 %, 0.08° |
| half of it seen | 100 %, 0.10° | 100 %, 0.10° | 100 %, 0.08° | 100 %, 0.09° | 100 %, 0.09° | 100 %, 0.17° |
| all of them | 81 %, 1.33° | 81 %, 1.33° | 75 %, 1.58° | 75 %, 1.39° | 69 %, 1.54° | 12 %, 3.23° |
| **Mean over all, median time** | 97 %, 51.0 ms | 97 %, 20.7 ms | 96 %, 3.7 ms | 96 %, 5.9 ms | 95 %, 18.4 ms | 81 %, 21.3 ms |
