# Degradations

**[sketch]** What real depth sensors do, each a filter on the depth image,
each with one strength parameter, so the video can turn one up at a time.

| Filter | What it imitates |
| --- | --- |
| **Gaussian noise**, growing with the square of depth | Stereo and time-of-flight noise |
| **Blur** | Optics, motion, the averaging of a structured-light sensor |
| **Holes** | Dark or shiny surfaces that return nothing |
| **Flying pixels** at depth edges | Mixed pixels between foreground and background |
| **Quantisation** | The integer millimetres of a cheap sensor |
| **Clutter** | A table and a hand: outliers that sit next to the part |

**[guess]** Clutter and holes break tracking first; noise is absorbed by
the robust kernel.
