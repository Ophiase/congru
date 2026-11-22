# Acceleration

**[defined]** The author's rule: the project works on a CPU alone, and hardware
acceleration is a bonus when the machine has it. Speed is the big focus: several
kinds of hardware, possibly different algorithms on each, ready to use.

- **CPU by default**: no build, test or recipe may need a GPU.
- **A GPU is an optional crate**, falling back to the CPU when no device answers.
- **The same answer either way**: a test compares the two within a bound.

## What each brings

| Step | Followed frame, 12 cores | Why |
| --- | ---: | --- |
| Before | 395 ms | Normals fitted on every pixel, at every look |
| Normals when asked | 126 ms | A pose asks under its own silhouette |
| Rays and judgement on all cores | 110 ms | Most of a frame was on one |

An alignment alone, one thread, [same frames](../../docs/benchmark/comparison.md):

| Held as | Time |
| --- | ---: |
| Samples, to convergence at 1e-7 | 51 ms |
| Samples, stopped at 0.1° and 1 mm | 20.7 ms |
| A distance field | 3.7 ms |

## Choosing

**[sketch]** By what the caller has, not by a flag:

- **A mesh of the model**: a field. Its 250 ms are paid once.
- **Only points**: the k-d tree.
- **A few marks and time to spare**: [the certificate](../method/certificate.md).
- **One core**: the `parallel` feature off, the same answers.

## A GPU

**[sketch]** `congru-gpu` reads [the field](../method/representations.md) on a
device, [measured against each kind of hardware](../../docs/benchmark/hardware.md):

- **The field on one core beats the k-d tree on twelve**, 5 to 10 times.
- **On a GTX 960 the GPU never wins**: 78 ms a million points against 21.5 on
  twelve cores. The trip through memory costs more than the reads save.
- **So nothing is assumed**: `GpuField` measures the machine when built, and
  sends a batch to the device only from the size where it was faster.
- **[to check]** A recent card, and a pipeline that stays on the device instead
  of coming back at each iteration, are likely to change this.
- **Not rendering**: 0.18 ms a call there, about what a round trip costs.
