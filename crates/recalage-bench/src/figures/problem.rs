use recalage::{NearestNeighbors, PointCloud};
use recalage_synth::{Corruption, Outliers, Resampling};

use super::toy::{sight, spiral, together, turn};
use crate::draw::{Figure, Ink, View};

const COUNT_LABELS: [&str; 6] = ["N=0", "N=1", "N=2", "N=3", "N=4", "N=5+"];
const TURN_DEGREES: f64 = 35.0;

pub fn figure() -> anyhow::Result<String> {
    let model = spiral()?;
    let corruption = Corruption {
        resampling: Resampling::Poisson(1.5),
        noise: 0.14,
        outliers: Outliers::InBox { count: 12 },
        ..Corruption::default()
    };
    let seen = sight(&model, &corruption, 7)?;
    let split = seen.observed.count() - seen.outliers;
    let (samples, outliers) = seen.observed.points().split_at(split);
    let moved = turn(&seen.observed, TURN_DEGREES);
    let (moved_samples, moved_outliers) = moved.points().split_at(split);
    let mut figure = Figure::new(View::from_above(&together(&model, &moved)));
    figure
        .panel("1. each cross seen Nᵢ times, with noise")
        .crosses(model.points(), Ink::Model)
        .dots(samples, Ink::Observed)
        .labels(
            model.points(),
            &counts(&model, &PointCloud::new(samples.to_vec())?)?,
        );
    figure
        .panel("2. points from nowhere: outliers")
        .crosses(model.points(), Ink::Model)
        .dots(samples, Ink::Observed)
        .dots(outliers, Ink::Outlier);
    figure
        .panel("3. all moved by an unknown (R, t)")
        .crosses(model.points(), Ink::Model)
        .dots(moved_samples, Ink::Observed)
        .dots(moved_outliers, Ink::Outlier);
    Ok(figure.render()?)
}

fn counts(model: &PointCloud, samples: &PointCloud) -> anyhow::Result<Vec<&'static str>> {
    let index = NearestNeighbors::new(model)?;
    let mut tally = vec![0_usize; model.count()];
    for sample in samples.points() {
        tally[index.nearest(sample).index] += 1;
    }
    Ok(tally
        .into_iter()
        .map(|count| COUNT_LABELS[count.min(COUNT_LABELS.len() - 1)])
        .collect())
}
