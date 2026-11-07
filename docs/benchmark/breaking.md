# When the object is not quite the model

Written by `just bench-breaking`: a bracket scanned with 1 mm of noise, from a guess
10° and 1 cm off, median of 16 seeds.

- **Rigid**: ICP, the only motion it knows.
- **With a scale**: nearest points and a
  [similarity](https://doi.org/10.1109/34.88573), in turn.
- **Centre off**: how far the object's centre, which a scale leaves in place, lands
  from where it should.

| The object | Rigid, ° | Rigid, centre off, mm | With a scale, ° | Centre off, mm | Scale found |
| --- | ---: | ---: | ---: | ---: | ---: |
| 0.0 % larger | 0.04 | 0.1 | 0.05 | 0.2 | 0.999 |
| 0.5 % larger | 0.05 | 0.6 | 0.06 | 0.2 | 0.994 |
| 1.0 % larger | 0.07 | 1.2 | 0.06 | 0.2 | 0.989 |
| 2.0 % larger | 0.12 | 2.3 | 0.05 | 0.2 | 0.979 |
| 5.0 % larger | 0.23 | 5.8 | 0.05 | 0.1 | 0.951 |
| 10.0 % larger | 0.51 | 11.5 | 0.04 | 0.1 | 0.908 |
| bent by 1 mm | 0.06 | 0.3 | 0.07 | 0.4 | 0.999 |
| bent by 2 mm | 0.08 | 0.6 | 0.08 | 0.7 | 0.999 |
| bent by 4 mm | 0.11 | 1.2 | 0.12 | 1.2 | 0.999 |
| bent by 8 mm | 0.20 | 2.4 | 0.21 | 2.4 | 1.000 |
| bent by 16 mm | 0.35 | 4.9 | 0.40 | 4.8 | 1.001 |
