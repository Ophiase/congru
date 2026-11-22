# Graded against a proof

Written by `just bench-certified`, with [this bound](certified-method.md). Few
marks on a part, their correspondences unknown, some of them strays.

- **Gap**: how far a method's value can be above the optimum, as an rms in mm, on
  the instances the proof closed: elsewhere it is only a ceiling. The proof stops
  at 1 mm, so every gap counts that 1 mm.
- **Near the truth**: its pose within 5° of the motion that made the marks. A gear
  turned by some teeth counts as far, though it is a right answer.

## Methods

| Method | Mean gap, mm | Worst, mm | Within 2 mm | Near the truth, all |
| --- | ---: | ---: | ---: | ---: |
| ICP, a guess 20° off | 4.26 | 8.30 | 5 of 27 | 5 of 32 |
| ICP, no guess | 4.52 | 9.31 | 6 of 27 | 2 of 32 |
| Multistart, 64 starts | 2.74 | 4.74 | 10 of 27 | 7 of 32 |

## What the proof says

- **Closed**: 27 of 32 within the budget; elsewhere the bound holds, far.
- **The best pose is near the truth**: 11 of 32.
- **The data choose**: on 0 of 32, every pose not excluded is within
  10°; a `+` below says more may be left.

## Instances

| Instance | Closed | Best pose off, ° | Not excluded, ° | Seconds |
| --- | --- | ---: | ---: | ---: |
| bracket, 12 marks, 1 mm, 0 strays | yes | 0.7 | 180+ | 7 |
| bracket, 12 marks, 1 mm, 0 strays | yes | 0.4 | 27+ | 1 |
| bracket, 12 marks, 1 mm, 3 strays | yes | 141.5 | 60+ | 5 |
| bracket, 12 marks, 1 mm, 3 strays | yes | 19.5 | 0+ | 7 |
| bracket, 12 marks, 3 mm, 0 strays | no | 14.1 | 179+ | 12 |
| bracket, 12 marks, 3 mm, 0 strays | yes | 5.3 | 29+ | 3 |
| bracket, 12 marks, 3 mm, 3 strays | yes | 88.1 | 179+ | 6 |
| bracket, 12 marks, 3 mm, 3 strays | yes | 162.4 | 0+ | 7 |
| bracket, 30 marks, 1 mm, 0 strays | yes | 0.4 | 16 | 1 |
| bracket, 30 marks, 1 mm, 0 strays | yes | 0.3 | 23 | 2 |
| bracket, 30 marks, 1 mm, 7 strays | yes | 0.4 | 0+ | 10 |
| bracket, 30 marks, 1 mm, 7 strays | yes | 1.8 | 0+ | 9 |
| bracket, 30 marks, 3 mm, 0 strays | yes | 2.9 | 18+ | 4 |
| bracket, 30 marks, 3 mm, 0 strays | yes | 1.2 | 26+ | 11 |
| bracket, 30 marks, 3 mm, 7 strays | yes | 4.4 | 0+ | 12 |
| bracket, 30 marks, 3 mm, 7 strays | yes | 1.2 | 0+ | 11 |
| gear, 12 marks, 1 mm, 0 strays | yes | 89.6 | 180+ | 4 |
| gear, 12 marks, 1 mm, 0 strays | yes | 0.9 | 180+ | 2 |
| gear, 12 marks, 1 mm, 3 strays | yes | 85.3 | 0+ | 3 |
| gear, 12 marks, 1 mm, 3 strays | yes | 147.6 | 0+ | 2 |
| gear, 12 marks, 3 mm, 0 strays | no | 15.8 | 180+ | 11 |
| gear, 12 marks, 3 mm, 0 strays | no | 164.5 | 179+ | 11 |
| gear, 12 marks, 3 mm, 3 strays | yes | 146.3 | 0+ | 3 |
| gear, 12 marks, 3 mm, 3 strays | yes | 28.0 | 0+ | 3 |
| gear, 30 marks, 1 mm, 0 strays | yes | 59.3 | 180+ | 3 |
| gear, 30 marks, 1 mm, 0 strays | yes | 120.7 | 180+ | 3 |
| gear, 30 marks, 1 mm, 7 strays | yes | 58.9 | 0+ | 3 |
| gear, 30 marks, 1 mm, 7 strays | yes | 121.4 | 0+ | 4 |
| gear, 30 marks, 3 mm, 0 strays | no | 118.1 | 156+ | 22 |
| gear, 30 marks, 3 mm, 0 strays | no | 178.9 | 0+ | 23 |
| gear, 30 marks, 3 mm, 7 strays | yes | 124.2 | 0+ | 4 |
| gear, 30 marks, 3 mm, 7 strays | yes | 116.5 | 0+ | 7 |
