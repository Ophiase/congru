# Benchmark results

Written by `just bench` from the evaluation seeds, 16 per situation; the
protocol, the situations and the metrics are defined in
[protocol.md](protocol.md). A cell reads *success rate · median
rotation error*; times are from the machine that wrote this page.

## Leaderboard

Every situation and model weighs the same. *Confidently wrong* is the
share of trials where the method trusted a pose that missed the
tolerance: for a robot, the most dangerous outcome.

| Method | Suite | Mean success | Worst situation | Confidently wrong | Median time |
| --- | --- | --- | --- | --- | --- |
| icp-point | registration | 32.2 % | 0 % (outliers 30 %, bracket) | 67.8 % | 76.29 ms |
| icp-plane | registration | 54.6 % | 0 % (outliers 30 %, bracket) | 45.4 % | 53.51 ms |
| icp-tukey | registration | 65.9 % | 0 % (no guess, bracket) | 32.9 % | 82.34 ms |
| icp-trimmed | registration | 59.1 % | 0 % (no guess, bracket) | 40.9 % | 69.59 ms |
| annealed | registration | 74.3 % | 0 % (no guess, bracket) | 21.9 % | 317.79 ms |
| adaptive | registration | 70.9 % | 0 % (no guess, bracket) | 26.0 % | 205.42 ms |
| search | registration | 79.8 % | 0 % (outliers 300 %, bracket) | 20.2 % | 3857.56 ms |
| guarded | registration | 66.3 % | 0 % (outliers 300 %, bracket) | 33.7 % | 113.72 ms |
| icp-tukey | tracking | 7.9 % | 2 % (knocks, bracket) | 86.6 % | 13.76 ms |
| adaptive | tracking | 30.5 % | 26 % (knocks, bracket) | 69.5 % | 44.27 ms |
| guarded | tracking | 9.9 % | 6 % (lens, bracket) | 90.1 % | 19.93 ms |
| tracker | tracking | 59.7 % | 32 % (shake, bracket) | 40.3 % | 12.72 ms |
| tracker-whole | tracking | 34.6 % | 16 % (shake, bracket) | 65.3 % | 7.28 ms |
| tracker-loose | tracking | 62.1 % | 34 % (shake, bracket) | 37.9 % | 6.44 ms |

## Recommendations

The best one-shot method for each class of situation, by mean success,
ties broken by time.

| Class | bracket | blob |
| --- | --- | --- |
| nominal | icp-tukey (100 %) | icp-trimmed (100 %) |
| noise | icp-point (97 %) | icp-plane (34 %) |
| outliers | icp-trimmed (100 %) | icp-trimmed (100 %) |
| partial view | icp-plane (97 %) | icp-tukey (100 %) |
| sparse | icp-plane (100 %) | icp-plane (62 %) |
| far guess | icp-plane (100 %) | icp-plane (97 %) |
| no guess | search (100 %) | icp-trimmed (6 %) |
| combined | adaptive (25 %) | annealed (12 %) |

## Methods and models

| Method | Suite | What it is |
| --- | --- | --- |
| icp-point | OneShot | point-to-point ICP, least squares |
| icp-plane | OneShot | point-to-plane ICP, least squares |
| icp-tukey | OneShot | point-to-plane ICP, Tukey kernel, gated |
| icp-trimmed | OneShot | point-to-plane ICP, Cauchy kernel, 70 % trimmed |
| annealed | OneShot | point-to-plane ICP, Cauchy kernel annealed from coarse to fine |
| adaptive | OneShot | point-to-plane ICP, Tukey kernel and gate set each stage from the median distance |
| search | OneShot | multistart over 32 rotations, Cauchy kernel |
| guarded | OneShot | Tukey ICP, multistart only when it keeps under 80 % |
| tracker | Tracking | two-phase tracker on the side facing the sensor |
| tracker-whole | Tracking | two-phase tracker on the whole model |
| tracker-loose | Tracking | two-phase tracker, motion prior made negligible |

| Model | What it tests | Size |
| --- | --- | --- |
| bracket | machined part: flat faces, thin plates, a boss and a pin | 0.250 |
| blob | smooth closed surface, nearly round: few features to lock on | 3.697 |
| patch | open curved surface, seen from one side, as a depth camera sees | 1.438 |
| gear | twelve teeth: the same after a twelfth of a turn, a trap for every method | 0.159 |

## Registration, one frame

### bracket

