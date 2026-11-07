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
| ICP, a guess 20° off | 2.32 | 3.20 | 3 of 7 | 5 of 32 |
| ICP, no guess | 3.88 | 9.31 | 2 of 7 | 2 of 32 |
| Multistart, 64 starts | 1.46 | 2.55 | 5 of 7 | 7 of 32 |

## What the proof says

- **Closed**: 7 of 32 within the budget; elsewhere the bound holds, far.
- **The best pose is near the truth**: 10 of 32.
- **The data choose**: on 0 of 32, every pose not excluded is within
  10°; a `+` below says more may be left.

## Instances

| Instance | Closed | Best pose off, ° | Not excluded, ° | Seconds |
| --- | --- | ---: | ---: | ---: |
| bracket, 12 marks, 1 mm, 0 strays | yes | 7.0 | 180+ | 2 |
| bracket, 12 marks, 1 mm, 0 strays | yes | 0.4 | 23+ | 4 |
| bracket, 12 marks, 1 mm, 3 strays | no | 147.0 | 0+ | 5 |
| bracket, 12 marks, 1 mm, 3 strays | no | 172.1 | 0+ | 7 |
| bracket, 12 marks, 3 mm, 0 strays | no | 147.6 | 180+ | 7 |
| bracket, 12 marks, 3 mm, 0 strays | no | 5.3 | 22+ | 5 |
| bracket, 12 marks, 3 mm, 3 strays | no | 86.2 | 0+ | 6 |
| bracket, 12 marks, 3 mm, 3 strays | no | 173.3 | 0+ | 7 |
| bracket, 30 marks, 1 mm, 0 strays | yes | 0.4 | 16 | 4 |
| bracket, 30 marks, 1 mm, 0 strays | yes | 0.3 | 23+ | 5 |
| bracket, 30 marks, 1 mm, 7 strays | no | 3.8 | 0+ | 14 |
| bracket, 30 marks, 1 mm, 7 strays | no | 1.4 | 0+ | 16 |
| bracket, 30 marks, 3 mm, 0 strays | yes | 2.6 | 15+ | 8 |
| bracket, 30 marks, 3 mm, 0 strays | no | 1.4 | 22+ | 9 |
| bracket, 30 marks, 3 mm, 7 strays | no | 10.3 | 0+ | 14 |
| bracket, 30 marks, 3 mm, 7 strays | no | 1.9 | 0+ | 15 |
| gear, 12 marks, 1 mm, 0 strays | no | 0.5 | 180+ | 3 |
| gear, 12 marks, 1 mm, 0 strays | no | 90.0 | 180+ | 3 |
| gear, 12 marks, 1 mm, 3 strays | no | 76.6 | 0+ | 4 |
| gear, 12 marks, 1 mm, 3 strays | no | 49.6 | 0+ | 4 |
| gear, 12 marks, 3 mm, 0 strays | no | 104.0 | 158+ | 3 |
| gear, 12 marks, 3 mm, 0 strays | no | 165.0 | 179+ | 3 |
| gear, 12 marks, 3 mm, 3 strays | no | 174.0 | 0+ | 4 |
| gear, 12 marks, 3 mm, 3 strays | no | 140.3 | 0+ | 4 |
| gear, 30 marks, 1 mm, 0 strays | yes | 120.7 | 180+ | 5 |
| gear, 30 marks, 1 mm, 0 strays | yes | 149.3 | 179+ | 6 |
| gear, 30 marks, 1 mm, 7 strays | no | 14.5 | 0+ | 10 |
| gear, 30 marks, 1 mm, 7 strays | no | 172.6 | 0+ | 9 |
| gear, 30 marks, 3 mm, 0 strays | no | 2.1 | 0+ | 7 |
| gear, 30 marks, 3 mm, 0 strays | no | 150.9 | 0+ | 7 |
| gear, 30 marks, 3 mm, 7 strays | no | 102.2 | 0+ | 9 |
| gear, 30 marks, 3 mm, 7 strays | no | 62.4 | 0+ | 9 |
