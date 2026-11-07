# Compared on the same frames

Written by `just comparison`. Same frames, same 3 cm gate, at most 50 iterations,
one thread, each library at its own defaults otherwise, not tuned; a bracket, 16
seeds per line.

- **Success**: within 2° and 2 % of the size of the truth.
- **Median time**: the alignment alone, its target already prepared.
- **Mean over all**: the last line weighs each situation the same.

| Situation | congru, point to plane | Open3D, point to plane | small_gicp, point to plane | small_gicp, GICP |
| --- | ---: | ---: | ---: | ---: |
| a guess 10° off | 100 %, 0.04° | 100 %, 0.05° | 100 %, 0.04° | 100 %, 0.08° |
| a guess 30° off | 100 %, 0.04° | 100 %, 0.04° | 100 %, 0.04° | 100 %, 0.08° |
| 5 mm of noise | 100 %, 0.48° | 100 %, 0.49° | 100 %, 0.40° | 75 %, 1.48° |
| 30 % strays | 100 %, 0.12° | 100 %, 0.12° | 100 %, 0.13° | 100 %, 0.08° |
| half of it seen | 100 %, 0.10° | 100 %, 0.09° | 100 %, 0.09° | 100 %, 0.17° |
| all of them | 81 %, 1.33° | 75 %, 1.45° | 69 %, 1.54° | 12 %, 3.23° |
| **Mean over all, median time** | 97 %, 51.3 ms | 96 %, 16.0 ms | 95 %, 30.3 ms | 81 %, 35.7 ms |
