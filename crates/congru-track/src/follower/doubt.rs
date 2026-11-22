use congru::RigidTransform;

use super::{Follower, Sight};

/// A search beats a believed pose only past this many times its confirmation.
const MARGIN: f64 = 1.1;
/// Two wins of a rival count as one rival within this turn, in radians: the object
/// moves a little between two looks.
const SAME_RIVAL: f64 = 0.2;

impl Follower {
    /// `forecast`, or a rival that beat it at two looks running: a single search can
    /// win by chance on a noisy frame, where a right pose must not change. The
    /// trajectory then starts over.
    pub(super) fn challenged(
        &mut self,
        sight: Sight<'_>,
        forecast: RigidTransform,
    ) -> RigidTransform {
        let due = self
            .doubt_every
            .is_some_and(|every| self.frame > 0 && self.frame.is_multiple_of(every));
        if !due {
            return forecast;
        }
        let winner = self
            .finder
            .located_in(sight.depth, sight.region, &self.camera)
            .filter(|found| {
                let confirmed = |pose: &RigidTransform| {
                    self.finder.confirmed_at(sight.depth, &self.camera, pose)
                };
                confirmed(found) > MARGIN * confirmed(&forecast).max(0.0)
            });
        let again = winner.filter(|found| {
            self.rival
                .is_some_and(|rival| found.rotation_error_to(&rival) < SAME_RIVAL)
        });
        self.rival = winner;
        match again {
            Some(found) => {
                self.rival = None;
                self.hold.forget();
                found
            }
            None => forecast,
        }
    }
}
