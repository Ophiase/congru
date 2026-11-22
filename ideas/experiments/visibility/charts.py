import json
import sys
from pathlib import Path

import matplotlib.pyplot as plt
import numpy as np


INK = "#2b2d33"
BUDGET_SHOWN = 256
METHOD_COLOURS = {
    "one way": "#3f6fb5",
    "rays": "#0f8a6b",
    "rays, free space pushes": "#c2571a",
}


def medians(results: list[dict[str, object]], method: str, can: bool, starts: list[float]) -> list[float]:
    return [
        float(
            np.median(
                [
                    row["rotation_error"]
                    for row in results
                    if row["method"] == method
                    and row["with_can"] == can
                    and row["start"] == start
                    and row["budget"] == BUDGET_SHOWN
                ]
            )
        )
        for start in starts
    ]


def basin_figure(results: list[dict[str, object]], target: Path) -> None:
    starts = sorted({row["start"] for row in results})
    figure, axes = plt.subplots(1, 2, figsize=(14.0, 5.2), sharey=True)
    for axis, can, title in zip(
        axes, (False, True), ("In the room", "In the room, a can in front"), strict=True
    ):
        for method, colour in METHOD_COLOURS.items():
            axis.plot(starts, medians(results, method, can, starts), color=colour,
                      linewidth=2.6, marker="o", markersize=9, label=method)  # fmt: skip
        axis.set_title(title, loc="left", color=INK, fontsize=17)
        axis.set_xlabel("how far the start is from the truth, degrees", color=INK, fontsize=14)
        axis.spines[["top", "right"]].set_visible(False)
        axis.grid(axis="y", color="#e4e2da", linewidth=0.8)
        axis.set_yscale("log")
        axis.tick_params(labelsize=12)
    axes[0].set_ylabel(f"median rotation error, degrees, K = {BUDGET_SHOWN}", color=INK, fontsize=14)
    axes[0].legend(frameon=False, fontsize=14)
    figure.tight_layout()
    figure.savefig(target, dpi=120)
    plt.close(figure)


def kernels(target: Path) -> None:
    gap = np.linspace(-8.0, 8.0, 800)
    hit = np.exp(-0.5 * (gap / 0.8) ** 2)
    unsigned = 4.4 * hit
    signed = 4.4 * hit - 0.4 * (gap < 0.0) * (1.0 - hit) - 4.6 * (gap > 0.0) * (1.0 - hit)
    figure, axes = plt.subplots(1, 2, figsize=(14.0, 5.0), sharey=True)
    axes[0].plot(gap, unsigned, color="#3f6fb5", linewidth=3.0)
    axes[0].set_title("A: a distance. Far is far, either way.", loc="left", fontsize=17)
    axes[0].set_xlabel("distance to the model, cm", fontsize=14)
    axes[0].set_ylabel("evidence for the pose (shape only)", fontsize=14)
    axes[1].plot(gap, signed, color=INK, linewidth=3.0)
    axes[1].axvspan(-8.0, -2.0, color="#7a56b0", alpha=0.12)
    axes[1].axvspan(2.0, 8.0, color="#d6332b", alpha=0.12)
    axes[1].text(-5.0, 2.6, "in the shadow\nno cost to speak of", ha="center", fontsize=14)
    axes[1].text(5.0, 2.6, "in empty space\nrefuted", ha="center", fontsize=14)
    axes[1].set_title("B: a signed gap along the ray.", loc="left", fontsize=17)
    axes[1].set_xlabel("measured depth minus the model point's depth, cm", fontsize=14)
    for axis in axes:
        axis.axhline(0.0, color=INK, linewidth=0.7)
        axis.spines[["top", "right"]].set_visible(False)
        axis.tick_params(labelsize=12)
    figure.tight_layout()
    figure.savefig(target, dpi=120, bbox_inches="tight")
    plt.close(figure)


def main() -> None:
    measured, figures = Path(sys.argv[1]), Path(sys.argv[2])
    kernels(figures / "visibility-kernel.png")
    basin_figure(json.loads((measured / "basin.json").read_text()), figures / "visibility-basin.png")


if __name__ == "__main__":
    main()
