from itertools import product

import numpy as np

from draw import SEED
from fit import Pose, rotation_error, verdict
from scenes import LADDER, resting_shift, sensed

TURN_STEP = 5.0
SHIFT_STEP = 0.015
SHIFT_CELLS = 4
THROUGH_COST = 3
CLOSE_SHARE = 0.9


def main() -> None:
    print("| Step | Best pose by rays | Poses within 10 % of the best | Farthest of them |")
    print("| --- | --- | --- | --- |")
    for number, step in enumerate(LADDER, start=1):
        if step.around:
            continue
        _, observed = sensed(step, np.random.default_rng(SEED + number))
        truth = Pose(step.turned, resting_shift(step))
        turns = np.radians(np.arange(0.0, 360.0, TURN_STEP) + TURN_STEP / 2.0)
        shifts = (np.arange(-SHIFT_CELLS, SHIFT_CELLS + 1) + 0.5) * SHIFT_STEP
        scored = []
        for turn, across, along in product(turns, shifts, shifts):
            pose = Pose(truth.angle + float(turn), truth.shift + np.array([across, along]))
            found = verdict(pose, observed)
            scored.append((found.hits - THROUGH_COST * found.through, pose))
        best_score, best = max(scored, key=lambda scored_pose: scored_pose[0])
        close = [pose for score, pose in scored if score >= CLOSE_SHARE * best_score]
        farthest = max(rotation_error(pose, truth) for pose in close)
        offset = 1000.0 * float(np.linalg.norm(best.shift - truth.shift))
        print(
            f"| {number} {step.name} | {rotation_error(best, truth):.1f}°, {offset:.0f} mm "
            f"| {len(close)} of {len(scored)} | {farthest:.1f}° |",
            flush=True,
        )


if __name__ == "__main__":
    main()