| Situation | icp-point | icp-plane | icp-tukey | icp-trimmed | annealed | adaptive | search | guarded |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| nominal | 100 % · 0.07° | 100 % · 0.11° | 100 % · 0.10° | 88 % · 0.19° | 100 % · 0.10° | 100 % · 0.10° | 100 % · 0.10° | 100 % · 0.10° |
| noise 2 % | 100 % · 0.56° | 100 % · 0.62° | 88 % · 1.18° | 56 % · 1.80° | 100 % · 0.87° | 100 % · 0.91° | 100 % · 0.72° | 88 % · 1.18° |
| noise 5 % | 94 % · 1.14° | 94 % · 0.99° | 6 % · 4.37° | 25 % · 3.12° | 31 % · 2.67° | 56 % · 1.68° | 69 % · 1.71° | 6 % · 4.37° |
| outliers 30 % | 0 % · 3.48° | 0 % · 3.47° | 100 % · 0.12° | 100 % · 0.14° | 100 % · 0.10° | 100 % · 0.10° | 100 % · 0.18° | 100 % · 0.12° |
| outliers 100 % | 0 % · 4.42° | 0 % · 3.79° | 100 % · 0.13° | 100 % · 0.14° | 100 % · 0.10° | 100 % · 0.14° | 100 % · 0.55° | 100 % · 0.13° |
| outliers 300 % | 0 % · 5.38° | 0 % · 5.69° | 100 % · 0.23° | 100 % · 0.38° | 100 % · 0.18° | 100 % · 0.85° | 0 % · 5.09° | 0 % · 5.09° |
| half visible | 94 % · 0.16° | 100 % · 0.16° | 100 % · 0.16° | 75 % · 0.38° | 100 % · 0.15° | 100 % · 0.16° | 100 % · 0.15° | 100 % · 0.16° |
| quarter visible | 88 % · 0.36° | 94 % · 0.32° | 81 % · 0.49° | 69 % · 0.69° | 94 % · 0.29° | 94 % · 0.32° | 50 % · 0.78° | 81 % · 0.49° |
| sparse | 81 % · 0.70° | 100 % · 0.42° | 62 % · 0.50° | 31 % · 4.78° | 100 % · 0.41° | 100 % · 0.48° | 100 % · 0.41° | 62 % · 0.50° |
| guess 30° | 100 % · 0.07° | 100 % · 0.11° | 100 % · 0.10° | 69 % · 0.27° | 100 % · 0.10° | 100 % · 0.10° | 100 % · 0.10° | 100 % · 0.10° |
| guess 60° | 100 % · 0.06° | 100 % · 0.11° | 6 % · 31.87° | 31 % · 7.88° | 100 % · 0.10° | 100 % · 0.10° | 100 % · 0.10° | 31 % · 20.13° |
| no guess | 0 % · 175.39° | 6 % · 164.59° | 0 % · 173.05° | 0 % · 179.47° | 0 % · 179.77° | 0 % · 179.79° | 100 % · 0.10° | 94 % · 0.11° |
| everything | 0 % · 9.83° | 0 % · 10.11° | 0 % · 11.30° | 0 % · 4.58° | 12 % · 2.35° | 25 % · 2.99° | 19 % · 3.53° | 0 % · 11.30° |

### blob

| Situation | icp-point | icp-plane | icp-tukey | icp-trimmed | annealed | adaptive |
| --- | --- | --- | --- | --- | --- | --- |
| nominal | 38 % · 2.79° | 100 % · 0.40° | 100 % · 0.37° | 100 % · 0.56° | 100 % · 0.39° | 100 % · 0.46° |
| noise 2 % | 19 % · 2.95° | 56 % · 1.50° | 19 % · 3.08° | 0 % · 3.10° | 38 % · 2.18° | 25 % · 2.56° |
| noise 5 % | 12 % · 4.19° | 12 % · 5.94° | 0 % · 12.35° | 0 % · 9.33° | 0 % · 12.79° | 0 % · 10.70° |
| outliers 30 % | 0 % · 8.78° | 0 % · 12.01° | 100 % · 0.40° | 100 % · 0.56° | 100 % · 0.37° | 100 % · 0.42° |
| outliers 100 % | 0 % · 20.21° | 0 % · 22.05° | 100 % · 0.39° | 100 % · 0.70° | 100 % · 0.50° | 100 % · 0.39° |
| outliers 300 % | 0 % · 27.12° | 0 % · 29.20° | 100 % · 0.40° | 100 % · 1.30° | 100 % · 0.49° | 0 % · 5.44° |
| half visible | 6 % · 3.42° | 100 % · 0.45° | 100 % · 0.43° | 100 % · 1.01° | 100 % · 0.45° | 100 % · 0.46° |
| quarter visible | 6 % · 5.16° | 100 % · 0.78° | 100 % · 0.77° | 81 % · 1.11° | 100 % · 0.78° | 100 % · 0.91° |
| sparse | 0 % · 5.58° | 62 % · 1.46° | 56 % · 1.54° | 12 % · 2.86° | 56 % · 1.54° | 56 % · 1.87° |
| guess 30° | 0 % · 6.79° | 100 % · 0.40° | 100 % · 0.37° | 100 % · 0.53° | 100 % · 0.39° | 100 % · 0.46° |
| guess 60° | 0 % · 19.15° | 94 % · 0.40° | 88 % · 0.40° | 94 % · 0.61° | 88 % · 0.38° | 88 % · 0.46° |
| no guess | 0 % · 178.59° | 0 % · 123.33° | 0 % · 177.79° | 6 % · 155.35° | 0 % · – | 0 % · – |
| everything | 0 % · 18.90° | 0 % · 25.02° | 6 % · 4.62° | 0 % · 4.17° | 12 % · 3.62° | 0 % · 4.31° |

## Tracking, frame after frame

### bracket

| Situation | icp-tukey | adaptive | guarded | tracker | tracker-whole | tracker-loose |
| --- | --- | --- | --- | --- | --- | --- |
| noise | 14 % · 3.64° | 37 % · 2.26° | 14 % · 3.64° | 80 % · 1.40° | 52 % · 1.84° | 82 % · 1.38° |
| shake | 8 % · 3.33° | 34 % · 2.21° | 8 % · 3.33° | 32 % · 1.53° | 16 % · 2.17° | 34 % · 1.53° |
| bend | 11 % · 3.22° | 29 % · 2.38° | 11 % · 3.22° | 68 % · 1.55° | 36 % · 2.10° | 69 % · 1.72° |
| occluder | 6 % · 3.77° | 31 % · 2.44° | 12 % · 3.74° | 41 % · 2.25° | 35 % · 2.34° | 56 % · 1.76° |
| lens | 6 % · 3.96° | 27 % · 2.46° | 6 % · 3.96° | 72 % · 1.52° | 32 % · 2.33° | 66 % · 1.65° |
| knocks | 2 % · 31.53° | 26 % · 2.46° | 8 % · 3.65° | 66 % · 1.72° | 36 % · 2.26° | 64 % · 1.70° |
