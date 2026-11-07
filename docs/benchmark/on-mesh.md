# Aligned on what the model is

Written by `just bench-on-mesh`, [why](../../ideas/method/representations.md). A
bracket scanned with about 1 mm of noise, from a guess 10° off, 16 seeds, one thread.

- **Error**: the mean distance of the scan's points to where they truly belong.
- **Time**: one alignment, with what it needs already built.
- **Built in**: making that once, per model.

| Model held as | Error, mm | Time, ms | Built in, ms | Bytes |
| --- | ---: | ---: | ---: | ---: |
| Samples every 12 mm | 0.318 | 13.97 | 0 | 45504 |
| Samples every 3 mm | 0.098 | 17.92 | 1 | 520320 |
| The mesh, exact | 0.094 | 43.60 | 0 | 25344 |
| A distance field, 2 mm | 0.083 | 5.88 | 252 | 4623696 |
