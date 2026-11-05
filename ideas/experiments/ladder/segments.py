"""Read a cloud as segments, as a person does, and place the model on them."""

from dataclasses import dataclass

import numpy as np
from numpy.typing import NDArray

from fit import Pose
from rays import along_rays, polish, read
from scenes import BRACKET, CAMERA, Points, ray_directions

JUMP = 0.025
SPLIT = 0.012
LEAST_RETURNS = 4
LENGTH_SLACK = 0.02
SLIDE_STEP = 0.01
KEPT = 24
POLISHED = 3


@dataclass(frozen=True)
class Segment:
    start: Points
    end: Points

    @property
    def length(self) -> float:
        return float(np.linalg.norm(self.end - self.start))


def runs_of_rays(observed: NDArray[np.float64]) -> list[Points]:
    """Neighbouring rays at close depths: a stray, alone on its ray, is a run of one."""
    points = CAMERA + ray_directions() * observed[:, None]
    runs, current = [], []
    for ray in range(len(observed)):
        if not np.isfinite(observed[ray]):
            continue
        if current and np.linalg.norm(points[ray] - current[-1]) > JUMP:
            runs.append(np.array(current))
            current = []
        current.append(points[ray])
    if current:
        runs.append(np.array(current))
    return runs


def split(run: Points) -> list[Points]:
    """Cut a run where it leaves the chord between its ends."""
    chord = run[-1] - run[0]
    norm = float(np.linalg.norm(chord))
    if len(run) < 3 or norm == 0.0:
        return [run]
    offset, unit = run - run[0], chord / norm
    away = np.abs(offset[:, 0] * unit[1] - offset[:, 1] * unit[0])
    worst = int(away.argmax())
    if away[worst] <= SPLIT or worst in (0, len(run) - 1):
        return [run]
    return split(run[: worst + 1]) + split(run[worst:])


def segments_of(observed: NDArray[np.float64]) -> list[Segment]:
    found = []
    for run in runs_of_rays(observed):
        for piece in split(run):
            if len(piece) < LEAST_RETURNS:
                continue
            centre = piece.mean(axis=0)
            along = np.linalg.svd(piece - centre)[2][0]
            reach = (piece - centre) @ along
            found.append(Segment(centre + along * reach.min(), centre + along * reach.max()))
    return found


def placements(seen: Segment, edge_start: Points, edge_end: Points) -> list[Pose]:
    """Every pose laying this model edge on this segment, its outside to the camera."""
    edge = edge_end - edge_start
    edge_length = float(np.linalg.norm(edge))
    if seen.length > edge_length + LENGTH_SLACK:
        return []
    outward = np.array([edge[1], -edge[0]]) / edge_length
    centre = (seen.start + seen.end) / 2.0
    to_camera = (CAMERA - centre) / np.linalg.norm(CAMERA - centre)
    along_seen = (seen.end - seen.start) / max(seen.length, 1e-9)
    facing = np.array([along_seen[1], -along_seen[0]])
    facing = facing if facing @ to_camera > 0.0 else -facing
    angle = float(np.arctan2(facing[1], facing[0]) - np.arctan2(outward[1], outward[0]))
    turn = np.array([[np.cos(angle), -np.sin(angle)], [np.sin(angle), np.cos(angle)]])
    middle = turn @ ((edge_start + edge_end) / 2.0)
    direction = turn @ (edge / edge_length)
    room = max(0.0, (edge_length - seen.length) / 2.0)
    slides = np.arange(-room, room + SLIDE_STEP, SLIDE_STEP) if room > 0.0 else np.zeros(1)
    return [Pose(angle, centre + direction * slide - middle) for slide in slides]


def hypotheses(found: list[Segment]) -> list[Pose]:
    ends = np.roll(BRACKET, -1, axis=0)
    return [
        pose
        for seen in found
        for start, end in zip(BRACKET, ends, strict=True)
        for pose in placements(seen, start, end)
    ]


def find(observed: NDArray[np.float64]) -> tuple[Pose | None, int]:
    """No start: every placement is judged by the rays, the best are refined."""
    candidates = hypotheses(segments_of(observed))
    if not candidates:
        return None, 0
    candidates.sort(key=lambda pose: read(pose, observed).value, reverse=True)
    brought = [along_rays(pose, observed) for pose in candidates[:KEPT]]
    brought.sort(key=lambda pose: read(pose, observed).value, reverse=True)
    polished = [polish(pose, observed) for pose in brought[:POLISHED]]
    return max(polished, key=lambda pose: read(pose, observed).value), len(candidates)
