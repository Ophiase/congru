# Acceleration

**[defined]** The author's rule: the project works on a CPU alone, and hardware
acceleration is a bonus when the machine has it.

- **CPU by default**: no build, test or recipe may need a GPU.
- **A GPU is an optional feature**, chosen at build time, and falls back to the
  CPU when no device answers.
- **The same answer either way**: a test compares the two within a bound.

## Where a frame's time goes

**[sketch]** Part on a table, 320×240, 72 frames, 12 cores, at 0.1.25, about
0.4 s per frame, one run with a stopwatch:

| Stage | Share |
| --- | --- |
| Scan, the pose brought along rays | 40 % |
| Short alignment and its confirmation | 40 % |
| Judgement of the answer | 20 % |

- **Rendering is 6 % of it**: 110 calls per frame, 0.18 ms each, spread over the
  stages above.
- **No GPU for rendering**: **[guess]** a round trip to a device costs more than
  0.18 ms, so each call would get slower.
- **Tracking and vision run on one thread**: [performance](performance.md) is
  parallel in the library's search only. That comes before any GPU, and helps
  every laptop.
- **[to check]** Which loops inside the three stages cost most: no single one
  stands out at this grain.
