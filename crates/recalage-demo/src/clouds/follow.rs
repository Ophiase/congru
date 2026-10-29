use recalage::{Distance, Error, Icp, Kernel, Multistart, OrientedCloud, PointCloud, Target};
use recalage_synth::sequence::Shot;
use recalage_track::{Gains, Health, Tracker, Update, WholeModel};

const SCAN_VOXEL: f64 = 0.005;
const KERNEL_SCALE: f64 = 0.006;
const CORRESPONDENCE_LIMIT: f64 = 0.025;
const ITERATIONS_PER_FRAME: u32 = 15;
const STARTS: usize = 64;
const HEALTH: Health = Health {
    min_inliers: 0.6,
    max_rmse: 0.01,
};
const GAINS: Gains = Gains {
    pose: 1.0,
    velocity: 0.5,
};

/// A frame, and the tracker's answer to it.
#[derive(Debug, Clone, PartialEq)]
pub struct Record {
    pub shot: Shot,
    pub update: Option<Update>,
}

impl Record {
    pub fn error_degrees(&self) -> f64 {
        self.update.as_ref().map_or(f64::INFINITY, |update| {
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

fn tracker(model: &OrientedCloud) -> Result<Tracker<WholeModel>, Error> {
    let kernel = Kernel::Cauchy(Distance::new(KERNEL_SCALE)?);
    let follow = Icp {
        kernel,
        max_correspondence_distance: Some(Distance::new(CORRESPONDENCE_LIMIT)?),
        convergence: recalage::Convergence {
            max_iterations: ITERATIONS_PER_FRAME,
            ..recalage::Convergence::default()
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
    let view = WholeModel(Target::from_oriented(model.clone())?);
    Ok(Tracker::new(view, follow, find, HEALTH, GAINS))
}
