use std::fmt::Write;

use crate::aligners::Trial;
use crate::elf::Image;
use crate::plan::{GATE, KERNEL_SCALE, SEEDS, SETUPS, SITUATIONS};
use crate::trials::{Cell, Scene};

const NEAR_DEGREES: f64 = 1.0;
const NEAR_MILLIMETRES: f64 = 5.0;

pub fn render(single: &Image, double: &Image, scenes: &[Scene]) -> String {
    let mut page = format!(
        "# Embedded\n\n\
         Written by `just embedded-measure`. Code is measured on images linked for\n\
         `thumbv7em-none-eabihf`, time and accuracy on this {} machine, one thread.\n\n\
         - **Not run on a board**: this recipe links for the target and stops there,\n  \
         so nothing here is a cycle count. Where `f64` is emulated, its ratio to\n  \
         `f32` will be far worse than on this machine.\n\
         - **Same frames**: a bracket as in [the comparison](comparison.md), {SEEDS} seeds,\n  \
         a gate of {GATE} m, at most 50 iterations; Cauchy of scale {KERNEL_SCALE} m.\n\n",
        std::env::consts::ARCH
    );
    code_section(&mut page, single, double);
    memory_section(&mut page, scenes);
    time_section(&mut page, scenes);
    for scene in scenes {
        accuracy_section(&mut page, scene);
    }
    page.truncate(page.trim_end().len());
    page.push('\n');
    page
}

fn code_section(page: &mut String, single: &Image, double: &Image) {
    page.push_str(
        "## Code\n\n\
         One image: every fit, the tree, normals, both metrics, a panic that halts\n\
         and a bump allocator. Optimised for size with full link-time optimisation.\n\n\
         | Scalar | Flash, bytes | Helpers for soft `f64` |\n| --- | ---: | ---: |\n",
    );
    for (name, image) in [("`f32`", single), ("`f64`", double)] {
        let _ = writeln!(
            page,
            "| {name} | {} | {} |",
            image.flash_bytes, image.double_helpers
        );
    }
    page.push('\n');
}

fn memory_section(page: &mut String, scenes: &[Scene]) {
    page.push_str(
        "## Data\n\n\
         The whole heap of an alignment, read from the structures. The caller also\n\
         holds the source, 12 bytes a point in `f32`. A query adds a fixed stack.\n\n\
         | Scene | Target points | Source points | Tree `f32` | Tree `f64` | Buffer `f32` | Buffer `f64` |\n\
         | --- | ---: | ---: | ---: | ---: | ---: | ---: |\n",
    );
    for scene in scenes {
        let memory = &scene.memory;
        let _ = writeln!(
            page,
            "| {} | {} | {} | {} | {} | {} | {} |",
            scene.scale.name,
            memory.target_points,
            memory.source_points,
            memory.single_tree_bytes,
            memory.double_tree_bytes,
            memory.single_workspace_bytes,
            memory.double_workspace_bytes,
        );
    }
    let _ = writeln!(
        page,
        "\nStack of one query: {} bytes in `f32`.\n",
        congru_embedded::KdTree::<f32>::query_stack_bytes()
    );
}

fn time_section(page: &mut String, scenes: &[Scene]) {
    page.push_str(
        "## Time per iteration\n\n\
         All frames of a setup: total time over total iterations, in microseconds.\n\n\
         | Scene | Setup | `congru` | `f64` here | `f32` here | `f64` over `f32` |\n\
         | --- | --- | ---: | ---: | ---: | ---: |\n",
    );
    for scene in scenes {
        for setup in scene.scale.setups {
            let cells: Vec<&Cell> = scene
                .cells
                .iter()
                .filter(|cell| cell.setup == *setup)
                .collect();
            let main = per_iteration(cells.iter().flat_map(|cell| &cell.main));
            let wide = per_iteration(cells.iter().flat_map(|cell| &cell.double));
            let narrow = per_iteration(cells.iter().flat_map(|cell| &cell.single));
            let _ = writeln!(
                page,
                "| {} | {} | {main:.0} | {wide:.0} | {narrow:.0} | {:.2} |",
                scene.scale.name,
                SETUPS[*setup].name,
                wide / narrow
            );
        }
    }
    let last = scenes.iter().map(|scene| &scene.memory);
    page.push_str(
        "\nBuilding the tree and estimating normals, `f32`, once:\n\n\
         | Target points | Tree, ms | Normals of 10 neighbours, ms |\n| ---: | ---: | ---: |\n",
    );
    for memory in last {
        let _ = writeln!(
            page,
            "| {} | {:.1} | {:.1} |",
            memory.target_points,
            memory.build_seconds * 1e3,
            memory.normals_seconds * 1e3
        );
    }
    page.push('\n');
}

fn per_iteration<'trial>(trials: impl Iterator<Item = &'trial Trial>) -> f64 {
    let (seconds, iterations) = trials.fold((0.0, 0_u32), |(seconds, iterations), trial| {
        (seconds + trial.seconds, iterations + trial.iterations)
    });
    seconds * 1e6 / f64::from(iterations.max(1))
}

fn accuracy_section(page: &mut String, scene: &Scene) {
    let _ = write!(
        page,
        "## Accuracy, {}\n\n\
         Median error over the seeds, then how many came within {NEAR_DEGREES}° and\n\
         {NEAR_MILLIMETRES} mm of the truth.\n\n\
         | Setup | Situation | `congru` °, mm, near | `f64` °, mm, near | `f32` °, mm, near |\n\
         | --- | --- | ---: | ---: | ---: |\n",
        scene.scale.name
    );
    for cell in &scene.cells {
        let _ = writeln!(
            page,
            "| {} | {} | {} | {} | {} |",
            SETUPS[cell.setup].name,
            SITUATIONS[cell.situation].name,
            summary(&cell.main),
            summary(&cell.double),
            summary(&cell.single),
        );
    }
    page.push('\n');
}

fn summary(trials: &[Trial]) -> String {
    let answers: Vec<(f64, f64)> = trials
        .iter()
        .filter_map(|trial| trial.errors)
        .map(|(rotation, translation)| (rotation.to_degrees(), translation * 1e3))
        .collect();
    let near = answers
        .iter()
        .filter(|(rotation, translation)| {
            *rotation < NEAR_DEGREES && *translation < NEAR_MILLIMETRES
        })
        .count();
    format!(
        "{:.3}, {:.2}, {near}/{}",
        median(answers.iter().map(|answer| answer.0)),
        median(answers.iter().map(|answer| answer.1)),
        trials.len()
    )
}

fn median(values: impl Iterator<Item = f64>) -> f64 {
    let mut values: Vec<f64> = values.collect();
    values.sort_by(f64::total_cmp);
    values.get(values.len() / 2).copied().unwrap_or(f64::NAN)
}
