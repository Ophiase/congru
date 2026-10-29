import itertools
from dataclasses import dataclass
from typing import Final

import numpy as np
from numpy.typing import NDArray

type Points = NDArray[np.float64]
type Matrix = NDArray[np.float64]
type Triple = tuple[float, float, float]

GRID_STEP: Final = 0.08
BOX_SIDES: Final = (1.0, 1.6, 2.3)


def asymmetric_surface() -> Points:
    """Build a height field whose only rigid symmetry is the identity."""
    along, across = np.meshgrid(
        np.arange(-1.0, 1.0, GRID_STEP), np.arange(-0.6, 0.6, GRID_STEP)
    )
    height = np.sin(3 * along) * np.cos(2 * across) + 0.2 * across
    return np.column_stack((along.ravel(), across.ravel(), height.ravel()))


@dataclass(frozen=True)
class Mesh:
    vertices: Points
    triangles: NDArray[np.int64]


def box_mesh() -> Mesh:
    corners = np.array(tuple(itertools.product((0.0, 1.0), repeat=3)))
    faces = (
        (0, 1, 3), (0, 3, 2), (4, 6, 7), (4, 7, 5), (0, 4, 5), (0, 5, 1),
        (2, 3, 7), (2, 7, 6), (0, 2, 6), (0, 6, 4), (1, 5, 7), (1, 7, 3),
    )  # fmt: skip
    return Mesh(corners * BOX_SIDES, np.array(faces, dtype=np.int64))


def rigid_motion(axis_angle: Triple, shift: Triple) -> Matrix:
    """Build the `(4, 4)` motion turning by `|axis_angle|` radians, then shifting."""
    angle = float(np.linalg.norm(axis_angle))
    axis = np.asarray(axis_angle) / angle
    cross = np.array(
        [[0, -axis[2], axis[1]], [axis[2], 0, -axis[0]], [-axis[1], axis[0], 0]]
    )
    motion = np.eye(4)
    motion[:3, :3] = np.eye(3) + np.sin(angle) * cross
    motion[:3, :3] += (1 - np.cos(angle)) * cross @ cross
    motion[:3, 3] = shift
    return motion


def moved(points: Points, motion: Matrix) -> Points:
    return points @ motion[:3, :3].T + motion[:3, 3]


def rotation_error(estimate: Matrix, truth: Matrix) -> float:
    """Measure the angle, in radians, of the rotation between two motions."""
    relative = estimate[:3, :3].T @ truth[:3, :3]
    return float(np.arccos(np.clip((np.trace(relative) - 1) / 2, -1.0, 1.0)))


def translation_error(estimate: Matrix, truth: Matrix) -> float:
    return float(np.linalg.norm(estimate[:3, 3] - truth[:3, 3]))
