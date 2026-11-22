# Types

**[sketch]** A number with a meaning has a type; a misuse fails to compile,
or fails at the boundary, never deep in an iteration.

| Type | Guarantees |
| --- | --- |
| `PointCloud` | Non-empty, every coordinate finite |
| `RigidTransform` | A proper rotation (no reflection, no scale) and a translation |
| `Distance` | Finite and positive |
| `Fraction` | In $(0, 1]$ |
| `Kernel`, `Metric` | A closed set of choices: an enum, never a string |
| `Target` | A cloud with its index (and normals), built once, reused |

## [question] Frames

`RigidTransform<Source, Target>`, with phantom frame types, would make
composing two motions in the wrong order a compile error. Strong in Rust;
invisible in Python and C, where frames become strings or disappear.
**[guess]** worth it only if the Rust API is the main one.

## [caution]

Validation happens once, at construction. Inside the loop the types are
trusted; a check per iteration costs time a control loop does not have.
