# Sensor

**[sketch]** A pinhole depth camera, simulated by **rasterising** the mesh
into a depth buffer: every triangle projected, every pixel keeping the
nearest depth. Linear in triangles and pixels, fast enough for hundreds of
frames on a laptop, and exact, so every error that follows comes from the
degradations, never from the renderer.

The same pass writes a shaded grey image, for the video only: the tracker
never sees it.

**[question]** Resolution: 320×240 is the depth of a cheap sensor and
keeps the frames small.
