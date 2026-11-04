import json
import sys
from dataclasses import asdict
from pathlib import Path

import numpy as np
from scipy.ndimage import median_filter
from scipy.spatial.transform import Rotation

from camera import back_project, frame_depth, model_depth, nudge, sample_surface
from meshes import FloatArray, Pose, bracket
from scores import current_agreement, health_inliers, judge_points, rotation_error_degrees
from solver import along_rays, one_way_budget, rays_budget

REACH = 0.125
BUDGETS = (32, 64, 128, 256, 512)
RANDOM_STARTS = 60
MODEL_POINTS = 800
PROTRUSION = 0.012
PROTRUSION_WINDOW = 21
NUDGE_DEGREES = (3.0, 6.0, 9.0, 12.0)
NUDGES = 4
START_DEGREES = (5.0, 10.0, 15.0, 20.0, 30.0)
STARTS_PER_ANGLE = 4
SEARCH_FRAMES = 6
BASIN_FRAMES = 4
BASIN_SEED = 500
SEARCH_SEED = 700
Row = dict[str, object]


def protruding_points(depth: FloatArray) -> FloatArray:
    filled = np.nan_to_num(depth, nan=float(np.nanmax(depth)))
    surroundings = median_filter(filled, size=PROTRUSION_WINDOW)
    return back_project(np.where(surroundings - filled > PROTRUSION, depth, np.nan))


def random_starts(anchors: FloatArray, generator: np.random.Generator) -> list[Pose]:
    return [
        Pose(rotation, anchors[generator.integers(len(anchors))])
        for rotation in Rotation.random(RANDOM_STARTS, random_state=generator).as_matrix()
    ]


def visible_points(pose: Pose, points: FloatArray, normals: FloatArray) -> FloatArray:
    placed, turned = pose.apply(points), pose.turn(normals)
    return placed[(placed * turned).sum(axis=1) < 0.0]


def errors(pose: Pose, truth: Pose) -> Row:
    return {
        "rotation_error": rotation_error_degrees(pose.rotation, truth.rotation),
        "translation_error": float(np.linalg.norm(pose.translation - truth.translation)),
    }


def search_frame(spin: float, seed: int, with_can: bool, budget: int) -> dict[str, object]:
    generator = np.random.default_rng(seed + 9000 + budget)
    depth, truth = frame_depth(spin, seed, with_can)
    cloud = back_project(depth)
    mesh = bracket()
    points, normals = sample_surface(mesh, MODEL_POINTS, generator)
    standing = protruding_points(depth)
    labelled: list[tuple[str, float, Pose]] = [("truth", 0.0, truth)]
    labelled += [
        ("nudged", angle, nudge(truth, angle, generator))
        for angle in NUDGE_DEGREES
        for _ in range(NUDGES)
    ]
    for kind, anchors, refine in (
        ("one way, anywhere", cloud, "one way"),
        ("rays, anywhere", cloud, "rays"),
        ("rays, standing", standing, "rays"),
    ):
        for start in random_starts(anchors, generator):
            pose = (
                one_way_budget(start, cloud, points, normals, REACH, budget, generator)
                if refine == "one way"
                else rays_budget(start, depth, points, normals, budget, generator)
            )
            labelled.append((kind, 0.0, pose))
    rows = []
    for kind, angle, pose in labelled:
        verdict = judge_points(depth, pose, points, normals, budget, generator)
        rows.append(
            {
                "kind": kind,
                "nudge": angle,
                **errors(pose, truth),
                "inliers": health_inliers(
                    cloud, pose.translation, REACH, visible_points(pose, points, normals),
                    budget, generator,
                ),
                "agreement": current_agreement(depth, model_depth(mesh, pose)),
                "score": verdict.score,
                "outline_share": verdict.outline_share,
                "through_share": verdict.through_share,
                "hit_share": verdict.hit_share,
                **asdict(verdict),
            }
        )  # fmt: skip
    return {"with_can": with_can, "budget": budget, "rows": rows}


def basin_frame(spin: float, seed: int, with_can: bool, budget: int) -> list[Row]:
    generator = np.random.default_rng(seed + 5000 + budget)
    depth, truth = frame_depth(spin, seed, with_can)
    cloud = back_project(depth)
    mesh = bracket()
    points, normals = sample_surface(mesh, MODEL_POINTS, generator)
    results = []
    for degrees in START_DEGREES:
        for _ in range(STARTS_PER_ANGLE):
            start = nudge(truth, degrees, generator)
            for method, pose in (
                ("one way", one_way_budget(start, cloud, points, normals, REACH, budget, generator)),
                ("rays", rays_budget(start, depth, points, normals, budget, generator)),
                (
                    "rays, free space pushes",
                    rays_budget(start, depth, points, normals, budget, generator, through=True),
                ),
                *([("rays, every pixel", along_rays(start, depth, mesh, through=False))]
                  if budget == BUDGETS[-1] else []),
            ):
                results.append(
                    {"with_can": with_can, "budget": budget, "start": degrees, "method": method,
                     **errors(pose, truth)}
                )  # fmt: skip
    return results


def measure_basin(folder: Path) -> None:
    basins: list[Row] = []
    plan = [(i, can, k) for i in range(BASIN_FRAMES) for can in (False, True) for k in BUDGETS]
    for done, (index, can, budget) in enumerate(plan, start=1):
        spin = 2.0 * np.pi * index / BASIN_FRAMES
        basins.extend(basin_frame(spin, BASIN_SEED + index, can, budget))
        (folder / "basin.json").write_text(json.dumps(basins))
        print(f"basin {done} / {len(plan)}", flush=True)


def measure_search(folder: Path) -> None:
    searches: list[dict[str, object]] = []
    plan = [(i, can, k) for i in range(SEARCH_FRAMES) for can in (False, True) for k in BUDGETS]
    for done, (index, can, budget) in enumerate(plan, start=1):
        spin = 2.0 * np.pi * index / SEARCH_FRAMES
        searches.append(search_frame(spin, SEARCH_SEED + index, can, budget))
        (folder / "search.json").write_text(json.dumps(searches))
        print(f"search {done} / {len(plan)}", flush=True)


def main() -> None:
    folder = Path(sys.argv[1])
    folder.mkdir(parents=True, exist_ok=True)
    if sys.argv[2] in ("basin", "all"):
        measure_basin(folder)
    if sys.argv[2] in ("search", "all"):
        measure_search(folder)


if __name__ == "__main__":
    main()
