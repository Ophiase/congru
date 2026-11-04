import sys
from dataclasses import dataclass
from pathlib import Path

import matplotlib.pyplot as plt
import numpy as np
from matplotlib.lines import Line2D
from matplotlib.patches import Polygon
from numpy.typing import NDArray

Points = NDArray[np.float64]

INK = "#2b2d33"
OBJECT_FILL = "#d6dbe6"
EMPTY_FILL = "#e4f2ea"
SHADOW_FILL = "#e3dcf0"
HIT = "#128a5b"
SHADOW = "#7a56b0"
THROUGH = "#d6332b"
CAMERA = np.array([0.0, 0.55])
TOLERANCE = 0.012
MODEL_POINTS = 14
SHOWN_RAYS = 9
FAN_LOW, FAN_HIGH = np.radians(-42.0), np.radians(4.0)

TABLE = np.array([(0.15, -0.06), (1.05, -0.06), (1.05, 0.0), (0.15, 0.0)])
WALL = np.array([(1.3, -0.5), (1.36, -0.5), (1.36, 0.9), (1.3, 0.9)])
FLOOR = np.array([(-0.1, -0.56), (1.36, -0.56), (1.36, -0.5), (-0.1, -0.5)])
CAN = np.array([(0.26, 0.2), (0.34, 0.2), (0.34, 0.32), (0.26, 0.32)])
PART = np.array(
    [(0.0, 0.0), (0.26, 0.0), (0.26, 0.05), (0.08, 0.05), (0.08, 0.2), (0.0, 0.2)]
)


@dataclass(frozen=True)
class Scene:
    solids: tuple[Points, ...]

    def distance_along(self, direction: Points) -> float:
        return min(ray_hit(direction, solid) for solid in self.solids)


def placed(outline: Points, along: float, up: float, turned: bool = False) -> Points:
    shape = outline * np.array([-1.0, -1.0]) + np.array([0.26, 0.2]) if turned else outline
    return shape + np.array([along, up])


def ray_hit(direction: Points, polygon: Points) -> float:
    start = polygon - CAMERA
    edge = np.roll(polygon, -1, axis=0) - polygon
    denominator = direction[0] * edge[:, 1] - direction[1] * edge[:, 0]
    with np.errstate(divide="ignore", invalid="ignore"):
        distance = (start[:, 0] * edge[:, 1] - start[:, 1] * edge[:, 0]) / denominator
        position = (start[:, 0] * direction[1] - start[:, 1] * direction[0]) / denominator
    valid = (np.abs(denominator) > 1e-12) & (distance > 1e-6) & (position >= 0.0) & (position <= 1.0)
    return float(distance[valid].min()) if valid.any() else np.inf


def perimeter_points(outline: Points, count: int) -> Points:
    closed = np.vstack([outline, outline[:1]])
    lengths = np.linalg.norm(np.diff(closed, axis=0), axis=1)
    stations = np.concatenate([[0.0], np.cumsum(lengths)])
    wanted = (np.arange(count) + 0.5) * stations[-1] / count
    return np.column_stack(
        [np.interp(wanted, stations, closed[:, axis]) for axis in (0, 1)]
    )


def verdict(point: Points, scene: Scene) -> str:
    offset = point - CAMERA
    distance = float(np.linalg.norm(offset))
    gap = scene.distance_along(offset / distance) - distance
    if gap > TOLERANCE:
        return THROUGH
    if gap < -TOLERANCE:
        return SHADOW
    return HIT


def directions() -> Points:
    angles = np.linspace(FAN_LOW, FAN_HIGH, 400)
    return np.column_stack([np.cos(angles), np.sin(angles)])


def stage(axis: plt.Axes, scene: Scene, shade: bool = True) -> None:
    if shade:
        fan = directions()
        lengths = np.array([scene.distance_along(ray) for ray in fan])
        ends = CAMERA + fan * lengths[:, None]
        axis.add_patch(Polygon(np.vstack([CAMERA, ends]), facecolor=EMPTY_FILL, edgecolor="none", zorder=0))
    for solid in scene.solids:
        axis.add_patch(Polygon(solid, facecolor=OBJECT_FILL, edgecolor=INK, linewidth=1.4, zorder=2))
    axis.plot(*CAMERA, marker="s", color=INK, markersize=17, zorder=6)
    axis.text(CAMERA[0], CAMERA[1] + 0.07, "camera", fontsize=18, ha="center", color=INK)
    axis.set_xlim(-0.12, 1.4)
    axis.set_ylim(-0.6, 0.72)
    axis.set_aspect("equal")
    axis.axis("off")


def shadow_region(axis: plt.Axes, scene: Scene, blockers: Scene) -> None:
    fan = directions()
    behind = np.array([blockers.distance_along(ray) for ray in fan])
    far = np.array([Scene(tuple(s for s in scene.solids if s is not blockers.solids[0])).distance_along(ray) for ray in fan])
    runs = np.flatnonzero(np.isfinite(behind) & (behind < far - 0.02))
    if len(runs) == 0:
        return
    near_edge = CAMERA + fan[runs] * behind[runs, None]
    far_edge = CAMERA + fan[runs] * far[runs, None]
    axis.add_patch(
        Polygon(np.vstack([near_edge, far_edge[::-1]]), facecolor=SHADOW_FILL, edgecolor="none", zorder=1)
    )


