//! Every result checked against a motion known beforehand, in both scalars.

#![expect(
    clippy::unwrap_used,
    reason = "a broken fixture should fail the test loudly"
)]

mod fit;
mod fixtures;
mod icp;
mod refusal;
mod search;
