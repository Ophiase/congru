use congru::{Distance, Error, Icp, Kernel, Multistart, OrientedCloud, PointCloud};
use congru_synth::sequence::Shot;
use congru_track::{Agility, Gains, Health, SensorView, Tracker, Update};

const SCAN_VOXEL: f64 = 0.005;
const FIRST_KERNEL_SCALE: f64 = 0.01;
const FIRST_GATE: f64 = 0.03;
const ITERATIONS_PER_FRAME: u32 = 20;
const STARTS: usize = 64;
const NOISE_FLOOR: f64 = 0.002;
const HEALTH: Health = Health {
    min_inliers: 0.5,
    max_noise_ratio: 2.0,
};
const AGILITY: Agility = Agility {
    rotation: 0.08,
    translation: 0.01,
};
const GAINS: Gains = Gains {
    pose: 0.6,
    velocity: 0.2,
};

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
    let kernel = Kernel::Cauchy(Distance::new(FIRST_KERNEL_SCALE)?);
    let follow = Icp {
        kernel,
        max_correspondence_distance: Some(Distance::new(FIRST_GATE)?),
        convergence: congru::Convergence {
            max_iterations: ITERATIONS_PER_FRAME,
            ..congru::Convergence::default()
        },
        ..Icp::default()
    };
    let find = Multistart {
        starts: STARTS,
        icp: Icp {
            kernel,
            ..Icp::default()
        },
    };
    let view = SensorView::new(model.clone())?;
    Ok(Tracker::new(
        view,
        follow,
        find,
        HEALTH,
        AGILITY,
        GAINS,
        NOISE_FLOOR,
    ))
}