def one_ray(target: Path) -> None:
    figure, axis = plt.subplots(figsize=(14.0, 4.2))
    start, surface, end = 0.0, 0.62, 1.0
    axis.plot([start, surface], [0.0, 0.0], color=HIT, linewidth=7.0, solid_capstyle="butt")
    axis.plot([surface, end], [0.0, 0.0], color=SHADOW, linewidth=4.0, linestyle=(0, (2, 2)))
    axis.plot(start, 0.0, marker="s", color=INK, markersize=24)
    axis.plot(surface, 0.0, marker="o", color=INK, markersize=22, zorder=5)
    axis.text(start, 0.11, "camera", fontsize=22, ha="center", color=INK)
    axis.text(surface, 0.11, "the measure", fontsize=22, ha="center", color=INK)
    axis.text(0.31, -0.17, "EMPTY\nthe ray crossed it: nothing is here", fontsize=20, ha="center", color=HIT)
    axis.text(surface, -0.32, "SOMETHING\nis here", fontsize=20, ha="center", color=INK)
    axis.text(0.86, -0.17, "UNKNOWN\nthe ray stopped: no clue", fontsize=20, ha="center", color=SHADOW)
    axis.set_xlim(-0.08, 1.06)
    axis.set_ylim(-0.42, 0.22)
    axis.axis("off")
    figure.savefig(target, dpi=110, bbox_inches="tight")
    plt.close(figure)


def whole_scene(target: Path) -> None:
    part = placed(PART, 0.5, 0.0)
    scene = Scene((part, TABLE, WALL, FLOOR))
    figure, axis = plt.subplots(figsize=(14.0, 9.0))
    stage(axis, scene)
    shadow_region(axis, scene, Scene((part,)))
    for angle in np.linspace(FAN_LOW, FAN_HIGH, SHOWN_RAYS):
        ray = np.array([np.cos(angle), np.sin(angle)])
        hit = CAMERA + ray * scene.distance_along(ray)
        axis.plot(*np.column_stack([CAMERA, hit]), color=HIT, linewidth=1.4, zorder=3)
        axis.plot(*hit, marker="o", color=INK, markersize=15, zorder=5)
    axis.text(0.58, 0.27, "the part", fontsize=20, color=INK, ha="center")
    axis.text(1.2, 0.72, "wall", fontsize=20, color=INK)
    axis.text(0.28, -0.14, "table", fontsize=20, color=INK)
    axis.annotate(
        "its shadow: having no point\nhere is normal", xy=(0.68, 0.07), xytext=(0.62, -0.32),
        fontsize=20, color=SHADOW, arrowprops={"arrowstyle": "->", "color": SHADOW, "linewidth": 2.0},
    )  # fmt: skip
    figure.savefig(target, dpi=110, bbox_inches="tight")
    plt.close(figure)


def judged(axis: plt.Axes, title: str, scene: Scene, outline: Points) -> None:
    stage(axis, scene, shade=False)
    axis.add_patch(Polygon(outline, facecolor="none", edgecolor=INK, linestyle="--", linewidth=2.0, zorder=3))
    points = perimeter_points(outline, MODEL_POINTS)
    colours = [verdict(point, scene) for point in points]
    for point, colour in zip(points, colours, strict=True):
        axis.plot(*point, marker="o", color=colour, markersize=16, markeredgecolor="white", zorder=5)
    tally = (
        f"on a surface {colours.count(HIT)}   ·   in the shadow {colours.count(SHADOW)}"
        f"   ·   in the void {colours.count(THROUGH)}"
    )
    axis.set_title(title, fontsize=23, color=INK, loc="left", pad=4.0)
    axis.text(0.64, -0.59, tally, fontsize=17, color=INK, ha="center", va="top")


def verdicts(target: Path) -> None:
    true_part = placed(PART, 0.5, 0.0)
    world = Scene((true_part, TABLE, WALL, FLOOR))
    with_can = Scene((true_part, TABLE, WALL, FLOOR, CAN))
    figure, axes = plt.subplots(2, 2, figsize=(17.0, 12.5))
    judged(axes[0, 0], "Right place: half is in its shadow, no cost", world, true_part)
    judged(axes[0, 1], "A can in front: hidden is not refuted", with_can, true_part)
    judged(axes[1, 0], "In the air: rays cross it, refuted", world, placed(PART, 0.42, 0.14, turned=True))
    judged(axes[1, 1], "Under the table: not refuted, not supported", world, placed(PART, 0.42, -0.3))
    figure.legend(
        handles=[
            Line2D([], [], marker="o", linestyle="", color=HIT, markersize=19, label="model point on a surface"),
            Line2D([], [], marker="o", linestyle="", color=SHADOW, markersize=19, label="in the shadow: no cost"),
            Line2D([], [], marker="o", linestyle="", color=THROUGH, markersize=19, label="in the void: refutes the pose"),
        ],
        loc="lower center", ncols=3, fontsize=20, frameon=False,
    )  # fmt: skip
    figure.tight_layout(rect=(0.0, 0.05, 1.0, 1.0))
    figure.savefig(target, dpi=100, bbox_inches="tight")
    plt.close(figure)


def main() -> None:
    folder = Path(sys.argv[1])
    one_ray(folder / "visibility-ray.png")
    whole_scene(folder / "visibility-scene.png")
    verdicts(folder / "visibility-verdicts.png")


if __name__ == "__main__":
    main()
