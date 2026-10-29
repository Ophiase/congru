//! Demo A: a model moving in front of a sensor, seen as noisy point clouds,
//! hidden, distorted, knocked; the tracker follows it and finds it again.

mod follow;
mod lens;
mod scenarios;

use std::path::Path;

use nalgebra::Vector3;
use congru::{Distance, OrientedCloud, Point, RigidTransform};
use congru_sim::bracket;

use crate::Summary;
use crate::picture::palette::{BAD, FAIR, GOOD, MODEL, OBSERVED, TEXT, Tone};
use crate::picture::{self, Tile, font};
use follow::{Record, follow};
use lens::{Layer, Lens};
use congru_track::Phase;
use congru_vision::Image;
use scenarios::{FRAMES, scenarios};

const TILE_WIDTH: usize = 260;
const TILE_HEIGHT: usize = 200;
const MODEL_SPACING: f64 = 0.003;
const DRAWN_MODEL_STRIDE: usize = 7;
const SEED: u64 = 7;
const STANDING_ANGLE: f64 = 118.0;

pub fn run(target: &Path) -> anyhow::Result<Vec<Summary>> {
    let model = standing(&bracket()?.sample_surface(Distance::new(MODEL_SPACING)?)?)?;
    let runs = scenarios()
        .into_iter()
        .map(|scenario| {
            let shots = scenario.sequence.shoot(model.cloud(), SEED)?;
            Ok((scenario.name, follow(&model, shots)?))
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    let frames: Vec<Vec<Tile>> = (0..FRAMES)
        .map(|frame| {
            runs.iter()
                .map(|(name, records)| tile(name, &records[frame], &model))
                .collect()
        })
        .collect();
    picture::write(target, &frames, 2)?;
    Ok(runs
        .iter()
        .map(|(name, records)| Summary::of(name, records.iter().map(Record::error_degrees)))
        .collect())
}

fn tile(name: &str, record: &Record, model: &OrientedCloud) -> Tile {
    let lens = Lens::around(
        &Point::from(record.shot.truth.translation()),
        TILE_WIDTH,
        TILE_HEIGHT,
    );
    let placed: Vec<Point> = record.update.as_ref().map_or_else(Vec::new, |update| {
        model
            .cloud()
            .points()
            .iter()
            .step_by(DRAWN_MODEL_STRIDE)
            .map(|point| update.pose.apply(point))
            .collect()
    });
    let mut picture = lens.draw(&[
        Layer {
            points: record.shot.cloud.points(),
            tone: OBSERVED,
            size: 2,
        },
        Layer {
            points: &placed,
            tone: MODEL,
            size: 2,
        },
    ]);
    let error = label(&mut picture, name, record);
    Tile {
        picture,
        error_degrees: error,
    }
}

/// The bracket stood up and tipped toward the sensor, as a part held in
/// front of it would be, instead of seen from straight above.
fn standing(model: &OrientedCloud) -> Result<OrientedCloud, congru::Error> {
    let tip = RigidTransform::from_axis_angle(Vector3::x() * STANDING_ANGLE.to_radians(), [0.0; 3]);
    let normals: Vec<Vector3<f64>> = model
        .normals()
        .iter()
        .map(|normal| tip.rotation() * normal.into_inner())
        .collect();
    OrientedCloud::new(model.cloud().transformed(&tip), &normals)
}

fn label(picture: &mut Image<Tone>, name: &str, record: &Record) -> f64 {
    if record.shot.jumped {
        font::write(picture, TILE_WIDTH - 70, 8, "knock", BAD);
    }
    font::write(picture, 8, 8, name, TEXT);
    let (phase, tone) = match record.update.as_ref().map(|update| update.phase) {
        Some(Phase::Follow) => ("follow", GOOD),
        Some(Phase::Find) => ("find", FAIR),
        None => ("lost", BAD),
    };
    let error = record.error_degrees();
    font::write(
        picture,
        8,
        TILE_HEIGHT - 22,
        &format!("{phase} {error:.1}°"),
        tone,
    );
    error
}
