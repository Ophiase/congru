use recalage_sim::Degradation;

/// One way of seeing the scene: how the depth is degraded, and whether the
/// turntable is in view as clutter.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Condition {
    pub name: &'static str,
    pub degradation: Degradation,
    pub clutter: bool,
}

pub const CONDITIONS: [Condition; 4] = [
    Condition {
        name: "exact depth",
        degradation: Degradation {
            blur: 0,
            noise: 0.0,
            holes: 0.0,
            flying_pixels: false,
        },
        clutter: false,
    },
    Condition {
        name: "noise and blur",
        degradation: Degradation {
            blur: 1,
            noise: 0.02,
            holes: 0.0,
            flying_pixels: false,
        },
        clutter: false,
    },
    Condition {
        name: "holes and flying pixels",
        degradation: Degradation {
            blur: 0,
            noise: 0.01,
            holes: 0.15,
            flying_pixels: true,
        },
        clutter: false,
    },
    Condition {
        name: "everything, and the turntable",
        degradation: Degradation {
            blur: 1,
            noise: 0.02,
            holes: 0.15,
            flying_pixels: true,
        },
        clutter: true,
    },
];
