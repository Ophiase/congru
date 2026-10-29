# Sensor

**[defined]** A pinhole depth camera, simulated by **rasterising** the mesh
into a depth buffer: every triangle projected, every pixel keeping the
nearest depth. Linear in triangles and pixels, fast enough for hundreds of
frames on a laptop, and exact, so every error that follows comes from the
degradations, never from the renderer.

The renderer lives with the camera in `congru-vision`, not in the
simulator: the tracker uses it too, to know which part of the model a
pose shows.

**[question]** Resolution: 320×240 is the depth of a cheap sensor and
keeps the frames small.
