# Milestones

**[defined]** Two milestones, the second one being why the first is worth
waiting for.

## M1 — The alignment, and a simulated robot that uses it

The library aligns, says how sure it is, and is measured against the truth.
It is shown on a **simulated video**: a machined part moves in front of a
depth camera, frames are degraded (noise, blur, holes), and the model is
tracked frame after frame, its outline drawn over the image, the pose error
plotted beside ([demo](demo/README.md)).

The code is real-time by design; the video is computed offline, frame by
frame, at whatever speed the laptop allows.

## M2 — The real world

The same pipeline on **real recordings**: an RGB-D camera, or a lidar as
the source of depth. Public datasets with a ground truth come first, so the
error stays measured; a recording of one's own desk comes next.

**[question]** Which datasets: a tabletop RGB-D benchmark with object
poses, and a driving or handheld lidar sequence. **[to check]** licences
before any file lands in `data/`.
