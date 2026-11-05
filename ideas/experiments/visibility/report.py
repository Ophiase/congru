import json
import sys
from collections.abc import Callable
from pathlib import Path

import numpy as np

RIGHT_ROTATION = 10.0
RIGHT_TRANSLATION = 0.03
MOST_THROUGH = 0.15
LEAST_IN_FRAME = 0.75
HEALTH_INLIERS = 0.5
BUDGETS = (32, 64, 128, 256, 512)
METHODS = ("one way", "rays", "rays, free space pushes", "rays, every pixel")
Row = dict[str, object]
Runs = list[dict[str, object]]


def is_right(row: Row) -> bool:
    return row["rotation_error"] <= RIGHT_ROTATION and row["translation_error"] <= RIGHT_TRANSLATION


def percent(rows: list[Row], test: Callable[[Row], bool]) -> str:
    return f"{100.0 * np.mean([test(row) for row in rows]):.0f} %"


def wins(runs: Runs, keep: Callable[[Row], bool], score: Callable[[Row], float]) -> str:
    won = sum(
        is_right(max((row for row in run["rows"] if keep(row)), key=score))
        for run in runs
    )
    return f"{won}/{len(runs)}"


def basin_table(results: list[Row]) -> None:
    starts = sorted({row["start"] for row in results})
    print("| Refinement | K | Can | " + " | ".join(f"from {s:g}°" for s in starts) + " |")
    print("| --- | --- | --- |" + " --- |" * len(starts))
    for method in METHODS:
        for budget in BUDGETS:
            for can in (False, True):
                group = [
                    row for row in results
                    if row["method"] == method and row["budget"] == budget and row["with_can"] == can
                ]  # fmt: skip
                if not group:
                    continue
                cells = []
                for start in starts:
                    angles = np.array(
                        [row["rotation_error"] for row in group if row["start"] == start]
                    )
                    cells.append(f"{np.median(angles):.1f}° ({100.0 * np.mean(angles <= 2.0):.0f} %)")
                print(f"| {method} | {budget} | {'yes' if can else 'no'} | " + " | ".join(cells) + " |")


def in_frame(row: Row) -> float:
    return 1.0 - row["silent"] / row["drawn"]


def agreement_in_frame(row: Row) -> float:
    return row["agreement"] * in_frame(row)


def refuted(row: Row) -> bool:
    return row["through_share"] > MOST_THROUGH


def judge_table(runs: Runs) -> None:
    judges = (
        ("one-way inliers", lambda row: row["inliers"]),
        ("agreement today", lambda row: row["agreement"]),
        ("agreement x in frame", agreement_in_frame),
        ("K points only", lambda row: row["score"]),
    )
    print("| K | Right pose offered | " + " | ".join(name for name, _ in judges) + " |")
    print("| --- | --- |" + " --- |" * len(judges))
    for budget in BUDGETS:
        chosen = [run for run in runs if run["budget"] == budget]
        for name, angle in (("exact", None), ("3° off", 3.0), ("6° off", 6.0), ("9° off", 9.0)):
            cells = [wins(chosen, offered(angle), score) for _, score in judges]
            print(f"| {budget} | {name} | " + " | ".join(cells) + " |")


def offered(angle: float | None) -> Callable[[Row], bool]:
    def keep(row: Row) -> bool:
        if row["kind"] == "one way, anywhere":
            return not is_right(row)
        if angle is None:
            return row["kind"] == "truth"
        return row["kind"] == "nudged" and row["nudge"] == angle

    return keep


def trust_table(runs: Runs) -> None:
    print("| K | Poses | Count | Health test trusts | Not refuted by rays | Not refuted, in frame |")
    print("| --- | --- | --- | --- | --- | --- |")
    for budget in BUDGETS:
        rows = [row for run in runs if run["budget"] == budget for row in run["rows"]]
        groups = (
            ("true", [row for run in runs if run["budget"] == budget and not run["with_can"] for row in run["rows"] if row["kind"] == "truth"]),
            ("true, can", [row for run in runs if run["budget"] == budget and run["with_can"] for row in run["rows"] if row["kind"] == "truth"]),
            ("3° off", [row for row in rows if row["kind"] == "nudged" and row["nudge"] == 3.0]),
            ("wrong", [row for row in rows if row["kind"] not in ("truth", "nudged") and not is_right(row)]),
        )
        for name, group in groups:
            cells = (
                percent(group, lambda row: row["inliers"] >= HEALTH_INLIERS),
                percent(group, lambda row: not refuted(row)),
                percent(group, lambda row: not refuted(row) and in_frame(row) >= LEAST_IN_FRAME),
            )
            print(f"| {budget} | {name} | {len(group)} | " + " | ".join(cells) + " |")


def hidden_shares(runs: Runs) -> None:
    print("| K | True pose | On a surface | In the shadow | In the void |")
    print("| --- | --- | --- | --- | --- |")
    for budget in BUDGETS:
        for can in (False, True):
            group = [
                row
                for run in runs
                if run["budget"] == budget and run["with_can"] == can
                for row in run["rows"]
                if row["kind"] == "truth"
            ]
            cells = [
                np.mean([row[key] / row["drawn"] for row in group])
                for key in ("hits", "shadow", "through")
            ]
            name = "can in front" if can else "alone"
            print(f"| {budget} | {name} | " + " | ".join(f"{100.0 * cell:.0f} %" for cell in cells) + " |")


def main() -> None:
    folder = Path(sys.argv[1])
    basin_table(json.loads((folder / "basin.json").read_text()))
    runs = json.loads((folder / "search.json").read_text())
    for table in (judge_table, trust_table, hidden_shares):
        print()
        table(runs)


if __name__ == "__main__":
    main()
