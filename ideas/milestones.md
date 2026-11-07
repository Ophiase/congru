# Milestones

**[defined]** Two milestones, the second one being why the first is worth
waiting for.

## M1 — The alignment, and a simulated robot that uses it

The library aligns, says how sure it is, and is measured against the truth.
It is shown on a **simulated video**: a machined part moves in front of a
depth camera, frames are degraded (noise, blur, holes), and the model is
tracked frame after frame, its outline drawn over the image, the pose error
plotted beside ([demo](demo/README.md)).

**[caution]** The code is not real-time yet: about 0.4 s per frame in a scene on
12 cores ([where it goes](architecture/acceleration.md)). The video is computed
offline, frame by frame, at whatever speed the laptop allows.

## M2 — The real world

The same pipeline on **real recordings**: an RGB-D camera, or a lidar as
the source of depth. Public datasets with a ground truth come first, so the
error stays measured; a recording of one's own desk comes next.

**[question]** Which datasets: a tabletop RGB-D benchmark with object
poses, and a driving or handheld lidar sequence. **[to check]** licences
before any file lands in `data/`.

## The road from 0.1 to a reference library

**[sketch]** In order; each a few small versions merged to `main`.

| Step | What | Done when | Today |
| --- | --- | --- | --- |
| **Scenes** | A scene description and a library of ordinary scenes: a part on a table, a car on a road, a shelf, a bin of parts; fractal ground noise; free meshes with their licence | Every situation of the benchmark also exists as a realistic scene, drawn on one sheet | Fourteen scenes on one sheet; not every situation has its scene |
| **Presentation** | One pastel theme for every figure and animation; demo B's background and outline fixed | The author reviews it without a remark | Films readable since 0.1.26; the author's review open |
| **Agents** | A `congru` command printing JSON (align, track a folder of frames, bench), a portable harness imported by `agent.just`, a skill an agent follows, a container image | An agent tracks an object in a recording with nothing but the skill | The command, without `bench`, and the harness; no skill, no image |
| **Credibility** | The same frames written to disk and run through Open3D and small_gicp; a Typst report with every table from the CSV | A reader can check every claim from a command | Tables laid out from a CSV; no comparison, no report |
| **Real data** | Public RGB-D and lidar recordings with ground truth and a clear licence | Milestone 2 | Not started |
| **Industry** | C ABI and C++ header, `f32` and `no_std`, latency percentiles, fuzzing of every input | A firmware team can link it and bound its time | C ABI and C++ header; the rest not started |
