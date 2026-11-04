from dataclasses import dataclass

import numpy as np
from numpy.typing import NDArray

FloatArray = NDArray[np.float64]

CIRCLE_SEGMENTS = 16


@dataclass(frozen=True)
class Pose:
    rotation: FloatArray
    translation: FloatArray

    def apply(self, points: FloatArray) -> FloatArray:
        return points @ self.rotation.T + self.translation

    def turn(self, vectors: FloatArray) -> FloatArray:
        return vectors @ self.rotation.T


def axis_rotation(axis: int, radians: float) -> FloatArray:
    cosine, sine = np.cos(radians), np.sin(radians)
    first, second = (axis + 1) % 3, (axis + 2) % 3
    rotation = np.eye(3)
    rotation[first, first] = rotation[second, second] = cosine
    rotation[first, second], rotation[second, first] = -sine, sine
    return rotation


def outward(triangles: FloatArray) -> FloatArray:
    crossed = np.cross(triangles[:, 1] - triangles[:, 0], triangles[:, 2] - triangles[:, 0])
    away = triangles.mean(axis=1) - triangles.mean(axis=(0, 1))
    inward = (crossed * away).sum(axis=1) < 0.0
    oriented = triangles.copy()
    oriented[inward, 1], oriented[inward, 2] = triangles[inward, 2], triangles[inward, 1]
    return oriented


def cuboid(low: tuple[float, float, float], high: tuple[float, float, float]) -> FloatArray:
    corners = np.array(
        [[(high if bit >> axis & 1 else low)[axis] for axis in range(3)] for bit in range(8)]
    )
    faces = [
        (0, 1, 3), (0, 3, 2), (4, 6, 7), (4, 7, 5),
        (0, 4, 5), (0, 5, 1), (2, 3, 7), (2, 7, 6),
        (0, 2, 6), (0, 6, 4), (1, 5, 7), (1, 7, 3),
    ]  # fmt: skip
    return outward(corners[np.array(faces)])


def cylinder(base: tuple[float, float, float], radius: float, height: float) -> FloatArray:
    angles = np.linspace(0.0, 2.0 * np.pi, CIRCLE_SEGMENTS + 1)
    rim = np.stack([np.cos(angles), np.sin(angles), np.zeros_like(angles)], axis=1) * radius
    low, high = rim + np.array(base), rim + np.array(base) + [0.0, 0.0, height]
    centre_low, centre_high = np.array(base), np.array(base) + [0.0, 0.0, height]
    triangles = []
    for segment in range(CIRCLE_SEGMENTS):
        following = segment + 1
        triangles += [
            (low[segment], low[following], high[following]),
            (low[segment], high[following], high[segment]),
            (centre_high, high[segment], high[following]),
            (centre_low, low[following], low[segment]),
        ]
    return outward(np.array(triangles))


def prism(profile: list[tuple[float, float]], start: float, end: float) -> FloatArray:
    corners = np.array(
        [[across, along, up] for along in (start, end) for across, up in profile]
    )
    faces = [
        (0, 1, 2), (3, 5, 4), (0, 3, 4), (0, 4, 1),
        (1, 4, 5), (1, 5, 2), (2, 5, 3), (2, 3, 0),
    ]  # fmt: skip
    return outward(corners[np.array(faces)])


def bracket() -> FloatArray:
    return np.concatenate(
        [
            cuboid((-0.10, -0.06, 0.0), (0.10, 0.06, 0.012)),
            cuboid((-0.10, -0.06, 0.0), (-0.088, 0.06, 0.09)),
            prism([(-0.088, 0.012), (-0.03, 0.012), (-0.088, 0.07)], 0.042, 0.054),
            cylinder((0.05, -0.02, 0.012), 0.022, 0.03),
            cylinder((0.075, 0.035, 0.012), 0.009, 0.012),
        ]
    )


def turntable() -> FloatArray:
    return cylinder((0.0, 0.0, -0.015), 0.2, 0.015)


def room() -> FloatArray:
    return np.concatenate(
        [
            cuboid((-0.9, -0.7, 0.75), (0.9, 0.7, 0.78)),
            cuboid((-0.9, 0.13, 0.1), (0.9, 0.16, 0.9)),
            cuboid((-0.28, 0.0, 0.42), (-0.18, 0.13, 0.52)),
            np.stack(
                [
                    cylinder((0.2, 0.3, 0.45), 0.035, 0.12)[:, :, axis]
                    for axis in (0, 2, 1)
                ],
                axis=2,
            ),
        ]
    )


def can_in_front(sideways: float) -> FloatArray:
    upright = cylinder((0.0, 0.0, -0.04), 0.035, 0.08)
    return upright[:, :, [0, 2, 1]] + np.array([sideways, 0.04, 0.3])


def turntable_pose(spin: float, sway: float = 0.0) -> Pose:
    look = axis_rotation(0, np.radians(28.0)) @ axis_rotation(0, np.pi / 2.0)
    return Pose(
        rotation=look @ axis_rotation(2, spin),
        translation=np.array([0.03 * np.sin(sway), 0.04, 0.42 + 0.04 * np.cos(2.0 * sway)]),
    )
