use std::time::Instant;

use congru::{Distance, Target};
use congru_embedded::{KdTree, Real, Workspace};
use congru_sim::bracket;

use crate::aligners::{Trial, coordinates, embedded_trial, main_trial};
use crate::plan::{SCALES, SEEDS, SETUPS, SITUATIONS, Scale, frame};

/// The fastest of a few runs: one alone may catch the machine busy.
const REPEATS: usize = 5;

/// Each implementation's trials in one setup and situation.
pub struct Cell {
    pub setup: usize,
    pub situation: usize,
    pub main: Vec<Trial>,
    pub double: Vec<Trial>,
    pub single: Vec<Trial>,
}

pub struct Memory {
    pub target_points: usize,
    pub source_points: usize,
    pub single_tree_bytes: usize,
    pub double_tree_bytes: usize,
    pub single_workspace_bytes: usize,
    pub double_workspace_bytes: usize,
    pub normals_seconds: f64,
    pub build_seconds: f64,
}

pub struct Scene {
    pub scale: &'static Scale,
    pub memory: Memory,
    pub cells: Vec<Cell>,
}

/// # Errors
/// A mesh that cannot be sampled or a tree that cannot be built.
pub fn run() -> anyhow::Result<Vec<Scene>> {
    SCALES.iter().map(scene).collect()
}

fn scene(scale: &'static Scale) -> anyhow::Result<Scene> {
    let model = bracket()?;
    let target =
        Target::from_oriented(model.sample_surface(Distance::new(scale.target_spacing)?)?)?;
    let scan = model
        .sample_surface(Distance::new(scale.source_spacing)?)?
        .cloud()
        .clone();
    let single_tree = tree_of::<f32>(&target)?;
    let double_tree = tree_of::<f64>(&target)?;
    let memory = memory_of(&target, scan.count(), &single_tree, &double_tree)?;
    let mut cells = Vec::new();
    for setup in scale.setups {
        for situation in 0..SITUATIONS.len() {
            cells.push(cell(
                *setup,
                situation,
                &scan,
                &target,
                &single_tree,
                &double_tree,
            )?);
        }
    }
    Ok(Scene {
        scale,
        memory,
        cells,
    })
}

fn memory_of(
    target: &Target,
    source_points: usize,
    single: &KdTree<f32>,
    double: &KdTree<f64>,
) -> anyhow::Result<Memory> {
    let raw: Vec<[f32; 3]> = coordinates(target.cloud().points());
    let (mut build_seconds, mut normals_seconds) = (f64::INFINITY, f64::INFINITY);
    for _ in 0..REPEATS {
        let started = Instant::now();
        let mut built = KdTree::new(&raw)?;
        build_seconds = build_seconds.min(started.elapsed().as_secs_f64());
        let started = Instant::now();
        built.estimate_normals::<10>()?;
        normals_seconds = normals_seconds.min(started.elapsed().as_secs_f64());
    }
    Ok(Memory {
        target_points: raw.len(),
        source_points,
        single_tree_bytes: single.heap_bytes(),
        double_tree_bytes: double.heap_bytes(),
        single_workspace_bytes: Workspace::<f32>::new(source_points).heap_bytes(),
        double_workspace_bytes: Workspace::<f64>::new(source_points).heap_bytes(),
        normals_seconds,
        build_seconds,
    })
}

fn tree_of<Float: Real>(target: &Target) -> anyhow::Result<KdTree<Float>> {
    let normals = target
        .normals()
        .ok_or_else(|| anyhow::anyhow!("a sampled surface has normals"))?;
    let normals: Vec<[Float; 3]> = normals
        .iter()
        .map(|normal| [normal.x, normal.y, normal.z].map(Float::of))
        .collect();
    Ok(KdTree::with_normals(
        &coordinates(target.cloud().points()),
        &normals,
    )?)
}

fn cell(
    setup: usize,
    situation: usize,
    scan: &congru::PointCloud,
    target: &Target,
    single_tree: &KdTree<f32>,
    double_tree: &KdTree<f64>,
) -> anyhow::Result<Cell> {
    let mut built = Cell {
        setup,
        situation,
        main: Vec::new(),
        double: Vec::new(),
        single: Vec::new(),
    };
    for seed in 0..SEEDS {
        let (source, truth, guess) = frame(scan, &SITUATIONS[situation], seed);
        built
            .main
            .push(main_trial(&SETUPS[setup], &source, target, &truth, &guess)?);
        built.double.push(embedded_trial(
            &SETUPS[setup],
            &source,
            double_tree,
            &truth,
            &guess,
        ));
        built.single.push(embedded_trial(
            &SETUPS[setup],
            &source,
            single_tree,
            &truth,
            &guess,
        ));
    }
    Ok(built)
}
