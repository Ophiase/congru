# The demo — a simulated robot that tracks a part

**[sketch]** The proof of M1 ([milestones](../milestones.md)): a video in
which a 3D model is seen to sit exactly on a moving object, under worse and
worse sensing. Each stage is its own algorithm, so the video also shows
which stage breaks first.

```text
 scene ──► sensor ──► degradations ──► perception ──► tracking ──► video
 part and   depth      noise, blur,     depth image    pose per     outline
 its path   frames     holes, edges     to points      frame        over frame
```

| Stage | Idea |
| --- | --- |
| A part and its trajectory, the ground truth | [scene](scene.md) |
| A pinhole depth camera, by rasterising the mesh | [sensor](sensor.md) |
| What real sensors do to depth | [degradations](degradations.md) |
| From depth pixels to a point cloud | [perception](perception.md) |
| From one pose to the next | [tracking](tracking.md) |
| What the video shows | [video](video.md) |

**[defined]** No biological model: the object is a machined part.
