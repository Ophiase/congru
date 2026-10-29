# Layers

**[sketch]** One core, thin skins. Nothing above the core holds an
algorithm.

```text
   Python (NumPy)     C / C++ header     CLI (files → JSON)
          │                  │                  │
          └──────────────────┼──────────────────┘
                             │  thin skins: convert, validate, call
                     ┌───────┴───────┐
                     │   core crate  │  geometry, search, fit, ICP, global
                     └───────┬───────┘  no I/O, no printing, no threads forced
                             │
                     ┌───────┴───────┐
                     │  synth crate  │  the observation model, the truth
                     └───────────────┘  tests and benchmark only
```

## Rules

- **[sketch]** The core reads no file and prints nothing: a robot has no
  terminal, a surgeon's headset no file system to spare.
- **[sketch]** A skin never re-implements a default: defaults live in the
  core and are read from it.
- **[sketch]** The synthetic generator is a crate of its own: the benchmark,
  the tests and the paper's figures share one source of truth.
- **[question]** File formats (PLY, XYZ) in the CLI only, or a small I/O
  crate the Python skin may also use?
