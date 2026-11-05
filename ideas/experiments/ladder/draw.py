import sys
from collections.abc import Callable
from pathlib import Path
from statistics import median

import matplotlib.pyplot as plt
import numpy as np

from fit import Pose, Verdict, model_to_scan, rotation_error, scan_to_model, starts, verdict
from rays import by_rays
from scenes import BRACKET, CAMERA, LADDER, Points, Step, placed, resting_shift, sensed

INK = "#2b2d33"
TRUTH_COLOUR = "#0f8a6b"
NAIVE_COLOUR = "#c2571a"
SCAN_COLOUR = "#3f6fb5"
RAYS_COLOUR = "#7a3fb5"
OBSTACLE_COLOUR = "#d9d6cc"
SEED = 20261104

Method = Callable[[Points, Pose], Pose]


def closed(polygon: Points) -> Points:
    return np.vstack((polygon, polygon[:1]))


def median_run(scan: Points, truth: Pose, method: Method) -> tuple[Pose, float, float]:
    runs = [method(scan, start) for start in starts(truth)]
    runs.sort(key=lambda pose: rotation_error(pose, truth))
    middle = runs[len(runs) // 2]
    return (
        middle,
        median(rotation_error(pose, truth) for pose in runs),
        median(1000.0 * float(np.linalg.norm(pose.shift - truth.shift)) for pose in runs),
    )


def framed(axis: plt.Axes, step: Step, scan: Points) -> None:
    for polygon in step.others:
        axis.fill(*closed(polygon).T, color=OBSTACLE_COLOUR, zorder=1)
    axis.scatter(*scan.T, s=52, color=SCAN_COLOUR, zorder=3)
    if not step.around:
        axis.scatter(*CAMERA, s=150, marker="v", color=INK, zorder=4)
    axis.set_xlim(-0.46, 0.46)
    axis.set_ylim(-0.12, 0.64)
    axis.set_aspect("equal")
    axis.axis("off")


def figure_of(step: Step, scan: Points, truth: Pose, fits: tuple[Pose, float, Pose | None, float], target: Path) -> None:
    naive, error, rays, rays_error = fits
    figure, (given, answer) = plt.subplots(1, 2, figsize=(11.0, 4.8))
    framed(given, step, scan)
    given.set_title("What the sensor gives", loc="left", color=INK, fontsize=16)
    framed(answer, step, scan)
    answer.plot(*closed(placed(BRACKET, truth.angle, truth.shift)).T, color=TRUTH_COLOUR, linewidth=3.0,
                label="truth", zorder=5)  # fmt: skip
    answer.plot(*closed(placed(BRACKET, naive.angle, naive.shift)).T, color=NAIVE_COLOUR, linewidth=3.0,
                linestyle="--", label=f"naive fit, {error:.0f}° off", zorder=6)  # fmt: skip
    if rays is not None:
        answer.plot(*closed(placed(BRACKET, rays.angle, rays.shift)).T, color=RAYS_COLOUR, linewidth=3.0,
                    linestyle=":", label=f"along rays, {rays_error:.0f}° off", zorder=7)  # fmt: skip
    answer.legend(loc="upper right", frameon=False, fontsize=13)
    answer.set_title("The truth and the fits", loc="left", color=INK, fontsize=16)
    figure.tight_layout(pad=1.6)
    figure.savefig(target, dpi=110)
    plt.close(figure)


def cell(found: Verdict | None) -> str:
    return "—" if found is None else f"{found.hits} / {found.through} / {found.hidden}"


def main(target: Path) -> None:
    print("| Step | Scan to model | Model to scan | Along rays | Truth: hit / through / hidden | Naive: same |")
    print("| --- | --- | --- | --- | --- | --- |")
    for number, step in enumerate(LADDER, start=1):
        generator = np.random.default_rng(SEED + number)
        scan, observed = sensed(step, generator)
        truth = Pose(step.turned, resting_shift(step))
        naive, error, offset = median_run(scan, truth, scan_to_model)
        _, reverse, reverse_offset = median_run(scan, truth, model_to_scan)
        rays, rays_error, rays_offset = (None, 0.0, 0.0)
        if observed is not None:
            known = observed
            rays, rays_error, rays_offset = median_run(scan, truth, lambda _, start: by_rays(start, known))
        figure_of(step, scan, truth, (naive, error, rays, rays_error), target / f"ladder-{number}-{step.name}.png")
        truth_verdict = None if observed is None else verdict(truth, observed)
        naive_verdict = None if observed is None else verdict(naive, observed)
        print(f"| {number} {step.name} | {error:.1f}°, {offset:.0f} mm | {reverse:.1f}°, {reverse_offset:.0f} mm | {"—" if rays is None else f"{rays_error:.1f}°, {rays_offset:.0f} mm"} | {cell(truth_verdict)} | {cell(naive_verdict)} |")


if __name__ == "__main__":
    main(Path(sys.argv[1]))
