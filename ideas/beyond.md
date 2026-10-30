# Beyond

**[guess]** Where congru could go once it is a good library, and why it
would matter. Written to be argued with, not obeyed.

## A machine that knows when it does not know

Most registration code returns a pose and a residual, and a small residual
looks like a right answer. The most valuable thing congru already does is
to say otherwise: which motions the data left free
([observability](method/observability.md)), and whether it trusts its pose.
The next step is a **certificate**: certifiable solvers ([TEASER++],
[Go-ICP]) prove a pose optimal under stated noise. A robot that hands a
scalpel, a drone that docks, an arm that grasps next to a person should act
only on a pose it can defend. Honesty about uncertainty is a safety
property, not a detail.

## A commons of hard cases

The benchmark is the part most likely to outlive the code. Grown into a
shared, open collection of scenes (each a short file, its licence
recorded, its truth known) it would let anyone, a student or a lab or a
company, add a method and see at once where it wins and where it breaks.
No paper claim without the scene that shows it; no default without the
table that chose it.

## Cheap enough for everyone

Registration should not need a GPU or a cloud. Written in Rust without the
standard library and in single precision ([embedded](architecture/embedded.md)),
it could run on the microcontrollers of low-cost robots: a farm robot
that finds a fruit, a prosthetic hand that sees the cup, a rescue crawler
in rubble, an assistive arm in a home. The people who need these machines
most are rarely the ones who can afford the hardware that today's
pipelines assume.

## Understood, not only used

Every number on these pages is drawn from a run that anyone can repeat
with one command, and every layer is explained in pictures
([the pipeline](../docs/pipeline.md)). A method a person can understand is
a method a person can trust, fix and teach.

## What it would take

1. Fix what is known broken: the realistic scenes that end turned over.
2. A real detector, then real recordings: milestone 2.
3. Certificates on the answers that matter.
4. The open scene commons, and a page that welcomes contributions.
5. The embedded build, measured on a real board.

[TEASER++]: https://arxiv.org/abs/2001.07715
[Go-ICP]: https://arxiv.org/abs/1605.03344
