# Scene

**[sketch]** A **machined bracket**, built in code from boxes and
cylinders: an L-shaped plate, a boss, two bores and a chamfered edge. Built
in code, it needs no download and no licence; asymmetric, it has one right
pose; with flat faces and round holes, it tests point-to-plane where it is
strong and where it slides.

The part follows a smooth path in front of the camera: a slow turn on a
turntable, then a pick-and-place arc, with the speed of a real arm. The
pose at every frame is kept: it is the truth the tracker is scored against.

**[caution]** A plane alone leaves three motions free
([observability](../method/observability.md)): the path must show the
moments when only one face is visible, and the tracker must say so.
