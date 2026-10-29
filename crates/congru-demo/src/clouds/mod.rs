//! Demo A: a model moving in front of a sensor, seen as noisy point clouds,
//! hidden, distorted, knocked; the tracker follows it and finds it again.

mod follow;
mod lens;
mod scenarios;

use std::path::Path;

use congru::{Distance, OrientedCloud, Point, RigidTransform};
use congru_sim::bracket;
use nalgebra::Vector3;

use crate::Summary;
use crate::picture::palette::{BAD, FAIR, GOOD, MODEL, OBSERVED, TEXT, Tone};
use crate::picture::{self, Tile, font};
use congru_track::Phase;
use congru_vision::Image;
use follow::{Record, follow};
use lens::{Layer, Lens};
use scenarios::{FRAMES, scenarios};

const TILE_WIDTH: usize = 240;
const TILE_HEIGHT: usize = 190;
const MODEL_SPACING: f64 = 0.003;
const DRAWN_MODEL_STRIDE: usize = 3;
const SEED: u64 = 7;
const PART_SPAN: f64 = 0.2;
const STANDING_ANGLE: f64 = 118.0;

pub fn run(target: &Path) -> anyhow::Result<Vec<Summary>> {
    let model = standing(&bracket()?.sample_surface(Distance::new(MODEL_SPACING)?)?)?;
    let runs = scenarios()
        .into_iter()
        .map(|scenario| {
            let shots = scenario.sequence.shoot(model.cloud(), PART_SPAN, SEED)?;
            let records = follow(&model, shots)?;
            if std::env::var_os("CONGRU_TRACE").is_some_and(|wanted| wanted == scenario.name) {
                trace(&records);
            }
            Ok((scenario.name, records))
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    let frames: Vec<Vec<Tile>> = (0..FRAMES)
        .map(|frame| {
            runs.iter()
                .map(|(name, records)| {
                    let centre = Point::from(records[0].shot.truth.translation());
                    tile(
                        name,
                        &records[frame],
                        &model,
                        &Lens::around(&centre, TILE_WIDTH, TILE_HEIGHT),
                    )
                })
                .collect()
        })
        .collect();
    picture::write(target, &frames, 3)?;
    Ok(runs
        .iter()
        .map(|(name, records)| Summary::of(name, records.iter().map(Record::error_degrees)))
        .collect())
}

fn tile(name: &str, record: &Record, model: &OrientedCloud, lens: &Lens) -> Tile {
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
            points: &placed,
            tone: MODEL,
            size: 1,
        },
        Layer {
            points: record.shot.cloud.points(),
            tone: OBSERVED,
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

fn trace(records: &[Record]) {
    use std::io::Write;
    let mut output = std::io::stderr().lock();
    for (frame, record) in records.iter().enumerate() {
        let detail = record.update.as_ref().map(|update| {
            (
                update.phase,
                update.trusted,
                update.alignment.inlier_fraction,
                update.alignment.rmse,
                update.alignment.iterations,
            )
        });
        let _ = writeln!(
            output,
            "{frame:3} {:6.2} {} {detail:?}",
            record.error_degrees(),
            record.shot.cloud.count()
        );
    }
}
