# What a batch of questions costs

Written by `just bench-hardware`: how long to ask how far each point is, for a
bracket of 20 cm. [Why it matters](../../ideas/architecture/acceleration.md).

The device here: NVIDIA GeForce GTX 960, with 12 CPU threads.

`GpuField` measures the machine when built; here, a batch
goes to the device from 100000 points.

| Points | Tree, all cores | Mesh, all cores | Field, one core | Field, all cores | Field, GPU |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1000 | 0.55 ms | 0.37 ms | 0.06 ms | 0.10 ms | 2.28 ms |
| 10000 | 18.27 ms | 15.55 ms | 0.64 ms | 12.41 ms | 3.49 ms |
| 100000 | 54.75 ms | 46.33 ms | 6.85 ms | 4.47 ms | 5.17 ms |
| 1000000 | 264.96 ms | 237.96 ms | 120.73 ms | 52.77 ms | 111.14 ms |
