use congru::{Distance, Error, OrientedCloud, PointCloud};
use congru_synth::sequence::Shot;
use congru_track::{SensorView, Settings, Tracker, Update};

const SCAN_VOXEL: f64 = 0.005;
const PART_SPAN: f64 = 0.2;

/// A frame, and the tracker's answer to it.
#[derive(Debug, Clone, PartialEq)]
pub struct Record {
    pub shot: Shot,
    pub update: Option<Update>,
}

impl Record {
    pub fn error_degrees(&self) -> f64 {
        self.update
            .as_ref()
            .filter(|update| update.trusted)
            .map_or(f64::INFINITY, |update| {
                update.pose.rotation_error_to(&self.shot.truth).to_degrees()
            })
    }
}

pub fn follow(model: &OrientedCloud, shots: Vec<Shot>) -> Result<Vec<Record>, Error> {
    let mut tracker = tracker(model)?;
    let voxel = Distance::new(SCAN_VOXEL)?;
    Ok(shots
        .into_iter()
        .map(|shot| {
            let scan: PointCloud = shot.cloud.voxel_downsample(voxel);
            let update = tracker.update(&scan).ok();
            Record { shot, update }
        })
        .collect())
}

fn tracker(model: &OrientedCloud) -> Result<Tracker<SensorView>, Error> {
    Ok(Tracker::new(
        "tracker",
        SensorView::new(model.clone())?,
        Settings::for_part(PART_SPAN)?,
    ))
}
