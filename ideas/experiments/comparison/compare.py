"""Run other libraries on the frames congru wrote, and write the page."""

import sys
import time
from collections import defaultdict
from dataclasses import dataclass
from pathlib import Path
from typing import Final

import numpy as np
import open3d
import small_gicp

ROTATION_TOLERANCE: Final = 2.0
TRANSLATION_TOLERANCE: Final = 0.02
SIZE: Final = 0.2
GATE: Final = 0.03
ITERATIONS: Final = 50
VOXEL: Final = 1e-4
NEIGHBORS: Final = 10
INTRO: Final = """# Compared on the same frames

Written by `just comparison`. Same frames, same 3 cm gate, at most 50 iterations,
one thread, each library at its own defaults otherwise, not tuned; a bracket, 16
seeds per line.

- **Success**: within 2° and 2 % of the size of the truth.
- **Median time**: the alignment alone, its target already prepared.
- **Stopped as small_gicp**: congru's default stops at steps of 1e-7, much later
  than the others; this column stops at 0.1° and 1 mm.
- **Mean over all**: the last line weighs each situation the same.

"""


@dataclass(frozen=True)
class Outcome:
    success: bool
    degrees: float
    seconds: float


def pose(text: str) -> np.ndarray:
    return np.array([float(entry) for entry in text.split()]).reshape(4, 4)


def judged(found: np.ndarray, truth: np.ndarray, seconds: float) -> Outcome:
    relative = found[:3, :3].T @ truth[:3, :3]
    degrees = np.degrees(np.arccos(np.clip((np.trace(relative) - 1) / 2, -1.0, 1.0)))
    shift = float(np.linalg.norm(found[:3, 3] - truth[:3, 3]))
    good = degrees < ROTATION_TOLERANCE and shift < TRANSLATION_TOLERANCE * SIZE
    return Outcome(bool(good), float(degrees), seconds)


class Open3d:
    name: Final = "Open3D, point to plane"

    def __init__(self, target: np.ndarray) -> None:
        self.target = open3d.geometry.PointCloud()
        self.target.points = open3d.utility.Vector3dVector(target[:, :3])
        self.target.normals = open3d.utility.Vector3dVector(target[:, 3:])

    def align(self, source: np.ndarray, guess: np.ndarray) -> tuple[np.ndarray, float]:
        cloud = open3d.geometry.PointCloud()
        cloud.points = open3d.utility.Vector3dVector(source)
        started = time.perf_counter()
        result = open3d.pipelines.registration.registration_icp(
            cloud,
            self.target,
            GATE,
            guess,
            open3d.pipelines.registration.TransformationEstimationPointToPlane(),
            open3d.pipelines.registration.ICPConvergenceCriteria(
                max_iteration=ITERATIONS
            ),
        )
        return np.asarray(result.transformation), time.perf_counter() - started


class SmallGicp:
    def __init__(self, target: np.ndarray, kind: str, name: str) -> None:
        self.kind = kind
        self.name = name
        self.target, self.tree = small_gicp.preprocess_points(
            target[:, :3],
            downsampling_resolution=VOXEL,
            num_neighbors=NEIGHBORS,
            num_threads=1,
        )

    def align(self, source: np.ndarray, guess: np.ndarray) -> tuple[np.ndarray, float]:
        cloud, _ = small_gicp.preprocess_points(
            source,
            downsampling_resolution=VOXEL,
            num_neighbors=NEIGHBORS,
            num_threads=1,
        )
        started = time.perf_counter()
        result = small_gicp.align(
            self.target,
            cloud,
            self.tree,
            init_T_target_source=guess,
            registration_type=self.kind,
            max_correspondence_distance=GATE,
            num_threads=1,
            max_iterations=ITERATIONS,
        )
        return np.asarray(result.T_target_source), time.perf_counter() - started


def table(outcomes: dict[str, dict[str, list[Outcome]]]) -> str:
    methods = list(next(iter(outcomes.values())))
    head = "| Situation | " + " | ".join(methods) + " |\n"
    rule = "| --- |" + " ---: |" * len(methods) + "\n"
    rows = []
    for situation, of in outcomes.items():
        cells = []
        for method in methods:
            done = of[method]
            rate = 100 * np.mean([one.success for one in done])
            median = np.median([one.degrees for one in done])
            cells.append(f"{rate:.0f} %, {median:.2f}°")
        name = situation.replace("_", " ")
        rows.append(f"| {name} | " + " | ".join(cells) + " |\n")
    summary = []
    for method in methods:
        done = [one for of in outcomes.values() for one in of[method]]
        rate = 100 * np.mean([one.success for one in done])
        milliseconds = 1e3 * np.median([one.seconds for one in done])
        summary.append(f"{rate:.0f} %, {milliseconds:.1f} ms")
    rows.append("| **Mean over all, median time** | " + " | ".join(summary) + " |\n")
    return head + rule + "".join(rows)


def main(folder: Path, page: Path) -> None:
    target = np.loadtxt(folder / "target.csv", delimiter=",")
    others = (
        Open3d(target),
        SmallGicp(target, "PLANE_ICP", "small_gicp, point to plane"),
        SmallGicp(target, "GICP", "small_gicp, GICP"),
    )
    outcomes: dict[str, dict[str, list[Outcome]]] = defaultdict(lambda: defaultdict(list))
    for line in (folder / "trials.csv").read_text().splitlines()[1:]:
        situation, seed, seconds, guess, truth, found, early_seconds, early = line.split(
            ","
        )
        guess_pose, truth_pose = pose(guess), pose(truth)
        source = np.loadtxt(folder / f"source-{situation}-{seed}.csv", delimiter=",")
        of = outcomes[situation]
        of["congru, point to plane"].append(
            judged(pose(found), truth_pose, float(seconds))
        )
        of["congru, stopped as small_gicp"].append(
            judged(pose(early), truth_pose, float(early_seconds))
        )
        for other in others:
            answer, spent = other.align(source, guess_pose)
            of[other.name].append(judged(answer, truth_pose, spent))
    page.write_text(INTRO + table(outcomes))


if __name__ == "__main__":
    main(Path(sys.argv[1]), Path(sys.argv[2]))
